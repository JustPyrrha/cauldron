use std::ffi::CStr;
use std::fs::File;
use windows::Win32::System::Diagnostics::Debug::{IMAGE_NT_HEADERS64, IMAGE_SECTION_HEADER};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::SystemServices::IMAGE_DOS_HEADER;

pub fn get_module() -> Option<(usize, usize)> {
    let base = unsafe { GetModuleHandleW(None).unwrap() };
    if base.0.is_null() {
        return None;
    }

    let base = base.0 as usize;
    let dos_header = unsafe { &*(base as *const IMAGE_DOS_HEADER) };
    let nt_headers_ptr =
        (base as isize).wrapping_add(dos_header.e_lfanew as isize) as *const IMAGE_NT_HEADERS64;
    let nt_headers = unsafe {
        if nt_headers_ptr.is_null() {
            return None;
        } else {
            &*nt_headers_ptr
        }
    };
    let end = base + nt_headers.OptionalHeader.SizeOfImage as usize;
    Some((base, end))
}

pub fn get_pe_section_range(module: usize, section_name: &str) -> Option<(usize, usize)> {
    let dos_header = unsafe { &*(module as *const IMAGE_DOS_HEADER) };
    let nt_headers_ptr =
        (module as isize).wrapping_add(dos_header.e_lfanew as isize) as *const IMAGE_NT_HEADERS64;

    if !nt_headers_ptr.is_null() {
        let nt_headers = unsafe { &*nt_headers_ptr };
        let header_size = size_of::<IMAGE_SECTION_HEADER>();
        let sections_base =
            (nt_headers_ptr as *const u8).wrapping_add(size_of::<IMAGE_NT_HEADERS64>());

        for i in 0..nt_headers.FileHeader.NumberOfSections {
            let section = unsafe {
                &*((sections_base as usize + ((i as usize) * header_size))
                    as *const IMAGE_SECTION_HEADER)
            };

            let section_name_str = std::str::from_utf8({
                let buf = &section.Name;
                let mut len = buf.len();
                while len > 0 {
                    if buf[len - 1] != 0 {
                        break;
                    }
                    len -= 1;
                }
                &buf[..len]
            })
            .unwrap_or("");

            if section_name_str == section_name {
                let start = module + section.VirtualAddress as usize;
                let end = unsafe {
                    module + section.VirtualAddress as usize + section.Misc.VirtualSize as usize
                };
                if start != 0 && end != 0 {
                    return Some((start, end));
                }
            }
        }
    }
    None
}

pub fn get_section(section_name: &str) -> Option<(usize, usize)> {
    let module = get_module()?.0;
    let (base, end) = get_pe_section_range(module, section_name)?;
    Some((base, end))
}

pub fn get_code_section() -> Option<(usize, usize)> {
    get_section(".text") // todo: cache
}

pub fn get_rdata_section() -> Option<(usize, usize)> {
    get_section(".rdata") // todo: cache
}

pub fn get_data_section() -> Option<(usize, usize)> {
    get_section(".data") // todo: cache
}
