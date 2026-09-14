use crate::{generated_offsets, interceptor::Interceptor, nt_util};

pub fn hook_http_requests(interceptor: &mut Interceptor) {
    let base = nt_util::get_module_base(None);

    hook_get_serverlist_url(base, interceptor);
    hook_curl_http_request_set_url(base, interceptor);
}

fn hook_get_serverlist_url(base: usize, interceptor: &mut Interceptor) {
    const CUSTOM_URL: &str = "http://127.0.0.1:10001/serverlist_hhtest.json";

    interceptor.replace(base + generated_offsets::GET_SERVERLIST_URL, |ctx| {
        crate::trace!("GetServerListURL called");

        let get_serverlist_url = unsafe {
            std::mem::transmute::<usize, extern "win64" fn(u64, u64) -> usize>(ctx.original_fn)
        };

        let result = get_serverlist_url(ctx.registers().rcx, ctx.registers().rdx);

        let f_string = ctx.registers().rdx as usize;
        let buf = CUSTOM_URL.encode_utf16().chain([0]).collect::<Vec<_>>();
        unsafe {
            assert!(*(f_string.wrapping_add(12) as *const i32) >= buf.len() as i32,
                "server list FString capacity is too small");
            std::ptr::copy(
                buf.as_ptr(),
                (*(f_string as *const usize)) as *mut u16,
                buf.len(),
            );
            *(f_string.wrapping_add(8) as *mut i32) = buf.len() as i32;
        }
        crate::trace!("server list redirected to {CUSTOM_URL}");

        result
    });
}

fn hook_curl_http_request_set_url(base: usize, interceptor: &mut Interceptor) {
    const CUSTOM_URL_PREFIX: &str = "http://127.0.0.1:10001";

    interceptor.replace(base + generated_offsets::FCURL_HTTP_REQUEST_SET_URL, |ctx| {
        let f_string = ctx.registers().rdx as usize;
        let original = unsafe {
            std::mem::transmute::<usize, extern "win64" fn(u64, usize) -> usize>(ctx.original_fn)
        };
        let str_buf = unsafe {
            if f_string == 0 { return original(ctx.registers().rcx, f_string); }
            let count = *(f_string.wrapping_add(8) as *const i32);
            if count <= 0 { return original(ctx.registers().rcx, f_string); }
            let length = count - 1;
            std::slice::from_raw_parts(*(f_string as *const usize) as *const u16, length as usize)
        };

        let url = String::from_utf16_lossy(str_buf);
        crate::trace!("FCurlHttpRequest::SetURL: {url}");

        let mut new_url = String::from(CUSTOM_URL_PREFIX);
        url.split('/').skip(3).for_each(|s| {
            new_url.push('/');
            new_url.push_str(s);
        });

        let buf = new_url.encode_utf16().chain([0]).collect::<Vec<_>>();
        #[repr(C)]
        struct FStringRef { data: *const u16, count: i32, capacity: i32 }
        let redirected = FStringRef { data: buf.as_ptr(), count: buf.len() as i32, capacity: buf.len() as i32 };
        crate::trace!("HTTP redirected to {new_url}");
        // SetURL takes a const FString reference and copies it into the request.
        // Keep the temporary alive through that call, without touching the
        // caller's allocation or using a Rust allocation as UE-owned memory.
        original(ctx.registers().rcx, &redirected as *const _ as usize)
    });
}
