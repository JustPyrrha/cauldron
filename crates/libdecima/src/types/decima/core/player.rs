use std::{ffi::c_void, fmt::Debug};

use crate::{
    assert_offset, assert_size,
    mem::offsets::Offset,
    types::decima::p_core::prelude::{Array, GGUUID},
};

use super::{
    ai_faction::AIFaction,
    camera_entity::{CameraEntity, UnkCameraEntityRef},
    entity::Entity,
    rtti::RTTI,
    weak_ptr::WeakPtrBase,
};

#[repr(C)]
#[derive(Debug)]
pub struct Player_vtbl {
    pub fn_get_rtti: extern "C" fn(this: *mut Player) -> *const RTTI,
    pub fn_dtor: extern "C" fn(this: *mut Player),
    pub fn_unk_04: extern "C" fn(),
    pub fn_unk_05: extern "C" fn(),
    pub fn_unk_06: extern "C" fn(),
    pub fn_unk_07: extern "C" fn(),
    pub fn_unk_08: extern "C" fn(),
    pub fn_unk_09: extern "C" fn(),
    pub fn_unk_10: extern "C" fn(),
    pub fn_unk_11: extern "C" fn(),
    pub fn_unk_12: extern "C" fn(),
    pub fn_unk_13: extern "C" fn(),
    pub fn_unk_14: extern "C" fn(),
    pub fn_unk_15: extern "C" fn(),
    pub fn_unk_16: extern "C" fn(),
    pub fn_unk_17: extern "C" fn(),
}

#[repr(C)]
pub struct Player {
    pub __vftable: *mut Player_vtbl,

    // RTTIRefObject (base)
    pub uuid: GGUUID,
    pub refs: u32,

    // WeakPtrRTTITarget (base)
    pub target_head: *mut WeakPtrBase,

    _pad00: *mut c_void, // theres something missing here but im not sure what just yet, its 100% inheritance related

    _pad0: [u8; 0x18],
    pub entity: *mut Entity,
    _pad1: [u8; 0x10],
    pub faction: *mut AIFaction,
    _pad2: [u8; 0x88],
    pub camera_stack: Array<UnkCameraEntityRef>,
    // pub camera_stack_mutex: *mut c_void,
    _pad3: [u8; 0x28],
}

assert_offset!(Player, entity, 0x48);
assert_offset!(Player, faction, 0x60);
assert_offset!(Player, camera_stack, 0xf0);
assert_size!(Player, 0x128);

impl Player {
    pub fn get_local_player(index: u32) -> *mut Player {
        let func = Offset::from_signature("40 57 48 83 EC 30 48 63 F9 48 8B 0D ? ? ? ? 48 85 C9")
            .unwrap()
            .as_ptr::<c_void>();

        let func: extern "fastcall" fn(u32) -> *mut Player = unsafe { std::mem::transmute(func) };

        func(index)
    }

    pub fn get_last_active_camera(&self) -> Option<*mut CameraEntity> {
        if self.camera_stack.count == 0 {
            None
        } else {
            Some(self.camera_stack.as_slice().last().unwrap().base__.get())
        }
    }
}

impl Debug for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Player")
            .field("__vftable", &self.__vftable)
            .field("uuid", &self.uuid)
            .field("refs", &self.refs)
            .field("target_head", &self.target_head)
            .field("_pad00", &"[..]".to_string())
            .field("_pad0", &"[..]".to_string())
            .field("entity", &self.entity)
            .field("_pad1", &"[..]".to_string())
            .field("faction", &self.faction)
            .field("_pad2", &"[..]".to_string())
            .field("camera_stack", &self.camera_stack)
            .field("_pad3", &"[..]".to_string())
            .finish()
    }
}
