use crate::assert_size;
use glam::{DVec3, Mat3};

use super::{rot_matrix::RotMatrix, world_position::WorldPosition};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct WorldTransform {
    pub pos: WorldPosition,
    pub rot: RotMatrix,
}
assert_size!(WorldTransform, 0x40);

/// A [WorldTransform] representation using [glam] types.
#[derive(Copy, Clone)]
pub struct GlamTransform {
    pub pos: DVec3,
    pub rot: Mat3,
}

impl From<WorldTransform> for GlamTransform {
    fn from(value: WorldTransform) -> Self {
        GlamTransform {
            pos: DVec3::from(value.pos),
            rot: Mat3::from(value.rot),
        }
    }
}

impl From<GlamTransform> for WorldTransform {
    fn from(value: GlamTransform) -> Self {
        WorldTransform {
            pos: WorldPosition::from(value.pos),
            rot: RotMatrix::from(value.rot),
        }
    }
}
