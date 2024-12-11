#![feature(decl_macro)]

use libc::c_void;
use once_cell::sync::OnceCell;
use paste::paste;
use std::{mem::transmute, path::PathBuf, thread};
use windows_sys::{
    core::{s, PCSTR, PCWSTR, PSTR, PWSTR},
    Win32::{
        Foundation::MAX_PATH,
        Storage::FileSystem::{
            GET_FILE_VERSION_INFO_FLAGS, VER_FIND_FILE_FLAGS, VER_FIND_FILE_STATUS,
        },
        System::{
            LibraryLoader::{GetProcAddress, LoadLibraryA},
            SystemInformation::GetSystemDirectoryA,
            SystemServices::DLL_PROCESS_ATTACH,
        },
    },
};

#[no_mangle]
extern "system" fn DllMain(_: isize, reason: u32, _: usize) -> bool {
    if reason == DLL_PROCESS_ATTACH {
        thread::spawn(inject_loader);
    }

    true
}

fn inject_loader() {
    unsafe {
        LoadLibraryA(s!("cauldron.dll"));
    };
}

macro __lazy_export($(fn $f:ident($($i:ident: $a:ty),*) -> $r:ty);+;) {
    #[inline]
    #[must_use]
    pub fn __h_version() -> isize {
        static VERSION: OnceCell<isize> = OnceCell::new();
        *VERSION.get_or_init(|| unsafe {
            let mut buffer = [0u8; MAX_PATH as usize];
            let buffer_len = GetSystemDirectoryA(buffer.as_mut_ptr(), buffer.len() as u32);
            assert_ne!(buffer_len, 0u32);

            let dir = PathBuf::from(String::from_utf8(buffer[..buffer_len as usize].to_vec()).unwrap()).join("version.dll");
            let dir = [dir.to_str().unwrap().as_bytes(), &[0u8]].concat();
            LoadLibraryA(dir.as_ptr()) as isize
        })
    }

    paste! {
        $(
            #[allow(clippy::many_single_char_names)]
            #[export_name = "" $f ""]
            unsafe extern "system" fn [<__ $f:snake>]($($i: $a),*) -> $r {
                static [<$f:snake:upper>]: OnceCell<usize> = OnceCell::new();

                unsafe {
                    transmute::<usize, unsafe extern "system" fn($($a),*) -> $r>(
                        *[<$f:snake:upper>].get_or_init(|| {
                            GetProcAddress(
                                __h_version() as *mut c_void,
                                format!("{}\0", stringify!($f)).as_ptr(),
                            )
                            .unwrap() as usize
                        }),
                    )($($i),*)
                }
            }
        )*
    }
}

#[rustfmt::skip]
__lazy_export! {
    fn GetFileVersionInfoA(a: PCSTR, b: u32, c: u32, d: *mut c_void) -> i32;
    fn GetFileVersionInfoExA(a: GET_FILE_VERSION_INFO_FLAGS, b: PCSTR, c: u32, d: u32, e: *mut c_void) -> i32;
    fn GetFileVersionInfoExW(a: GET_FILE_VERSION_INFO_FLAGS, b: PCWSTR, c: u32, d: u32, e: *mut c_void) -> i32;
    fn GetFileVersionInfoSizeA(a: PCSTR, b: *mut u32) -> u32;
    fn GetFileVersionInfoSizeExA(a: GET_FILE_VERSION_INFO_FLAGS, b: PCSTR, c: *mut u32) -> u32;
    fn GetFileVersionInfoSizeExW(a: GET_FILE_VERSION_INFO_FLAGS, b: PCWSTR, c: *mut u32) -> u32;
    fn GetFileVersionInfoSizeW(a: PCWSTR, b: *mut u32) -> u32;
    fn GetFileVersionInfoW(a: PCWSTR, b: u32, c: u32, d: *mut c_void) -> i32;
    fn VerFindFileA(a: VER_FIND_FILE_FLAGS, b: PCSTR, c: PCSTR, d: PCSTR, e: PSTR, f: *mut u32, g: PSTR, h: *mut u32) -> VER_FIND_FILE_STATUS;
    fn VerFindFileW(a: VER_FIND_FILE_FLAGS, b: PCWSTR, c: PCWSTR, d: PCWSTR, e: PWSTR, f: *mut u32, g: PWSTR, h: *mut u32) -> VER_FIND_FILE_STATUS;
    fn VerInstallFileA(a: VER_FIND_FILE_FLAGS, b: PCSTR, c: PCSTR, d: PCSTR, e: PSTR, f: PSTR, g: PSTR, h: *mut u32) -> VER_FIND_FILE_STATUS;
    fn VerInstallFileW(a: VER_FIND_FILE_FLAGS, b: PCWSTR, c: PCWSTR, d: PCWSTR, e: PWSTR, f: PWSTR, g: PWSTR, h: *mut u32) -> VER_FIND_FILE_STATUS;
    fn VerLanguageNameA(a: u32, b: PSTR, c: u32) -> u32;
    fn VerLanguageNameW(a: u32, b: PWSTR, c: u32) -> u32;
    fn VerQueryValueA(a: *const c_void, b: PCSTR, c: *mut *mut c_void, d: *mut u32) -> i32;
    fn VerQueryValueW(a: *const c_void, b: PCWSTR, c: *mut *mut c_void, d: *mut u32) -> i32;
}
