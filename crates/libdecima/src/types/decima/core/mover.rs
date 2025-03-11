use crate::types::decima::core::entity::Entity;
use crate::types::decima::core::rtti::RTTI;
use crate::types::decima::core::rtti_ref_object::RTTIRefObject;
use crate::types::decima::core::weak_ptr::{WeakPtrBase, WeakPtrTarget};
use crate::types::decima::core::world_transform::WorldTransform;
use crate::types::decima::p_core::prelude::{GGUUID, Ref};
use crate::{assert_size, gen_with_vtbl};
use std::ffi::c_void;

gen_with_vtbl!(
    Mover,

    fn get_rtti() -> *const RTTI;
    fn fn_destroy();
    fn is_active() -> bool;
    fn set_active(active: bool);
    fn override_movement(transform: &WorldTransform, duration: f32, unk: bool);
    fn update_override_move_target(transform: &WorldTransform);
    fn is_movement_overridden() -> bool;
    fn stop_override_movement();
    fn get_move_duration() -> f32;
    fn get_move_time() -> f32;
    fn get_move_speed() -> f32;

    // RTTIRefObject (base)
    pub uuid: GGUUID,
    pub refs: u32,

    // WeakPtrRTTITarget (base)
    pub ptr: *mut WeakPtrTarget,
    pub prev: *mut WeakPtrBase,
    pub next: *mut WeakPtrBase,

    // EntityComponent (base)
    pub resource: Ref<RTTIRefObject>, // class EntityComponentResource : public RTTIRefObject
    pub initialized: bool,
    pub representation: *mut c_void,
    pub entity: *mut Entity,
);

assert_size!(Mover, 0x58);
