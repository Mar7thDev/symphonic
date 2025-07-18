use crate::{interceptor::Interceptor, nt_util};

pub fn hook_http_requests(interceptor: &mut Interceptor) {
    let base = nt_util::get_module_base(None);

    hook_get_serverlist_url(base, interceptor);
    hook_curl_http_request_set_url(base, interceptor);
}

fn hook_get_serverlist_url(base: usize, interceptor: &mut Interceptor) {
    const GET_SERVERLIST_URL: usize = 0x671B940;
    const CUSTOM_URL: &str = "http://127.0.0.1:10001/serverlist_hhtest.json";

    interceptor.replace(base + GET_SERVERLIST_URL, |ctx| {
        println!("GetServerListURL called");

        let get_serverlist_url = unsafe {
            std::mem::transmute::<usize, extern "fastcall" fn(u64, u64) -> usize>(ctx.original_fn)
        };

        let result = get_serverlist_url(ctx.registers().rcx, ctx.registers().rdx);

        let f_string = ctx.registers().rdx as usize;
        let buf = CUSTOM_URL.encode_utf16().chain([0]).collect::<Vec<_>>();
        unsafe {
            std::ptr::copy(
                buf.as_ptr(),
                (*(f_string as *const usize)) as *mut u16,
                buf.len(),
            );
        }

        result
    });
}

fn hook_curl_http_request_set_url(base: usize, interceptor: &mut Interceptor) {
    const FCURL_HTTP_REQUEST_SET_URL: usize = 0x1A5CE30;
    const CUSTOM_URL_PREFIX: &str = "http://127.0.0.1:10001";

    interceptor.attach(base + FCURL_HTTP_REQUEST_SET_URL, |ctx| {
        let f_string = ctx.registers().rdx as usize;
        let str_buf = unsafe {
            let length = *(f_string.wrapping_add(8) as *const u32) - 1;
            std::slice::from_raw_parts(*(f_string as *const usize) as *const u16, length as usize)
        };

        let url = String::from_utf16_lossy(str_buf);
        println!("FCurlHttpRequest::SetURL: {url}");

        let mut new_url = String::from(CUSTOM_URL_PREFIX);
        url.split('/').skip(3).for_each(|s| {
            new_url.push('/');
            new_url.push_str(s);
        });

        let buf = new_url.encode_utf16().chain([0]).collect::<Vec<_>>();

        unsafe {
            std::ptr::copy(
                buf.as_ptr(),
                (*(f_string as *const usize)) as *mut u16,
                buf.len(),
            );

            *(f_string.wrapping_add(8) as *mut u32) = new_url.len() as u32;
        }
    });
}
