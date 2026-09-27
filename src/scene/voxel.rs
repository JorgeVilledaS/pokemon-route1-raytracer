use crate::materials::MaterialId;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RotationY {
    #[default]
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

impl RotationY {
    pub fn rotate_uv(self, (u, v): (f64, f64)) -> (f64, f64) {
        match self {
            RotationY::Deg0 => (u, v),
            RotationY::Deg90 => (v, 1.0 - u),
            RotationY::Deg180 => (1.0 - u, 1.0 - v),
            RotationY::Deg270 => (1.0 - v, u),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Voxel {
    pub material: MaterialId,
    pub rotation_y: RotationY,
}

impl Voxel {
    pub const fn new(material: MaterialId, rotation_y: RotationY) -> Self {
        Self {
            material,
            rotation_y,
        }
    }
}
