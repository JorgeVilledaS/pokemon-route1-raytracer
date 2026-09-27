use crate::{core::Vec3, utils::Texture};

#[derive(Clone, Debug)]
pub struct Material {
    pub albedo: Vec3,
    pub specular: f32,
    pub reflectivity: f32,
    pub transparency: f32,
    pub ior: f32,
    pub emissive: Vec3,
    pub texture: Option<Texture>,
    pub normal_map: Option<Texture>,
}
