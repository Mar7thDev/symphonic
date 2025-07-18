use std::ffi::CString;

use windows::{Win32::System::LibraryLoader::GetModuleHandleA, core::PSTR};

pub fn get_module_base(name: Option<&str>) -> usize {
    unsafe {
        GetModuleHandleA(match name {
            Some(name) => {
                let c_name = CString::new(name).unwrap().to_bytes_with_nul().as_ptr();
                PSTR::from_raw(c_name as *mut _)
            }
            None => PSTR::null(),
        })
        .unwrap()
        .0 as usize
    }
}

pub fn get_executable_entry_point_offset(executable_base: usize) -> usize {
    use windows::Win32::System::{Diagnostics::Debug::*, SystemServices::*};

    unsafe {
        let dos_header = executable_base as *const IMAGE_DOS_HEADER;
        let nt_header = executable_base.wrapping_add((*dos_header).e_lfanew as usize)
            as *const IMAGE_NT_HEADERS64;

        (*nt_header).OptionalHeader.AddressOfEntryPoint as usize
    }
}
