use crate::{generated_offsets, interceptor::Interceptor, nt_util};

pub fn enable_gm_login_button(interceptor: &mut Interceptor) {
    let base = nt_util::get_module_base(None);

    interceptor.attach(base + generated_offsets::UHTUI_LOGIN_SETUP_CALLBACKS, |ctx| {
        let login = ctx.registers().rdi as usize;
        // Immediately before this binding block the game calls this same
        // UHTUIBase virtual helper with CanvasPanel_AccountButton and the SDK
        // mode flag. Use its boolean API rather than an old UWidget vtable slot.
        unsafe {
            let panel = *((login + generated_offsets::LOGIN_ACCOUNT_PANEL) as *const usize);
            let vtable = *(login as *const usize);
            let helper = *((vtable + generated_offsets::LOGIN_SHOW_PANEL) as *const usize);
            crate::trace!("enabling local login panel login={login:#x} panel={panel:#x} helper={helper:#x}");
            let show = std::mem::transmute::<usize, extern "win64" fn(usize, usize, bool)>(helper);
            show(login, panel, true);
        }
    });

    // The shipping build's manual branch sets the mode and returns without
    // invoking TryLogin. Reconnect it to the existing native local login path.
    interceptor.replace(base + generated_offsets::LOCAL_LOGIN_ENTRY, |ctx| {
        unsafe { login_local(ctx.registers().rcx as usize); }
        0
    });
}

#[repr(C)]
#[derive(Default)]
struct FString { data: usize, count: i32, capacity: i32 }

fn borrowed_string(text: &[u16]) -> FString {
    FString { data: text.as_ptr() as usize, count: text.len() as i32, capacity: text.len() as i32 }
}

unsafe fn login_local(login: usize) {
    let base = nt_util::get_module_base(None);
    let free: extern "win64" fn(usize) = unsafe { std::mem::transmute(base + generated_offsets::FMEMORY_FREE) };
    let assign: extern "win64" fn(usize, &FString) -> usize = unsafe { std::mem::transmute(base + generated_offsets::FSTRING_ASSIGN) };
    let submit: extern "win64" fn(usize, &FString, &FString, u8, &FString, bool) = unsafe { std::mem::transmute(base + generated_offsets::LOCAL_LOGIN_SUBMIT) };
    let server: Vec<u16> = "127.0.0.1:30031\0".encode_utf16().collect();
    let token: Vec<u16> = "local\0".encode_utf16().collect();
    let fallback: Vec<u16> = "nte-local\0".encode_utf16().collect();
    let mut account = FString::default();
    unsafe {
        let widget = *((login + generated_offsets::LOGIN_ACCOUNT) as *const usize);
        if widget != 0 {
            let get_text: extern "win64" fn(usize, *mut u8, *mut usize) = std::mem::transmute(base + generated_offsets::EDITABLE_TEXT_GET_TEXT);
            let to_string: extern "win64" fn(&mut FString, *const usize) -> usize = std::mem::transmute(base + generated_offsets::TEXT_TO_STRING);
            let mut frame = [0u64; 24];
            let mut text = [0usize; 2];
            get_text(widget, frame.as_mut_ptr().cast(), text.as_mut_ptr());
            to_string(&mut account, text.as_ptr());
            if text[0] != 0 {
                let vtable = *(text[0] as *const usize);
                let release: extern "win64" fn(usize) = std::mem::transmute(*((vtable + 0x10) as *const usize));
                release(text[0]);
            }
        }
        assign(login + generated_offsets::LOGIN_SERVER_ADDRESS, &borrowed_string(&server));
        crate::trace!("submitting local GM login to 127.0.0.1:30031");
        let fallback = borrowed_string(&fallback);
        let account_ref = if account.count > 1 { &account } else { &fallback };
        submit(login, account_ref, &borrowed_string(&token), 3, &FString::default(), false);
        if account.data != 0 { free(account.data); }
        crate::trace!("local GM login submission returned");
    }
}
