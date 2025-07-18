#![allow(non_snake_case)]

use interceptor::Interceptor;
use windows::Win32::{
    Foundation::HINSTANCE,
    System::{Console, SystemServices::DLL_PROCESS_ATTACH},
};

mod bin_util;
mod interceptor;
mod nt_util;
mod patches;

fn on_attach() {
    const DUMP: &[u8] = include_bytes!("../symphony.bin");

    unsafe {
        let _ = Console::FreeConsole();
        let _ = Console::AllocConsole();
    }

    let mut interceptor = Interceptor::new();

    patches::ace::restore_executable_from_dump(DUMP);
    patches::login_ui::enable_gm_login_button(&mut interceptor);
    patches::http::hook_http_requests(&mut interceptor);

    interceptor.leak();

    println!("Symphonic successfully initialized. Time to play Neverness to Everness!");
    println!("Copyright 2025, ReversedRooms. All bytes reversed.");
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
unsafe extern "system" fn DllMain(_: HINSTANCE, call_reason: u32, _: *mut ()) -> bool {
    if call_reason == DLL_PROCESS_ATTACH {
        on_attach();
    }

    true
}
