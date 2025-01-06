use serde::{Deserialize, Serialize};
use std::env::current_exe;
use std::ptr;
use windows::core::{w, HSTRING, PCWSTR};
use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW, VS_FIXEDFILEINFO,
};

#[derive(Debug, Copy, Clone)]
pub struct GameVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub build: u32,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum CauldronGameType {
    HorizonForbiddenWest,
    HorizonZeroDawn,
    HorizonZeroDawnRemastered,
}

impl CauldronGameType {
    pub fn id(&self) -> String {
        match self {
            CauldronGameType::HorizonForbiddenWest => String::from("hfw"),
            CauldronGameType::HorizonZeroDawn => String::from("hzd"),
            CauldronGameType::HorizonZeroDawnRemastered => String::from("hzdr"),
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "hfw" => Some(CauldronGameType::HorizonForbiddenWest),
            "hzd" => Some(CauldronGameType::HorizonZeroDawn),
            "hzdr" => Some(CauldronGameType::HorizonZeroDawnRemastered),
            _ => None,
        }
    }
}

impl CauldronGameType {
    pub fn find_from_exe() -> Option<Self> {
        match current_exe()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
        {
            "HorizonForbiddenWest.exe" => Some(Self::HorizonForbiddenWest),
            "HorizonZeroDawn.exe" => Some(Self::HorizonZeroDawn),
            "HorizonZeroDawnRemastered.exe" => Some(Self::HorizonZeroDawn),
            &_ => None,
        }
    }
}

pub fn version() -> GameVersion {
    let path = current_exe().unwrap();
    let mut version_info_size = unsafe {
        GetFileVersionInfoSizeW(
            PCWSTR::from_raw(HSTRING::from(path.as_path()).as_ptr()),
            None,
        )
    };
    let mut version_info_buf = vec![0u8; version_info_size as usize];
    unsafe {
        GetFileVersionInfoW(
            PCWSTR::from_raw(HSTRING::from(path.as_path()).as_ptr()),
            0,
            version_info_size,
            version_info_buf.as_mut_ptr() as _,
        )
        .unwrap()
    };

    let mut version_info: *mut VS_FIXEDFILEINFO = ptr::null_mut();
    unsafe {
        let _ = VerQueryValueW(
            version_info_buf.as_ptr() as _,
            w!("\\\\\0"),
            &mut version_info as *mut *mut _ as _,
            &mut version_info_size,
        );
    };
    let version_info = unsafe { version_info.as_ref().unwrap() };
    let major = (version_info.dwFileVersionMS >> 16) & 0xffff;
    let minor = (version_info.dwFileVersionMS) & 0xffff;
    let patch = (version_info.dwFileVersionLS >> 16) & 0xffff;
    let build = (version_info.dwFileVersionLS) & 0xffff;

    GameVersion {
        major,
        minor,
        patch,
        build,
    }
}
