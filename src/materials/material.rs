use crate::{core::Vec3, utils::Texture};

#[derive(Clone, Debug)]
pub struct Material {
    pub albedo: Texture,
    pub ks: f64,
    pub shininess: f64,
    pub kr: f64,
    pub kt: f64,
    pub ior: f64,
    pub emissive: Vec3,
    pub normal_map: Option<Texture>,
}
