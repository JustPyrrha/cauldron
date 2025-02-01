use std::ffi::{c_char, c_void};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Threading::CRITICAL_SECTION;

#[derive(Debug)]
#[repr(C)]
pub struct NxLogImplVftable {
    pub fn_destructor: extern "C" fn(),
    pub fn_constructor: extern "C" fn(),
    pub fn_unk_24: extern "C" fn(instance: *mut NxLogImpl),
    pub fn_alloc_console: extern "C" fn(instance: *mut NxLogImpl),
    pub fn_free_console: extern "C" fn(instance: *mut NxLogImpl),
    pub fn_unk_48: extern "C" fn(),
    pub fn_unk_56: extern "C" fn(),
    pub fn_print: extern "C" fn(),
    pub fn_println: extern "C" fn(),
    pub fn_log:
        extern "C" fn(instance: *mut NxLogImpl, category: *const c_char, format: *const c_char),
    pub fn_log_utf16: extern "C" fn(),
    pub fn_print_memory_stats: extern "C" fn(instance: *mut NxLogImpl, category: *const c_char),
    pub fn_print_unk_102: extern "C" fn(),
    pub fn_get_game_dir: extern "C" fn(),
}

#[derive(Debug)]
#[repr(C)]
pub struct NxLogImpl {
    pub vftable: *const NxLogImplVftable,
    pub initialized: bool,
    pub handle: *mut c_void,
    pub unk_18: i32,
    pub unk_1c: i32,
    pub unk_20: [u8; 1048576],
    pub log_path: [i32; 128], // [wchar_t;128]
    pub unk_100120: [u8; 256],
    pub console_handle: HANDLE,
    pub lock: CRITICAL_SECTION,
}
