use std::ffi::c_void;

use super::mover::Mover;
use super::{entity::Entity, weak_ptr::WeakPtr};
use crate::assert_size;
use crate::types::decima::core::ai_faction::AIFaction;
use crate::types::decima::core::entity::EntityResource;
use crate::types::decima::core::entity_component::EntityComponentContainer;
use crate::types::decima::core::streaming_ref::StreamingRef;
use crate::types::decima::core::weak_ptr::{WeakPtrBase, WeakPtrTarget};
use crate::types::decima::core::world_transform::WorldTransform;
use crate::types::decima::p_core::prelude::GGUUID;

#[repr(C)]
#[derive(Debug)]
pub struct CameraEntity {
    pub __vftable: *mut c_void,

    // these are all from entity.rs cause c++ inheritance is weird

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
    pub flags: u32,
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

#[repr(C)]
#[derive(Debug)]
pub struct UnkCameraEntityRef {
    pub base__: WeakPtr<CameraEntity>,
    _pad0: [u8; 0x68],
}
assert_size!(UnkCameraEntityRef, 0x80);

impl CameraEntity {
    pub fn get_transform(&self) -> WorldTransform {
        // we REALLY should be locking access_mutex but mutex's from c++ cant be easily bound in rust
        self.transform.clone()
    }
}
