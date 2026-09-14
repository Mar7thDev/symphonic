use std::{fs::{File, OpenOptions}, io::Write, sync::{Mutex, OnceLock}, time::{SystemTime, UNIX_EPOCH}};
use windows::Win32::System::{Diagnostics::Debug::{AddVectoredExceptionHandler, EXCEPTION_POINTERS}, Threading::{GetCurrentProcessId, GetCurrentThreadId}};

static LOG: OnceLock<Mutex<File>> = OnceLock::new();

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {{
        $crate::logging::write(format_args!($($arg)*));
        println!($($arg)*);
    }};
}

pub fn init() {
    let path = std::env::current_exe().unwrap().with_file_name("symphonic.log");
    let file = OpenOptions::new().create(true).append(true).open(path).unwrap();
    let _ = LOG.set(Mutex::new(file));
    write(format_args!("attach pid={}", unsafe { GetCurrentProcessId() }));
    std::panic::set_hook(Box::new(|info| write(format_args!("PANIC {info}"))));
    unsafe { AddVectoredExceptionHandler(1, Some(exception)); }
}

pub fn write(args: std::fmt::Arguments<'_>) {
    if let Some(log) = LOG.get() {
        if let Ok(mut file) = log.lock() {
            let millis = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
            let _ = writeln!(file, "{millis} tid={} {args}", unsafe { GetCurrentThreadId() });
            let _ = file.flush();
        }
    }
}

unsafe extern "system" fn exception(info: *mut EXCEPTION_POINTERS) -> i32 {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    if matches!(unsafe { (*(*info).ExceptionRecord).ExceptionCode.0 as u32 }, 0x406d1388 | 0x40010006 | 0x4001000a) { return 0; }
    if COUNT.fetch_add(1, Ordering::Relaxed) < 16 {
        unsafe {
            let record = &*(*info).ExceptionRecord;
            let ctx = &*(*info).ContextRecord;
            write(format_args!("exception code={:#x} address={:?} rip={:#x} rsp={:#x} rcx={:#x} rdx={:#x} data={:x?}", record.ExceptionCode.0, record.ExceptionAddress, ctx.Rip, ctx.Rsp, ctx.Rcx, ctx.Rdx, &record.ExceptionInformation[..record.NumberParameters.min(15) as usize]));
        }
    }
    0
}
