pub mod offsets;
pub mod scan;

use crate::log;
use crate::mem::offsets::Offset;
use std::ffi::c_void;
use std::ptr::read_unaligned;
use std::slice;
use windows::Win32::System::Diagnostics::Debug::{
    FlushInstructionCache, IMAGE_NT_HEADERS64, IMAGE_SECTION_HEADER,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::{
    PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS, VirtualProtect,
};
use windows::Win32::System::SystemServices::IMAGE_DOS_HEADER;
use windows::Win32::System::Threading::GetCurrentProcess;

#[derive(Debug, Clone)]
pub enum PatternSearchError {
    ParseInt(std::num::ParseIntError),
    OutOfRange,
    NotFound,
}

/// parses an ida-style byte sequence pattern
pub fn parse_pattern(mask: &str) -> Result<Vec<(u8, bool)>, PatternSearchError> {
    let mask = mask.replace("?", "??");
    let mask = mask.replace(" ", "");

    (0..mask.len())
        .step_by(2)
        .map(|i| {
            let radix = &mask[i..i + 2];
            if radix == "??" {
                Ok((0x00, true))
            } else {
                Ok((
                    u8::from_str_radix(radix, 16).unwrap(), /*? todo */
                    false,
                ))
            }
        })
        .collect()
}

pub fn find_pattern(
    start_address: *mut u8,
    max_size: usize,
    mask: &str,
) -> Result<*mut u8, PatternSearchError> {
    let pattern = parse_pattern(mask)?;
    let data_end = start_address as usize + max_size + 1;

    let result = unsafe { slice::from_raw_parts(start_address, max_size + 1) }
        .windows(pattern.len())
        .position(|pos| {
            pos.iter()
                .enumerate()
                .all(|(i, b)| pattern[i].1 || pattern[i].0.eq(b))
        });

    let Some(result) = result else {
        return Err(PatternSearchError::NotFound);
    };

    if result > data_end {
        return Err(PatternSearchError::OutOfRange);
    }

    Ok((start_address as usize + result) as *mut u8)
}

pub fn offset_from_instruction(
    signature: &str,
    rip_rel_add: u32,
) -> Result<*const u8, PatternSearchError> {
    let (module_base, module_end) = get_module()?;
    let addr = find_pattern(module_base as *mut u8, module_end - module_base, signature)?;
    let rel_offset = unsafe {
        let ptr = addr.add(rip_rel_add as usize) as *const i32;
        read_unaligned(ptr) + size_of::<i32>() as i32
    };
    Ok(unsafe {
        addr.add(rip_rel_add as usize)
            .add(rel_offset as usize)
            .sub(module_base)
    })
}

pub fn get_module() -> Result<(usize, usize), PatternSearchError> {
    let base = unsafe { GetModuleHandleW(None).unwrap() };
    if base.0.is_null() {
        return Err(PatternSearchError::OutOfRange);
    }

    let base = base.0 as usize;
    let dos_header = unsafe { &*(base as *const IMAGE_DOS_HEADER) };
    let nt_headers_ptr =
        (base as isize).wrapping_add(dos_header.e_lfanew as isize) as *const IMAGE_NT_HEADERS64;
    let nt_headers = unsafe {
        if nt_headers_ptr.is_null() {
            return Err(PatternSearchError::OutOfRange);
        } else {
            &*nt_headers_ptr
        }
    };
    let end = base + nt_headers.OptionalHeader.SizeOfImage as usize;
    Ok((base, end))
}

pub fn get_pe_section_range(
    module: usize,
    section_name: &str,
) -> Result<(usize, usize), PatternSearchError> {
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
                    return Ok((start, end));
                }
            }
        }
    }
    Err(PatternSearchError::OutOfRange)
}

pub fn get_section(section_name: &str) -> Result<(usize, usize), PatternSearchError> {
    let module = get_module()?.0;
    let (base, end) = get_pe_section_range(module, section_name)?;
    Ok((base, end))
}

pub fn get_code_section() -> Result<(usize, usize), PatternSearchError> {
    get_section(".text")
}

pub fn get_rdata_section() -> Result<(usize, usize), PatternSearchError> {
    get_section(".rdata")
}

pub fn get_data_section() -> Result<(usize, usize), PatternSearchError> {
    get_section(".data")
}

pub fn patch(ptr: *mut c_void, data: &[u8]) {
    if !ptr.is_null() {
        unsafe {
            let mut flags = PAGE_PROTECTION_FLAGS::default();
            VirtualProtect(ptr, data.len(), PAGE_EXECUTE_READWRITE, &mut flags).unwrap();
            for (index, byte) in data.iter().enumerate() {
                std::ptr::write_bytes(ptr.add(index * size_of::<u8>()), *byte, 1);
            }
            VirtualProtect(ptr, data.len(), flags, &mut flags).unwrap();
            FlushInstructionCache(GetCurrentProcess(), Some(ptr as *const _), data.len()).unwrap();
        }
    }
}

/// Patches out the Crash Logger and Telemetry Logger
pub fn patch_reporting_loggers() {
    #[cfg(feature = "hfw")]
    {
        // patches by Nukem9

        // disable crash logger
        patch(
            Offset::from_signature("40 53 48 83 EC 20 80 79 38 00 48 8B D9 75 4C")
                .unwrap()
                .as_ptr::<c_void>(),
            &[
                0xc3, // ret
            ],
        );

        // disable telemetry logger
        patch(
            Offset::from_signature("E8 ? ? ? ? 0F B6 F8 47 38 ? ? 75 05 E8")
                .unwrap()
                .as_ptr::<c_void>(),
            &[
                0xB0, 0x01, // mov al,0x1
                0x90, // nop
                0x90, // nop
                0x90, // nop
            ],
        );
        log!("Disabled (patched) crash and telemetry loggers.");
    }
}
