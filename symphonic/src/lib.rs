#![allow(non_snake_case)]

use interceptor::Interceptor;
use windows::Win32::{
    Foundation::HINSTANCE,
    System::{Console, SystemServices::DLL_PROCESS_ATTACH},
};

mod bin_util;
mod generated_offsets;
mod interceptor;
mod logging;
mod nt_util;
mod patches;

fn on_attach() {
    const DUMP: &[u8] = include_bytes!("../symphony.bin");

    logging::init();

    unsafe {
        let _ = Console::FreeConsole();
        let _ = Console::AllocConsole();
    }

    let mut interceptor = Interceptor::new();

    patches::ace::restore_executable_from_dump(DUMP);
    logging::write(format_args!("executable restored; installing hooks"));
    patches::login_ui::enable_gm_login_button(&mut interceptor);
    patches::http::hook_http_requests(&mut interceptor);
    let entry = u32::from_le_bytes(DUMP[DUMP.len()-4..].try_into().unwrap()) as usize;
    interceptor.attach(nt_util::get_module_base(None) + entry, |_| {
        logging::write(format_args!("real entry point reached"));
    });

    interceptor.leak();

    println!("Symphonic successfully initialized. Time to play Neverness to Everness!");
    println!("Copyright 2025, ReversedRooms. All bytes reversed.");
    logging::write(format_args!("DllMain initialization complete; returning to loader"));
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
unsafe extern "system" fn DllMain(_: HINSTANCE, call_reason: u32, _: *mut ()) -> bool {
    if call_reason == DLL_PROCESS_ATTACH {
        on_attach();
    }

    true
}
