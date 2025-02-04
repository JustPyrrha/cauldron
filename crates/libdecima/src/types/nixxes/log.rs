use crate::with_vftable;
use std::ffi::{c_char, c_void};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Threading::CRITICAL_SECTION;

with_vftable!(
    NxLogImpl,
    NxLogImplVtbl,

    fn fn_destructor(),
    fn fn_constructor(),
    fn fn_open_log(),
    fn fn_alloc_console(this: *mut NxLogImpl),
    fn fn_free_console(this: *mut NxLogImpl),
    fn fn_unk_48(),
    fn fn_get_log_path(),
    fn fn_print(this: *mut NxLogImpl, text: *const c_char),
    fn fn_println(this: *mut NxLogImpl, text: *const c_char),
    fn fn_log(this: *mut NxLogImpl, category: *const c_char, format: *const c_char), // also has a va_list as last for the format, but we're in rust, we can use format!()
    fn fn_log_w(this: *mut NxLogImpl, category: *const c_char, format: *const c_char), // this also has the same va_list
    fn fn_log_memory_statistics(this: *mut NxLogImpl, category: *const c_char),
    fn fn_unk_80(),
    fn fn_unk_88(),

    pub initialized: bool,
    pub handle: *mut c_void,
    pub unk_18: i32,
    pub unk_1c: i32,
    pub pad_20: [u8;0x10000],
    pub log_path: [u16;128], // [wchar_t;128]
    pub pad_100120: [u8;256],
    pub console_handle: HANDLE,
    pub lock: CRITICAL_SECTION,
);
