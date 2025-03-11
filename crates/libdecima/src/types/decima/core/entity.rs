use super::mover::Mover;
use super::{
    ai_faction::AIFaction,
    entity_component::EntityComponentContainer,
    rtti_ref_object::RTTIRefObject,
    streaming_ref::StreamingRef,
    weak_ptr::{WeakPtrBase, WeakPtrTarget},
    world_transform::WorldTransform,
};
use crate::{assert_size, types::decima::p_core::prelude::GGUUID};
use std::ffi::c_void;

#[repr(C)]
#[derive(Debug)]
pub struct EntityResource {
    pub __vftable: *mut c_void,
    pub base__: RTTIRefObject,
}

#[repr(C)]
#[derive(Debug)]
pub struct Entity_vtbl {}

#[repr(C)]
#[derive(Debug)]
pub struct Entity {
    pub __vftable: *mut Entity_vtbl, // 0x8

    // RTTIRefObject (base)
    pub uuid: GGUUID,
    pub refs: u32,

    // WeakPtrRTTITarget (base)
    pub ptr: *mut WeakPtrTarget,
    pub prev: *mut WeakPtrBase,
    pub next: *mut WeakPtrBase,

    _pad0: [u8; 0x30],
    pub entity_resource: StreamingRef<EntityResource>,
    _pad1: [u8; 0x10],
    pub parent: *mut Entity,
    pub left_most_child: *mut Entity,
    pub right_sibling: *mut Entity,
    pub flags: i32,
    pub components: EntityComponentContainer,
    pub mover: *mut Mover,
    pub model: *mut c_void,
    pub destructibility: *mut c_void,
    pub transform: WorldTransform,
    _pad3: [u8; 0x70],
    pub faction: *mut AIFaction,
    _pad4: [u8; 0x108],
    //pub access_mutex: *mut c_void, // RecursiveMutex,
    _pad5: [u8; 0x68],
}
assert_size!(Entity, 0x300);

impl Entity {
    pub fn get_transform(&self) -> WorldTransform {
        // we REALLY should be locking access_mutex but mutex's from c++ cant be easily bound in rust
        self.transform.clone()
    }

    pub fn set_transform(&mut self, transform: &WorldTransform) {
        self.transform = transform.clone();
        self.flags |= 0x1; // Changed
    }
}
