use std::ffi::c_void;

use crate::types::decima::p_core::prelude::*;

use super::{
    entity::Entity,
    rtti_ref_object::RTTIRefObject,
    weak_ptr::{WeakPtrBase, WeakPtrTarget},
};

#[repr(C)]
#[derive(Debug)]
pub struct EntityComponent {
    pub __vftable: *mut c_void,
    // RTTIRefObject (base)
    pub uuid: GGUUID,
    pub refs: u32,

    // WeakPtrRTTITarget (base)
    pub ptr: *mut WeakPtrTarget,
    pub prev: *mut WeakPtrBase,
    pub next: *mut WeakPtrBase,

    pub resource: Ref<RTTIRefObject>, // EntityComponentResource : RTTIRefObject
    pub initialized: bool,
    pub representation: *mut c_void,
    pub entity: *mut Entity,
}

#[repr(C)]
#[derive(Debug)]
pub struct EntityComponentContainer {
    pub components: Array<*mut EntityComponent>,
    pub component_types: Array<u16>,
}
