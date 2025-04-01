use crate::assert_size;
use crate::mem::offsets::Offset;
use std::ffi::c_char;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct StringData {
    pub ref_count: u32,
    pub crc: u32,
    pub length: u32,
    pub _pad: u32,
}
assert_size!(StringData, 0x10);
impl StringData {
    pub const INVALID_CRC: u32 = 0xFFFFFFFF;
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct String {
    pub data: *const c_char,
}

impl String {
    fn internal_init(&self, data: *const c_char, size: usize) {
        let func = unsafe {
            *Offset::from_signature("48 89 5C 24 08 48 89 6C 24 10 48 89 74 24 18 57 48 83 EC 20 48 8B 01 48 8B EA 49 63 F8 48 8B F1 45 85 C0")
            .unwrap()
            .as_ptr::<extern "C" fn(*mut String, *const c_char, usize /* size_t */)>()
        };

        func(self as *const Self as *mut Self, data, size);
    }

    fn internal_data(&self) -> &StringData {
        unsafe {
            &*std::mem::transmute::<isize, *mut StringData>(
                std::mem::transmute::<*const c_char, isize>(self.data)
                    - size_of::<StringData>() as isize,
            )
        }
    }

    pub fn length(&self) -> u32 {
        self.internal_data().length
    }

    pub fn new() -> Self {
        let string = Self {
            data: std::ptr::null(),
        };
        string.internal_init(std::ptr::null(), 0);
        string
    }

    pub fn as_string(&self) -> std::string::String {
        let cstr = unsafe { std::ffi::CStr::from_ptr(self.data) };
        cstr.to_string_lossy().into_owned()
    }
}

impl Drop for String {
    fn drop(&mut self) {
        let func = Offset::from_signature(
            "40 53 48 83 EC 20 48 8B 19 48 8D 05 ? ? ? ? 48 83 EB 10 48 3B D8",
        )
        .unwrap()
        .as_ptr::<extern "C" fn(*mut String)>();
        unsafe { (*func)(self as *const Self as *mut Self) };
    }
}
