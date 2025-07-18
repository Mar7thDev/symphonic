use crate::{interceptor::Interceptor, nt_util};

#[repr(C)]
struct WidgetVTable {
    gap: [u8; 728],
    set_visibility: extern "fastcall" fn(usize, usize),
}

pub fn enable_gm_login_button(interceptor: &mut Interceptor) {
    const UHTUI_LOGIN_SETUP_CALLBACKS: usize = 0x6C40034;

    let base = nt_util::get_module_base(None);

    interceptor.attach(base + UHTUI_LOGIN_SETUP_CALLBACKS, |ctx| {
        let btn_account = ctx.registers().rcx as usize;
        let parent_object = unsafe {
            *((*(btn_account.wrapping_add(48) as *const usize)).wrapping_add(40) as *const usize)
        };

        unsafe {
            let vtable = *(parent_object as *const *const WidgetVTable);
            ((*vtable).set_visibility)(parent_object, 0x19);
        }
    });
}
