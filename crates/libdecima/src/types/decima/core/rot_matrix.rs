use super::vec::Vec3Pack;
use glam::{Mat3, Vec3};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RotMatrix {
    pub col0: Vec3Pack,
    pub col1: Vec3Pack,
    pub col2: Vec3Pack,
}

impl From<Mat3> for RotMatrix {
    fn from(value: Mat3) -> Self {
        RotMatrix {
            col0: value.x_axis.into(),
            col1: value.y_axis.into(),
            col2: value.z_axis.into(),
        }
    }
}

impl From<RotMatrix> for Mat3 {
    fn from(value: RotMatrix) -> Self {
        Mat3::from_cols(value.col0.into(), value.col1.into(), value.col2.into())
    }
}

pub trait DecimaCoordinateDirection {
    type Component;

    fn right(&self) -> &Self::Component;
    fn forward(&self) -> &Self::Component;
    fn up(&self) -> &Self::Component;
}

impl DecimaCoordinateDirection for RotMatrix {
    type Component = Vec3Pack;

    fn right(&self) -> &Self::Component {
        &self.col0
    }

    fn forward(&self) -> &Self::Component {
        &self.col1
    }

    fn up(&self) -> &Self::Component {
        &self.col2
    }
}

impl DecimaCoordinateDirection for Mat3 {
    type Component = Vec3;

    fn right(&self) -> &Self::Component {
        &self.x_axis
    }

    fn forward(&self) -> &Self::Component {
        &self.y_axis
    }

    fn up(&self) -> &Self::Component {
        &self.z_axis
    }
}
