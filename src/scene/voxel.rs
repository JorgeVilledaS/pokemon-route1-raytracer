use crate::materials::MaterialId;

#[derive(Clone, Copy, Debug)]
pub struct Voxel {
    pub material: MaterialId,
    pub texture_rotation: u8,
}
