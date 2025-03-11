use glam::DVec3;

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct WorldPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl From<DVec3> for WorldPosition {
    fn from(value: DVec3) -> Self {
        WorldPosition {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<WorldPosition> for DVec3 {
    fn from(value: WorldPosition) -> Self {
        DVec3 {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}
