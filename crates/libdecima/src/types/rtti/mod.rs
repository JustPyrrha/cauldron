pub mod symbols;

use crate::assert_size;
use bitflags::bitflags;
use std::ffi::{c_char, c_void};
use std::fmt::{Display, Formatter};
use std::slice;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum RTTIKind {
    Atom,       // 0
    Pointer,    // 1
    Container,  // 2
    Enum,       // 3
    Compound,   // 4
    EnumFlags,  // 5
    POD,        // 6
    EnumBitSet, // 7
}

bitflags! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct RTTIFlags: u8 {
        const RTTIFactory_Registered = 0x2;
        const FactoryManager_Registered = 0x4;
    }
}

#[derive(Debug, Clone)]
#[repr(C, packed(1))]
pub struct RTTI {
    pub id: u32,
    pub kind: RTTIKind,
    pub factory_flags: RTTIFlags,
}
assert_size!(RTTI, 0x6);

#[derive(Debug, Clone)]
#[repr(C)]
pub struct RTTIAtom {
    pub base: RTTI,
    pub size: u16,
    pub alignment: u8,
    pub simple: u8,
    pub type_name: *const c_char,
    pub parent_type: *const RTTIAtom,
    pub fn_from_string: *const c_void,
    pub fn_to_string: *const c_void,
    pub unk30: *const c_void,
    pub fn_copy: *const c_void,
    pub fn_equals: *const c_void,
    pub fn_constructor: *const c_void,
    pub fn_destructor: *const c_void,
    pub fn_assign_with_endian: *const c_void,
    pub fn_assign: *const c_void,
    pub fn_get_size: *const c_void,
    pub fn_compare_strings: *const c_void,
    pub representation_type: *const RTTI,
}
assert_size!(RTTIAtom, 0x80);

#[derive(Debug, Clone)]
#[repr(C)]
pub struct RTTIValue {
    pub value: u32,
    pub name: *const c_char,
    pub aliases: [*const c_char; 4],
}
assert_size!(RTTIValue, 0x30);

#[derive(Debug, Clone)]
#[repr(C)]
pub struct RTTIEnum {
    pub base: RTTI,
    pub size: u8,
    pub alignment: u8,
    pub num_values: u16,
    pub type_name: *const c_char,
    pub values: *const RTTIValue,
    pub representation_type: *const RTTI,
}
assert_size!(RTTIEnum, 0x28);

#[derive(Debug, Clone)]
#[repr(C)]
pub struct RTTIBase {
    pub r#type: *const RTTICompound,
    pub offset: u32,
}
assert_size!(RTTIBase, 0x10);

#[derive(Debug, Clone)]
#[repr(C)]
pub struct RTTIAttr {
    pub r#type: *const RTTI,
    pub offset: u16,
    pub flags: u16,
    pub name: *const c_char,
    pub fn_getter: *const c_void,
    pub fn_setter: *const c_void,
    pub min_value: *const c_char,
    pub max_value: *const c_char,
}
assert_size!(RTTIAttr, 0x38);

#[derive(Debug, Clone)]
#[repr(C)]
pub struct RTTIOrderedAttr {
    pub base: RTTIAttr,
    pub parent: *const RTTICompound,
    pub group: *const c_char,
}
assert_size!(RTTIOrderedAttr, 0x48);

#[derive(Debug, Clone)]
#[repr(C)]
pub struct RTTIMessageHandler {
    pub message: *const RTTI,
    pub handler: *const c_void,
}
assert_size!(RTTIMessageHandler, 0x10);

#[derive(Debug, Clone)]
#[repr(C)]
pub struct RTTIMessageOrderEntry {
    pub before: u32,
    pub message: *const RTTI,
    pub compound: *const RTTI,
}
assert_size!(RTTIMessageOrderEntry, 0x18);

#[derive(Debug, Clone)]
#[repr(C)]
pub struct RTTICompound {
    pub base: RTTI,
    pub num_bases: u8,
    pub num_attrs: u8,
    pub num_message_handlers: u8,
    pub num_message_order_entries: u8,
    pub _pad: u8,
    pub version: u32,
    pub size: u32,
    pub alignment: u16,
    pub flags: u16,
    pub fn_constructor: *const c_void,
    pub fn_destructor: *const c_void,
    pub fn_from_string: *const c_void,
    pub _unk_30: *const c_void,
    pub fn_to_string: *const c_void,
    pub type_name: *const c_char,
    pub next_type: *const RTTI,
    pub prev_type: *const RTTI,
    pub bases: *const RTTIBase,
    pub attrs: *const RTTIAttr,
    pub message_handlers: *const RTTIMessageHandler,
    pub message_order_entries: *const RTTIMessageOrderEntry,
    pub fn_get_exported_symbols: *const c_void,
    pub representation_type: *const RTTI,
    pub ordered_attrs: *const RTTIOrderedAttr,
    pub num_ordered_attrs: u32,
    pub msg_read_binary: RTTIMessageHandler,
    pub msg_read_binary_offset: u32,
    pub _unk_ac: u32,
}
assert_size!(RTTICompound, 0xb0);

#[derive(Debug, Clone)]
#[repr(C)]
pub struct RTTIPointer {
    pub base: RTTI,
    pub item_type: *const RTTI,
    pub pointer_type: *const RTTIPointerData,
    pub type_name: *const c_char,
}
assert_size!(RTTIPointer, 0x20);

pub struct RTTIPointerData {
    pub type_name: *const c_char,
    pub size: u32,
    pub alignment: u8,
    pub fn_constructor: *const c_void,
    pub fn_destructor: *const c_void,
    pub fn_getter: *const c_void,
    pub fn_setter: *const c_void,
    pub fn_copier: *const c_void,
}
assert_size!(RTTIPointerData, 0x38);

#[derive(Debug, Clone)]
#[repr(C)]
pub struct RTTIContainer {
    pub base: RTTI,
    pub item_type: *const RTTI,
    pub container_type: *const RTTIContainerData,
    pub type_name: *const c_char,
}
assert_size!(RTTIContainer, 0x20);

#[derive(Debug, Clone)]
#[repr(C)]
pub struct RTTIContainerData {
    pub type_name: *const c_char,
    pub size: u16,
    pub alignment: u8,
    pub array: u8,
    pub fn_constructor: *const c_void,
    pub fn_destructor: *const c_void,
}
assert_size!(RTTIContainerData, 0x20);

#[derive(Debug, Clone)]
#[repr(C)]
pub struct RTTIPod {
    pub base: RTTI,
    pub size: u32,
    pub type_name: *const c_char,
}
assert_size!(RTTIPod, 0x18);

macro_rules! impl_inherits_rtti {
    ($n:ident, $t:ty, $($e:expr),+) => {
        pub unsafe fn $n(rtti: *const RTTI) -> Option<*const $t> { unsafe {
            if !rtti.is_null() && ($((*rtti).kind == $e)||+) {
                Some(std::mem::transmute(rtti))
            } else {
                None
            }
        }}
    };
}

impl_inherits_rtti!(as_atom, RTTIAtom, RTTIKind::Atom);
impl_inherits_rtti!(as_compound, RTTICompound, RTTIKind::Compound);
impl_inherits_rtti!(
    as_enum,
    RTTIEnum,
    RTTIKind::Enum,
    RTTIKind::EnumFlags,
    RTTIKind::EnumBitSet
);
impl_inherits_rtti!(as_pointer, RTTIPointer, RTTIKind::Pointer);
impl_inherits_rtti!(as_container, RTTIContainer, RTTIKind::Container);
impl_inherits_rtti!(as_pod, RTTIPod, RTTIKind::POD);

pub fn rtti_base_name(rtti: *const RTTI) -> String {
    cstr_to_string(unsafe {
        match (*rtti).kind {
            RTTIKind::Atom => (*as_atom(rtti).unwrap()).type_name,
            RTTIKind::Pointer => (*(*as_pointer(rtti).unwrap()).pointer_type).type_name,
            RTTIKind::Container => (*(*as_container(rtti).unwrap()).container_type).type_name,
            RTTIKind::Enum | RTTIKind::EnumFlags | RTTIKind::EnumBitSet => {
                (*as_enum(rtti).unwrap()).type_name
            }
            RTTIKind::Compound => (*as_compound(rtti).unwrap()).type_name,
            RTTIKind::POD => (*as_pod(rtti).unwrap()).type_name,
        }
    })
}

pub fn rtti_name(rtti: *const RTTI) -> String {
    unsafe {
        match (*rtti).kind {
            RTTIKind::Atom => cstr_to_string((*as_atom(rtti).unwrap()).type_name),
            RTTIKind::Pointer | RTTIKind::Container => {
                let container = std::mem::transmute::<*const RTTI, *const RTTIContainer>(rtti);
                format!(
                    "{}<{}>",
                    cstr_to_string((*container).type_name),
                    cstr_to_string((*(*container).container_type).type_name)
                )
                .to_string()
            }
            RTTIKind::Enum | RTTIKind::EnumFlags | RTTIKind::EnumBitSet => {
                cstr_to_string((*as_enum(rtti).unwrap()).type_name)
            }
            RTTIKind::Compound => cstr_to_string((*as_compound(rtti).unwrap()).type_name),
            RTTIKind::POD => cstr_to_string((*as_pod(rtti).unwrap()).type_name),
        }
    }
}

pub fn cstr_to_string(cstr: *const c_char) -> String {
    let cstr = unsafe { std::ffi::CStr::from_ptr(cstr) };
    cstr.to_string_lossy().into_owned()
}

impl Display for RTTIKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            RTTIKind::Atom => "atom",
            RTTIKind::Pointer => "pointer",
            RTTIKind::Container => "container",
            RTTIKind::Enum => "enum",
            RTTIKind::Compound => "compound",
            RTTIKind::EnumFlags => "enum flags",
            RTTIKind::POD => "pod",
            RTTIKind::EnumBitSet => "enum bitset",
        })
    }
}

impl RTTICompound {
    pub unsafe fn bases(&self) -> &[RTTIBase] { unsafe {
        if self.num_bases > 0 {
            slice::from_raw_parts(self.bases, self.num_bases as usize)
        } else {
            &[]
        }
    }}

    pub unsafe fn attributes(&self) -> &[RTTIAttr] { unsafe {
        if self.num_attrs > 0 {
            slice::from_raw_parts(self.attrs, self.num_attrs as usize)
        } else {
            &[]
        }
    }}

    pub unsafe fn message_handlers(&self) -> &[RTTIMessageHandler] { unsafe {
        if self.num_message_handlers > 0 {
            slice::from_raw_parts(self.message_handlers, self.num_message_handlers as usize)
        } else {
            &[]
        }
    }}
}

impl RTTIEnum {
    pub unsafe fn values(&self) -> &[RTTIValue] { unsafe {
        if self.num_values > 0 {
            slice::from_raw_parts(self.values, self.num_values as usize)
        } else {
            &[]
        }
    }}
}

impl RTTIValue {
    pub unsafe fn aliases(&self) -> Option<Vec<String>> {
        if self.aliases[0].is_null() {
            None
        } else {
            let mut aliases = Vec::new();
            for alias in &self.aliases {
                if !alias.is_null() {
                    aliases.push(cstr_to_string(*alias));
                }
            }

            Some(aliases)
        }
    }
}
