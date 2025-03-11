use crate::{gen_with_vtbl, impl_instance};
use std::ffi::{c_char, c_void};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Threading::CRITICAL_SECTION;

gen_with_vtbl!(
    NxLogImpl,
    NxLogImplVtbl,

    fn fn_destructor();
    fn fn_constructor();
    fn fn_open_log();
    fn fn_alloc_console();
    fn fn_free_console();
    fn fn_unk_48();
    fn fn_get_log_path();
    fn fn_print(text: *const c_char);
    fn fn_println(text: *const c_char);
    fn fn_log(category: *const c_char, format: *const c_char); // also has a va_list as last for the format, but we're in rust, we can use format!()
    fn fn_log_w(category: *const c_char, format: *const c_char); // this also has the same va_list
    fn fn_log_memory_statistics(category: *const c_char);
    fn fn_unk_80();
    fn fn_unk_88();

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

impl_instance!(
    NxLogImpl,
    "48 8B 1D ? ? ? ? 48 8B 03 48 8B 78 48 48 8B 0D ? ? ? ?"
);
