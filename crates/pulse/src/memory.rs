use pattern16::Pat16_scan;
use std::ffi::CString;
use std::os::raw::c_void;
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

pub fn find_pattern(start: *const u8, end: usize, signature: &str) -> Option<*const c_void> {
    let cstr = CString::new(signature).unwrap();
    let result = unsafe { Pat16_scan(start as *const c_void, end, cstr.as_ptr()) };
    if result.is_null() {
        None
    } else {
        Some(result)
    }
}

pub unsafe fn get_memory_at(addr: &*const u8, size: usize) -> &[u8] {
    std::slice::from_raw_parts(*addr, size)
}

pub fn find_offset_from(sig: &str, add: u32) -> Option<usize> {
    let (module_start, module_end) = get_module()?;
    let addr = find_pattern(module_start as *const u8, module_end, sig)?;
    if addr.is_null() {
        return None;
    }

    let offset = (addr as u32) + add + size_of::<i32>() as u32;
    Some((addr as u32 + add + offset - module_start as u32) as usize)
}
