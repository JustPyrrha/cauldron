use super::entity::Entity;
use crate::mem::offsets::Offset;
use crate::types::decima::core::enums::EPhysicsCollisionLayerGame;
use crate::types::decima::core::vec::Vec3;
use crate::types::decima::core::world_position::WorldPosition;
use std::ffi::c_void;

pub struct GCore {}

impl GCore {
    /// bool GCore::IntersectLine(WorldPosition const &, WorldPosition const &, EPhysicsCollisionLayerGame, Entity const *, bool, bool, bool, WorldPosition *, Vec3 *, float *, Entity * *, MaterialTypeResource const * *, int *, uint32 *);
    #[allow(non_snake_case)]
    pub fn IntersectLine(
        line_start: &WorldPosition,
        line_end: &WorldPosition,
        layer: EPhysicsCollisionLayerGame,
        entity: *const Entity,
        flag_unk_a: bool,
        flag_unk_b: bool,
        unk0: i32,

        out_pos: *mut WorldPosition,
        out_dir: *mut Vec3,
        out_f: *mut f32,
        out_entity: *mut *mut Entity,
        out_object: *mut *mut c_void, // todo: check if this is something with an RTTI reference
        out_int: *mut i32,
        out_uint: *mut u32,
    ) -> bool {
        let func = Offset::from_signature(
            "4C 8B DC 49 89 5B 10 49 89 73 18 55 57 41 54 41 55 41 57 48 8D 6C 24 90",
        )
        .unwrap()
        .as_ptr::<c_void>();

        let func: extern "fastcall" fn(
            line_start: &WorldPosition,
            line_end: &WorldPosition,
            layer: EPhysicsCollisionLayerGame,
            entity: *const Entity,
            flag_unk_a: bool,
            flag_unk_b: bool,
            unk0: i32,

            out_pos: *mut WorldPosition,
            out_dir: *mut Vec3,
            out_f: *mut f32,
            out_entity: *mut *mut Entity,
            out_object: *mut *mut c_void,
            out_int: *mut i32,
            out_uint: *mut u32,
        ) -> bool = unsafe { std::mem::transmute(func) };

        func(
            line_start, line_end, layer, entity, flag_unk_a, flag_unk_b, unk0, out_pos, out_dir,
            out_f, out_entity, out_object, out_int, out_uint,
        )
    }
}
