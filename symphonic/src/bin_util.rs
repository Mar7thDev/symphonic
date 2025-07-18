use std::ffi::CStr;

pub trait BufExtensions {
    fn read_u16(&mut self) -> u16;
    fn read_u32(&mut self) -> u32;
    fn read_string(&mut self) -> String;
}

impl BufExtensions for &[u8] {
    fn read_u16(&mut self) -> u16 {
        let value = u16::from_le_bytes(self[0..2].try_into().unwrap());
        *self = &self[2..];

        value
    }

    fn read_u32(&mut self) -> u32 {
        let value = u32::from_le_bytes(self[0..4].try_into().unwrap());
        *self = &self[4..];

        value
    }

    fn read_string(&mut self) -> String {
        let cstr = CStr::from_bytes_until_nul(self).unwrap();
        let result = cstr.to_string_lossy();

        *self = &self[result.len() + 1..];
        result.to_string()
    }
}
