use crate::assert_size;
use glam::Vec3 as Vector3;

#[repr(C, align(16))]
#[derive(Debug, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C, align(4))]
#[derive(Debug, Clone, Default, Copy)]
pub struct Vec3Pack {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
assert_size!(Vec3Pack, 0xC);

impl From<Vector3> for Vec3 {
    fn from(value: Vector3) -> Self {
        Vec3 {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<Vector3> for Vec3Pack {
    fn from(value: Vector3) -> Self {
        Vec3Pack {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<Vec3> for Vector3 {
    fn from(value: Vec3) -> Self {
        Vector3 {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<Vec3Pack> for Vector3 {
    fn from(value: Vec3Pack) -> Self {
        Vector3 {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}
