use std::{
    collections::HashMap,
    ffi::{CString, c_void},
};

use windows::{
    Win32::System::{
        LibraryLoader::{GetModuleHandleA, GetProcAddress, LoadLibraryA},
        Memory::{PAGE_EXECUTE_READWRITE, PAGE_READWRITE, VirtualProtect},
    },
    core::{PSTR, s},
};

use crate::{bin_util::BufExtensions, nt_util};
use crate::logging;

pub fn restore_executable_from_dump(mut dump: &[u8]) {
    restore_sections(&mut dump);
    restore_imports(&mut dump);
    restore_entry_point(&mut dump);
}

fn restore_entry_point(dump: &mut &[u8]) {
    // Just put a relative jump instruction at fake entry point to real one

    let base = nt_util::get_module_base(None);

    let real_entry_point = dump.read_u32() as usize;
    let fake_entry_point = nt_util::get_executable_entry_point_offset(base);
    logging::write(format_args!("entry redirect base={base:#x} fake={fake_entry_point:#x} real={real_entry_point:#x}"));

    let jmp_diff = i32::try_from(real_entry_point as i64 - fake_entry_point as i64 - 5)
        .expect("entry point redirect exceeds rel32 range");

    unsafe {
        let ptr = (base + fake_entry_point) as *const c_void;
        let mut prot = PAGE_EXECUTE_READWRITE;

        VirtualProtect(ptr, 5, prot, &mut prot).unwrap();
        *(ptr as *mut u8) = 0xE9; // JMP
        std::slice::from_raw_parts_mut((base + fake_entry_point + 1) as *mut u8, 4)
            .copy_from_slice(&jmp_diff.to_le_bytes());

        VirtualProtect(ptr, 5, prot, &mut prot).unwrap();
        windows::Win32::System::Diagnostics::Debug::FlushInstructionCache(
            windows::Win32::System::Threading::GetCurrentProcess(), Some(ptr), 5).unwrap();
    }
}

fn restore_imports(dump: &mut &[u8]) {
    let fallback_imports = HashMap::from([
        (
            "NtdllDefWindowProc_A",
            (s!("user32.dll"), s!("DefWindowProcA")),
        ),
        (
            "NtdllDefWindowProc_W",
            (s!("user32.dll"), s!("DefWindowProcW")),
        ),
        (
            "RtlGetCurrentProcessorNumber",
            (s!("ntdll.dll"), s!("NtGetCurrentProcessorNumber")),
        ),
    ]);

    let base = nt_util::get_module_base(None);

    let module_count = dump.read_u16();
    crate::trace!("modules to import: {module_count}");

    for _ in 0..module_count {
        let module_name = dump.read_string();
        let import_count = dump.read_u16();

        crate::trace!("{module_name}: {import_count} symbols to import");

        let module = unsafe {
            let name = CString::new(module_name.clone()).unwrap();
            LoadLibraryA(PSTR(name.as_bytes_with_nul().as_ptr() as *mut _)).unwrap_or_else(|err| {
                crate::trace!("failed to load library: {err}");
                hang!();
            })
        };

        for _ in 0..import_count {
            let symbol_name = dump.read_string();
            let symbol_offset = dump.read_u32() as usize;

            crate::trace!("named import: {module_name}::{symbol_name} to 0x{symbol_offset:X}");

            let proc_address = unsafe {
                let c_name = CString::new(symbol_name.clone()).unwrap();
                if let Some(address) =
                    GetProcAddress(module, PSTR(c_name.as_bytes_with_nul().as_ptr() as *mut _))
                {
                    address
                } else if let Some(&(module, symbol)) = fallback_imports.get(&*symbol_name) {
                    GetProcAddress(GetModuleHandleA(module).unwrap(), symbol).unwrap()
                } else {
                    crate::trace!("import failed");
                    hang!();
                }
            } as usize;

            unsafe {
                let ptr = (base + symbol_offset) as *const c_void;
                let mut prot = PAGE_READWRITE;

                VirtualProtect(ptr, 8, prot, &mut prot).unwrap();
                *(ptr as *mut usize) = proc_address;
                VirtualProtect(ptr, 8, prot, &mut prot).unwrap();
            }
        }
    }
}

fn restore_sections(dump: &mut &[u8]) {
    let section_count = dump.read_u16();
    crate::trace!("sections in dump: {section_count}");

    for _ in 0..section_count {
        let offset = dump.read_u32() as usize;
        let size = dump.read_u32() as usize;
        let payload = &dump[..size];

        restore_section(offset, payload);
        *dump = &dump[size..];
    }
}

fn restore_section(offset: usize, payload: &[u8]) {
    crate::trace!(
        "restoring section at offset 0x{offset:X} of size 0x{size:X}",
        size = payload.len()
    );

    let base = nt_util::get_module_base(None);

    unsafe {
        let mut prot = PAGE_EXECUTE_READWRITE;
        let ptr = (base + offset) as *const c_void;

        VirtualProtect(ptr, payload.len(), prot, &mut prot).unwrap();
        std::slice::from_raw_parts_mut(ptr as *mut u8, payload.len()).copy_from_slice(payload);
        VirtualProtect(ptr, payload.len(), prot, &mut prot).unwrap();
        windows::Win32::System::Diagnostics::Debug::FlushInstructionCache(
            windows::Win32::System::Threading::GetCurrentProcess(), Some(ptr), payload.len()).unwrap();
    }
}

macro_rules! hang {
    () => {
        ::std::thread::sleep(::std::time::Duration::from_secs(u64::MAX));
        unreachable!()
    };
}

use hang;
