use std::ffi::{CStr, c_char};

pub trait ReadCStr {
    fn read_string(self) -> String;
    fn read_optional_string(self) -> Option<String>;
}

impl ReadCStr for *const c_char {
    fn read_string(self) -> String {
        let cstr = unsafe { CStr::from_ptr(self) };
        cstr.to_string_lossy().to_string()
    }

    fn read_optional_string(self) -> Option<String> {
        if self.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(self).to_string_lossy().to_string() })
        }
    }
}
