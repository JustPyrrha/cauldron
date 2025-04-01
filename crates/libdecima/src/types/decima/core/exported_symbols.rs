use crate::mem::offsets::Offset;
use crate::types::decima::core::rtti::RTTI;
use crate::types::decima::p_core::prelude::{Array, HashMap};
use crate::{assert_size, gen_with_vtbl};
use bitflags::bitflags;
use std::ffi::{CStr, c_char, c_void};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub enum ExportedSymbolKind {
    Atom,
    Enum,
    Class,
    Struct,
    Typedef,
    Function,
    Variable,
    Container,
    Reference,
    Pointer,
    SourceFile,
}
assert_size!(ExportedSymbolKind, 0x1);

impl Display for ExportedSymbolKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ExportedSymbolKind::Atom => "Atom",
            ExportedSymbolKind::Enum => "Enum",
            ExportedSymbolKind::Class => "Class",
            ExportedSymbolKind::Struct => "Struct",
            ExportedSymbolKind::Typedef => "Typedef",
            ExportedSymbolKind::Function => "Function",
            ExportedSymbolKind::Variable => "Variable",
            ExportedSymbolKind::Container => "Container",
            ExportedSymbolKind::Reference => "Reference",
            ExportedSymbolKind::Pointer => "Pointer",
            ExportedSymbolKind::SourceFile => "SourceFile",
        })
    }
}

bitflags! {
    #[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
    #[repr(C)]
    pub struct ExportedSymbolSignatureFlags : u8 {
        const Unk1 = 1;
        const Unk2 = 2;
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct ExportedSymbolSignature {
    pub type_name: *const c_char,
    pub modifiers: *const c_char,
    pub r#type: *mut RTTI,
    pub name: *const c_char,
    pub flags: ExportedSymbolSignatureFlags,
}
assert_size!(ExportedSymbolSignature, 0x28);

#[derive(Debug, Clone)]
#[repr(C)]
pub struct ExportedSymbolLanguage {
    pub address: *mut c_void,
    pub name: *const c_char,
    pub header_file: *const c_char,
    pub source_file: *const c_char,
    pub signature: Array<ExportedSymbolSignature>,
    pub fn_unk30: *mut c_void,
    pub fn_unk38: *mut c_void,
}
assert_size!(ExportedSymbolLanguage, 0x40);

bitflags! {
    #[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
    #[repr(C)]
    pub struct ExportedSymbolFlags : u8 {
        const Unk1 = 1;
        const Unk2 = 2;
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct ExportedSymbol {
    pub kind: ExportedSymbolKind,
    pub r#type: *const RTTI,
    pub namespace: *const c_char,
    pub name: *const c_char,
    pub size_type: *const RTTI,
    pub flags: ExportedSymbolFlags,
    pub language: [ExportedSymbolLanguage; 2],
}
assert_size!(ExportedSymbol, 0xB0);

impl ExportedSymbol {
    pub fn namespace(&self) -> Option<String> {
        unsafe {
            if self.namespace.is_null() {
                None
            } else {
                Some(CStr::from_ptr(self.namespace).to_string_lossy().to_string())
            }
        }
    }

    pub fn name(&self) -> Option<String> {
        unsafe {
            if self.name.is_null() {
                None
            } else {
                Some(CStr::from_ptr(self.name).to_string_lossy().to_string())
            }
        }
    }
}

gen_with_vtbl!(
    ExportedSymbolsGroup,
    ExportedSymbolsGroupVtbl,

    fn constructor();
    fn register_symbols();

    pub export_mask: u32,
    pub namespace: *const c_char,
    pub symbols: Array<ExportedSymbol>,
    pub dependencies: Array<*const RTTI>,
);
assert_size!(ExportedSymbolsGroup, 0x38);

#[derive(Debug)]
#[repr(C)]
pub struct ExportedSymbols {
    pub groups: Array<*mut ExportedSymbolsGroup>,
    pub dependencies1: Array<*const RTTI>,
    pub dependencies2: Array<*const RTTI>,
    pub all_symbols: HashMap<*mut ExportedSymbol, u32>,
    pub type_symbols: HashMap<crate::types::decima::p_core::prelude::String, *mut ExportedSymbol>,
}

impl ExportedSymbols {
    pub fn get() -> Option<&'static ExportedSymbols> {
        let ptr = Offset::from_signature("48 63 05 ? ? ? ? 4D 8B 3E")
            .unwrap()
            .as_relative(7)
            .as_ptr::<ExportedSymbols>();
        if !ptr.is_null() {
            let instance = unsafe { &*ptr };
            return Some(instance);
        }
        None
    }
}
