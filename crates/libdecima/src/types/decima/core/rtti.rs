use super::super::p_core::prelude::*;
use crate::assert_size;
use crate::mem::offsets::Offset;
use crate::types::decima::core::rtti_object::RTTIObject;
use bitflags::bitflags;
use std::ffi::{CStr, c_char, c_void};
use std::slice;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

unsafe impl Send for RTTIKind {}
unsafe impl Sync for RTTIKind {}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct RTTIFlags: u8 {
        const RTTIFactory_Registered = 0x2;
        const FactoryManager_Registered = 0x4;
    }
}
unsafe impl Send for RTTIFlags {}
unsafe impl Sync for RTTIFlags {}

pub trait SerializableRTTI {
    fn serialize(&self, object: *const c_void) -> Option<std::string::String>;
    fn deserialize(&self, object: *mut c_void, value: &mut String) -> bool;
}

pub trait NamedRTTI {
    fn name(&self) -> std::string::String;
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed(1))]
pub struct RTTI {
    pub id: u32,
    pub kind: RTTIKind,
    pub factory_flags: RTTIFlags,
}
assert_size!(RTTI, 0x6);

unsafe impl Send for RTTI {}
unsafe impl Sync for RTTI {}

pub type FnRTTIConstructor = *mut extern "C" fn(*const RTTI, *mut c_void) -> *mut c_void;
pub type FnRTTIDestructor = *mut extern "C" fn(*const RTTI, *mut c_void) -> *mut c_void;
pub type FnRTTIFromString = *mut extern "C" fn(*mut c_void, &mut String) -> bool;
pub type FnRTTIToString = *mut extern "C" fn(*const c_void, &mut String) -> bool;

impl SerializableRTTI for RTTI {
    fn serialize(&self, object: *const c_void) -> Option<std::string::String> {
        unsafe {
            match self.kind {
                RTTIKind::Atom => (&*(self as *const Self as *const RTTIAtom)).serialize(object),
                RTTIKind::Pointer | RTTIKind::Container => {
                    (&*(self as *const Self as *const RTTIContainer)).serialize(object)
                }
                _ => None,
            }
        }
    }

    fn deserialize(&self, object: *mut c_void, value: &mut String) -> bool {
        unsafe {
            match self.kind {
                RTTIKind::Atom => {
                    (&*(self as *const Self as *const RTTIAtom)).deserialize(object, value)
                }
                RTTIKind::Pointer | RTTIKind::Container => {
                    (&*(self as *const Self as *const RTTIContainer)).deserialize(object, value)
                }
                _ => false,
            }
        }
    }
}

impl RTTI {
    pub fn is_kind(&self, other: &RTTI) -> bool {
        if self.is_exact_kind(other) {
            return true;
        }

        if self.kind != RTTIKind::Compound && other.kind != RTTIKind::Compound {
            return self.is_exact_kind(other);
        }

        for base in self.as_compound().unwrap().bases() {
            let base_type = base.r#type;
            if base_type == self as *const Self as *mut Self {
                return true;
            }
        }

        false
    }

    pub fn is_exact_kind(&self, other: &RTTI) -> bool {
        self as *const Self == other as *const RTTI
    }

    pub fn as_atom(&self) -> Option<&RTTIAtom> {
        if self.kind != RTTIKind::Atom {
            None
        } else {
            Some(unsafe { &*(self as *const RTTI as *const RTTIAtom) })
        }
    }

    pub fn as_container(&self) -> Option<&RTTIContainer> {
        if self.kind != RTTIKind::Container && self.kind != RTTIKind::Pointer {
            None
        } else {
            Some(unsafe { &*(self as *const RTTI as *const RTTIContainer) })
        }
    }

    pub fn as_enum(&self) -> Option<&RTTIEnum> {
        if self.kind != RTTIKind::Enum && self.kind != RTTIKind::EnumFlags {
            None
        } else {
            Some(unsafe { &*(self as *const RTTI as *const RTTIEnum) })
        }
    }

    pub fn as_compound(&self) -> Option<&RTTICompound> {
        if self.kind != RTTIKind::Compound {
            None
        } else {
            Some(unsafe { &*(self as *const RTTI as *const RTTICompound) })
        }
    }

    pub fn as_pod(&self) -> Option<&RTTIPod> {
        if self.kind != RTTIKind::POD {
            None
        } else {
            Some(unsafe { &*(self as *const RTTI as *const RTTIPod) })
        }
    }

    pub fn as_bitset(&self) -> Option<&RTTIEnumBitSet> {
        if self.kind != RTTIKind::EnumBitSet {
            None
        } else {
            Some(unsafe { &*(self as *const RTTI as *const RTTIEnumBitSet) })
        }
    }

    pub fn get_contained_type(&self) -> &RTTI {
        match self.kind {
            RTTIKind::Pointer | RTTIKind::Container => unsafe {
                &*(&*(self as *const RTTI as *const RTTIContainer)).item_type
            },
            _ => self,
        }
    }

    pub fn get_symbol_name(&self) -> std::string::String {
        match self.kind {
            RTTIKind::Atom => self.as_atom().unwrap().name(),
            RTTIKind::Pointer | RTTIKind::Container => self.as_container().unwrap().name(),
            RTTIKind::EnumFlags | RTTIKind::Enum => self.as_enum().unwrap().name(),
            RTTIKind::Compound => self.as_compound().unwrap().name(),
            RTTIKind::POD => self.as_pod().unwrap().name(),
            RTTIKind::EnumBitSet => self.as_bitset().unwrap().name(),
        }
    }

    pub fn get_core_binary_type_id(&self) -> u64 {
        let func = unsafe {
            &*Offset::from_signature("40 57 48 81 EC 30 08 00 00 0F B6 41 04 4C 8D")
                .unwrap()
                .as_ptr::<extern "C" fn(*const RTTI) -> u64>()
        };

        func(self as *const Self)
    }

    pub fn create_instance(&self) -> *mut c_void {
        let func = unsafe {
            &*Offset::from_signature(
                "48 89 6C 24 10 48 89 74 24 18 57 48 83 EC 20 48 8B F9 E8 ? ? ? ? 80 79 04 04",
            )
            .unwrap()
            .as_ptr::<extern "C" fn(*const RTTI) -> *mut c_void>()
        };

        func(self as *const Self)
    }

    pub fn find_by_name<'a>(name: &str) -> &'a RTTI {
        let func = Offset::from_signature(
            "48 83 EC 38 48 85 C9 74 37 48 89 4C 24 20 48 C7 C0 FF FF FF FF",
        )
        .unwrap()
        .as_ptr::<extern "C" fn(*const c_char) -> *const RTTI>();

        unsafe { &*(&*func)(name.as_ptr() as *const c_char) }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct RTTIStringSpan {
    pub data: *const c_char,
    pub length: u32,
}

impl Default for RTTIStringSpan {
    fn default() -> Self {
        RTTIStringSpan {
            data: std::ptr::null(),
            length: 0,
        }
    }
}

impl RTTIStringSpan {
    pub fn new(value: &String) -> Self {
        RTTIStringSpan {
            data: value.data,
            length: value.length(),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct RTTIIter {
    pub container_type: *mut RTTI,
    pub container: *mut c_void,
    pub user_data: u32,
}
assert_size!(RTTIIter, 0x18);

#[derive(Debug)]
#[repr(C)]
pub struct RTTIAtom {
    pub base: RTTI,
    pub size: u16,
    pub alignment: u8,
    pub simple: bool,
    pub type_name: *const c_char,
    pub base_type: *const RTTIAtom,
    pub fn_from_string: *mut extern "C" fn(value: &RTTIStringSpan, out: *const c_void) -> bool,
    pub fn_to_string: FnRTTIToString,
    pub fn_unk30: *const c_void,
    pub fn_copy: *mut extern "C" fn(r#in: *mut c_void, out: *const c_void),
    pub fn_equals: *mut extern "C" fn(*const c_void, *const c_void) -> bool,
    pub fn_constructor: FnRTTIConstructor,
    pub fn_destructor: FnRTTIDestructor,
    pub fn_serialize:
        *mut extern "C" fn(r#in: *mut c_void, out: *mut c_void, swap_endian: bool) -> bool,
    pub fn_deserialize: *mut extern "C" fn(r#in: *mut c_void, out: *mut c_void) -> bool,
    pub fn_get_serialize_size: *mut extern "C" fn(*const c_void) -> i32,
    pub fn_range_check: *mut extern "C" fn(*const c_void, *const c_char, *const c_char),
    pub representation_type: *const RTTI,
}

impl NamedRTTI for RTTIAtom {
    fn name(&self) -> std::string::String {
        if self.type_name.is_null() {
            std::string::String::new()
        } else {
            unsafe { CStr::from_ptr(self.type_name).to_string_lossy().to_string() }
        }
    }
}

impl SerializableRTTI for RTTIAtom {
    fn serialize(&self, object: *const c_void) -> Option<std::string::String> {
        if self.fn_to_string.is_null() {
            None
        } else {
            let mut str = String::new();
            let res = unsafe { (&*self.fn_to_string)(object, &mut str) };
            if res {
                let cstr = unsafe { CStr::from_ptr(str.data) };
                Some(cstr.to_string_lossy().to_string())
            } else {
                None
            }
        }
    }

    fn deserialize(&self, object: *mut c_void, value: &mut String) -> bool {
        !self.fn_from_string.is_null()
            && unsafe { (&*self.fn_from_string)(&RTTIStringSpan::new(value), object) }
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct RTTIPointerData {
    pub base: RTTIContainerTypeBase,

    pub size: u16,
    pub alignment: u8,

    pub fn_constructor: FnRTTIConstructor,
    pub fn_destructor: FnRTTIDestructor,
    pub fn_get: *mut extern "C" fn(*const RTTI, *const c_void) -> *mut c_void,
    pub fn_set: *mut extern "C" fn(*const RTTI, *mut *mut c_void, *mut c_void) -> bool,
    pub fn_copy: *mut extern "C" fn(*mut *mut c_void, *mut *mut c_void),
}

#[derive(Debug)]
#[repr(C)]
pub struct RTTIContainerData {
    pub base: RTTIContainerTypeBase,

    pub size: u16,
    pub alignment: u8,
    pub is_simple: bool,
    pub is_associative: bool,

    pub fn_constructor: FnRTTIConstructor,
    pub fn_destructor: FnRTTIDestructor,
    pub fn_resize: *mut extern "C" fn(*const RTTI, *mut c_void, new_size: i32, bool) -> bool,
    _pad0: [u8; 0x8],
    pub fn_remove: *mut extern "C" fn(*const RTTI, *mut c_void, index: i32) -> bool,
    pub fn_len: *mut extern "C" fn(*const RTTI, *const c_void) -> i32,
    pub fn_iter_start: *mut extern "C" fn(*const RTTI, *const c_void) -> RTTIIter,
    pub fn_iter_end: *mut extern "C" fn(*const RTTI, *const c_void) -> RTTIIter,
    pub fn_iter_next: *mut extern "C" fn(&RTTIIter),
    pub fn_iter_deref: *mut extern "C" fn(&RTTIIter) -> *mut c_void,
    pub fn_is_iter_valid: *mut extern "C" fn(&RTTIIter) -> bool,
    _pad1: [u8; 0x10],
    pub fn_add_item: *mut extern "C" fn(*const RTTI, *mut c_void, *mut c_void) -> RTTIIter,
    pub fn_add_empty: *mut extern "C" fn(*const RTTI, *mut c_void) -> RTTIIter,
    pub fn_clear: *mut extern "C" fn(*const RTTI, *mut c_void) -> bool,
    pub fn_to_string: *mut extern "C" fn(*const c_void, *const RTTI, &String) -> bool,
    pub fn_from_string: *mut extern "C" fn(&RTTIStringSpan, *const RTTI, *const c_void) -> bool,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct RTTIContainerTypeBase {
    pub type_name: *const c_char,
}

#[derive(Debug)]
#[repr(C)]
pub struct RTTIContainer {
    pub base: RTTI,
    pub has_pointers: bool,
    pub item_type: *mut RTTI,
    pub container_type: *mut RTTIContainerTypeBase,
    pub full_type_name: *const c_char,
}

impl NamedRTTI for RTTIContainer {
    fn name(&self) -> std::string::String {
        let container_name = {
            let type_name = unsafe { (&*self.container_type).type_name };
            if type_name.is_null() {
                std::string::String::new()
            } else {
                unsafe { CStr::from_ptr(type_name).to_string_lossy().to_string() }
            }
        };
        if container_name == "cptr".to_string() {
            format!("CPtr<{}>", unsafe { &*self.item_type }.get_symbol_name())
        } else {
            format!(
                "{container_name}<{}>",
                unsafe { &*self.item_type }.get_symbol_name()
            )
        }
    }
}

impl SerializableRTTI for RTTIContainer {
    fn serialize(&self, object: *const c_void) -> Option<std::string::String> {
        if self.base.kind == RTTIKind::Pointer {
            let container = unsafe { &*(self.container_type as *mut RTTIPointerData) };

            // todo(tracking feature) let_chains: https://github.com/rust-lang/rust/issues/53667
            // if !container.fn_get.is_null() && let ptr = container.fn_get(self as *const Self as *mut Self, object) {
            if !container.fn_get.is_null() {
                let ptr =
                    unsafe { (&*container.fn_get)(self as *const Self as *const RTTI, object) };
                if !ptr.is_null() {
                    return unsafe { (&*self.item_type).serialize(ptr) };
                }
            }
            return Some(std::string::String::from("null"));
        } else if self.base.kind == RTTIKind::Container {
            let container = unsafe { &*(self.container_type as *mut RTTIContainerData) };
            if !container.fn_to_string.is_null() {
                let mut string = String::new();
                if unsafe {
                    (&*container.fn_to_string)(
                        object,
                        self as *const Self as *const RTTI,
                        &mut string,
                    )
                } {
                    let cstr = unsafe { std::ffi::CStr::from_ptr(string.data) };
                    let string = cstr.to_string_lossy().into_owned();
                    return Some(string);
                }
            }

            let mut string = std::string::String::from("[");

            let iter =
                unsafe { (&*container.fn_iter_start)(self as *const Self as *const RTTI, object) };
            while unsafe { (&&*container.fn_is_iter_valid)(&iter) } {
                let ptr = unsafe { (&*container.fn_iter_deref)(&iter) };
                let item = unsafe { &*(self.item_type) };
                let str = item
                    .serialize(ptr)
                    .unwrap_or(std::string::String::from("<INVALID>"));
                string += str.as_str();

                unsafe {
                    (&*container.fn_iter_next)(&iter);
                }
                if unsafe { (&*container.fn_is_iter_valid)(&iter) } {
                    string.push_str(",");
                } else {
                    break;
                }
            }
            string.push_str("]");

            return Some(string);
        }
        None
    }

    fn deserialize(&self, object: *mut c_void, value: &mut String) -> bool {
        if self.base.kind == RTTIKind::Pointer {
            return false; // todo: not yet implemented
        } else if self.base.kind == RTTIKind::Container {
            let container = unsafe { &*(self.container_type as *const RTTIContainerData) };

            if value.length() == 0 {
                unsafe {
                    (&*container.fn_clear)(self as *const Self as *const RTTI, object);
                }
                return true;
            }

            if !container.fn_from_string.is_null() {
                return unsafe {
                    (&*container.fn_from_string)(
                        &RTTIStringSpan::new(value),
                        self as *const Self as *const RTTI,
                        object,
                    )
                };
            }

            return false;
        }
        false
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct RTTIEnumValue {
    pub value: i32,
    pub name: *const c_char,
    pub aliases: [*const c_char; 4],
}

impl RTTIEnumValue {
    pub fn names(&self) -> Vec<std::string::String> {
        let mut names = vec![];
        {
            let cstr = unsafe { std::ffi::CStr::from_ptr(self.name) };
            let name = cstr.to_string_lossy().into_owned();
            names.push(name);
        }
        for alias in self.aliases {
            let cstr = unsafe { std::ffi::CStr::from_ptr(alias) };
            let alias = cstr.to_string_lossy().into_owned();
            names.push(alias);
        }

        names
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct RTTIEnum {
    pub base: RTTI,
    pub size: u8,
    pub alignment: u8,
    pub values_len: u16,
    pub type_name: *const c_char,
    pub values: *const RTTIEnumValue,
    pub pod_optimized_type: *const RTTI,
}

impl SerializableRTTI for RTTIEnum {
    fn serialize(&self, object: *const c_void) -> Option<std::string::String> {
        let obj_slice = unsafe { slice::from_raw_parts(object as *const u8, self.size as usize) };

        for value in self.values() {
            if unsafe {
                slice::from_raw_parts(&value.value as *const _ as *const u8, self.size as usize)
                    == obj_slice
            } {
                let cstr = unsafe { std::ffi::CStr::from_ptr(value.name) };
                let name = cstr.to_string_lossy().into_owned();
                return Some(format!("\"{name}\""));
            }
        }
        None
    }

    fn deserialize(&self, object: *mut c_void, value: &mut String) -> bool {
        if self.base.kind == RTTIKind::EnumFlags {
            return false; // todo: not yet implemented
        } else {
            for member in self.values() {
                for name in member.names() {
                    if name.to_owned() == value.as_string() {
                        // this is supposed to replicate memcpy but i have no idea if it actually works lol
                        let src = unsafe {
                            slice::from_raw_parts(
                                &member.value as *const _ as *const u8,
                                size_of::<i32>(),
                            )
                        };
                        let dst = unsafe {
                            slice::from_raw_parts_mut(
                                object as *const _ as *const u8 as *mut u8,
                                size_of::<i32>(),
                            )
                        };
                        dst.copy_from_slice(src);

                        return true;
                    }
                }
            }
        }
        false
    }
}

impl NamedRTTI for RTTIEnum {
    fn name(&self) -> std::string::String {
        if self.type_name.is_null() {
            std::string::String::new()
        } else {
            unsafe { CStr::from_ptr(self.type_name).to_string_lossy().to_string() }
        }
    }
}

impl RTTIEnum {
    pub fn values(&self) -> &[RTTIEnumValue] {
        if self.values_len == 0 {
            &[]
        } else {
            unsafe { slice::from_raw_parts(self.values, self.values_len as usize) }
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct RTTICompoundBase {
    pub r#type: *mut RTTI,
    pub offset: u32,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct RTTICompoundAttributeFlags : u16 {
        const ATTR_DONT_SERIALIZE_BINARY = 0x2;
        const ATTR_VALID_FLAG_MASK = 0xdeb;
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct RTTICompoundAttribute {
    pub r#type: *mut RTTI,
    pub offset: u16,
    pub flags: RTTICompoundAttributeFlags,
    pub name: *const c_char,

    pub fn_get: *mut extern "C" fn(*const c_void, *mut c_void),
    pub fn_set: *mut extern "C" fn(*mut c_void, *const c_void),

    pub min: *const c_char,
    pub max: *const c_char,
}

impl RTTICompoundAttribute {
    pub fn is_group(&self) -> bool {
        self.r#type.is_null()
    }

    pub fn is_save_state_only(&self) -> bool {
        (self.flags.clone() & RTTICompoundAttributeFlags::ATTR_DONT_SERIALIZE_BINARY)
            == RTTICompoundAttributeFlags::ATTR_DONT_SERIALIZE_BINARY
    }

    pub fn is_property(&self) -> bool {
        !self.fn_get.is_null() || !self.fn_set.is_null()
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct RTTICompoundOrderedAttribute {
    pub base: RTTICompoundAttribute,
    pub parent: *const RTTI,
    pub group: *const c_char,
}

#[derive(Debug)]
#[repr(C)]
pub struct RTTICompoundMessageHandler {
    pub message: *mut RTTI,
    pub handler: *mut extern "C" fn(*mut c_void, *mut c_void),
}

#[derive(Debug)]
#[repr(C)]
pub struct RTTICompoundMessageOrderEntry {
    pub before: bool,
    pub message: *mut RTTI,
    pub compound: *mut RTTI,
}

#[derive(Debug)]
#[repr(C)]
pub struct RTTICompound {
    pub base: RTTI,

    pub bases_len: u8,
    pub attributes_len: u8,
    pub message_handlers_len: u8,
    pub message_order_entries_len: u8,

    _pad: [u8; 0x3],
    pub version: u16,
    pub size: u32,
    pub alignment: u16,
    pub serialize_flags: u16,
    pub fn_constructor: FnRTTIConstructor,
    pub fn_destructor: FnRTTIDestructor,
    pub fn_from_string: FnRTTIFromString,
    pub fn_to_string: FnRTTIToString,

    _pad1: [u8; 0x3],
    pub type_name: *const c_char,
    pub cached_type_name_hash: u32,

    _pad2: [u8; 0xC],
    pub bases: *const RTTICompoundBase,
    pub attributes: *const RTTICompoundAttribute,
    pub message_handlers: *const RTTICompoundMessageHandler,
    pub message_order_entries: *const RTTICompoundMessageOrderEntry,
    pub fn_get_symbol_group: *mut extern "C" fn() -> *const RTTI,
    pub pod_optimized_type: *const RTTI,
    pub ordered_attributes: *const RTTICompoundOrderedAttribute,
    pub ordered_attributes_len: u32,
    pub message_read_binary: RTTICompoundMessageHandler,
    pub message_read_binary_offset: u32,
    pub unk0: u32,
}
assert_size!(RTTICompound, 0xb0);

impl SerializableRTTI for RTTICompound {
    fn serialize(&self, object: *const c_void) -> Option<std::string::String> {
        // todo: cache RTTIObject and GGUUID

        if self.base.is_kind(RTTI::find_by_name("RTTIObject")) {
            let downcast =
                unsafe { &mut *(object as *const RTTIObject as *mut RTTIObject) }.GetRTTI();
            if downcast.is_exact_kind(&self.base) {
                return downcast.serialize(object);
            }
        }

        if !self.fn_to_string.is_null() {
            let mut string = String::new();
            return if unsafe { (&*self.fn_to_string)(object, &mut string) } {
                let cstr = unsafe { std::ffi::CStr::from_ptr(string.data) };
                let string = cstr.to_string_lossy().into_owned();
                Some(string)
            } else {
                None
            };
        } else if self.base.is_exact_kind(RTTI::find_by_name("GGUUID")) {
            // handle GGUUID
            let uuid = unsafe { &*(object as *const GGUUID) };
            return Some(format!("\"{uuid}\""));
        }

        let mut string = format!("{{\n\"@type\": \"{}\",", self.name());

        self.visit_members(object as *mut c_void, &mut |member, raw_object| {
            if !member.is_property() {
                let member_value = unsafe { &*member.r#type }
                    .serialize(raw_object)
                    .unwrap_or(std::string::String::from("\"<FAILED>\""));
                let cstr = unsafe { std::ffi::CStr::from_ptr(member.name) };
                let member_name = cstr.to_string_lossy().into_owned();
                string = format!("{}\n\"{}\": {},", string, member_name, member_value)
            }

            false
        });

        Some(string)
    }

    fn deserialize(&self, object: *mut c_void, value: &mut String) -> bool {
        return if !self.fn_from_string.is_null() {
            unsafe { (&*self.fn_from_string)(object, value) }
        } else {
            false
        };
    }
}

impl NamedRTTI for RTTICompound {
    fn name(&self) -> std::string::String {
        if self.type_name.is_null() {
            std::string::String::new()
        } else {
            unsafe { CStr::from_ptr(self.type_name).to_string_lossy().to_string() }
        }
    }
}

impl RTTICompound {
    pub fn bases(&self) -> &[RTTICompoundBase] {
        if self.bases_len == 0 {
            &[]
        } else {
            unsafe { slice::from_raw_parts(self.bases, self.bases_len as usize) }
        }
    }

    pub fn attributes(&self) -> &[RTTICompoundAttribute] {
        if self.attributes_len == 0 {
            &[]
        } else {
            unsafe { slice::from_raw_parts(self.attributes, self.attributes_len as usize) }
        }
    }

    pub fn message_handlers(&self) -> &[RTTICompoundMessageHandler] {
        if self.message_handlers_len == 0 {
            &[]
        } else {
            unsafe {
                slice::from_raw_parts(self.message_handlers, self.message_handlers_len as usize)
            }
        }
    }

    pub fn message_order_entries(&self) -> &[RTTICompoundMessageOrderEntry] {
        if self.message_order_entries_len == 0 {
            &[]
        } else {
            unsafe {
                slice::from_raw_parts(
                    self.message_order_entries,
                    self.message_order_entries_len as usize,
                )
            }
        }
    }

    pub fn ordered_attributes(&self) -> &[RTTICompoundOrderedAttribute] {
        if self.ordered_attributes_len == 0 {
            &[]
        } else {
            unsafe {
                slice::from_raw_parts(
                    self.ordered_attributes,
                    self.ordered_attributes_len as usize,
                )
            }
        }
    }

    pub fn visit_members<C: FnMut(&RTTICompoundAttribute, *mut c_void) -> bool>(
        &self,
        object: *mut c_void,
        callback: &mut C,
    ) -> bool {
        self.visit_attributes(
            &mut |member, _, base_offset, _| {
                if member.is_group() {
                    return false;
                }
                let raw_object = unsafe {
                    std::mem::transmute::<usize, *mut c_void>(
                        std::mem::transmute::<*mut c_void, usize>(object)
                            + base_offset as usize
                            + member.offset as usize,
                    )
                };
                callback(member, raw_object)
            },
            0,
            true,
        )
    }

    pub fn visit_attributes<
        C: FnMut(&RTTICompoundAttribute, std::string::String, u32, bool) -> bool,
    >(
        &self,
        callback: &mut C,
        base_offset: u32,
        top_level: bool,
    ) -> bool {
        for base in self.bases() {
            if unsafe { &*(base.r#type as *const RTTICompound) }.visit_attributes(
                callback,
                base_offset + base.offset,
                false,
            ) {
                return true;
            }
        }

        let mut category = std::string::String::new();
        for member in self.attributes() {
            if member.is_group() {
                let cstr = unsafe { std::ffi::CStr::from_ptr(member.name) };
                category = cstr.to_string_lossy().into_owned();
            }
            if callback(member, category.clone(), base_offset, top_level) {
                return true;
            }
        }

        false
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct RTTIPod {
    pub base: RTTI,
    pub size: u32,
}

impl NamedRTTI for RTTIPod {
    fn name(&self) -> std::string::String {
        format!("POD({})", self.size)
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct RTTIEnumBitSet {
    pub r#type: *mut RTTI,
    pub type_name: *const c_char,
}

impl NamedRTTI for RTTIEnumBitSet {
    fn name(&self) -> std::string::String {
        if self.type_name.is_null() {
            std::string::String::new()
        } else {
            unsafe { CStr::from_ptr(self.type_name).to_string_lossy().to_string() }
        }
    }
}
