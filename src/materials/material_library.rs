use super::Material;
use crate::core::Vec3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaterialId {
    Grass,
    DirtPath,
    Wood,
    Leaves,
    Water,
    EmissiveSign,
}

pub fn material(id: MaterialId) -> Material {
    let albedo = match id {
        MaterialId::Grass => Vec3::new(0.18, 0.62, 0.25),
        MaterialId::DirtPath => Vec3::new(0.78, 0.68, 0.35),
        MaterialId::Wood => Vec3::new(0.36, 0.16, 0.08),
        MaterialId::Leaves => Vec3::new(0.12, 0.48, 0.18),
        MaterialId::Water => Vec3::new(0.08, 0.35, 0.65),
        MaterialId::EmissiveSign => Vec3::new(0.65, 0.32, 0.12),
    };
    Material {
        albedo,
        specular: 0.2,
        reflectivity: 0.0,
        transparency: 0.0,
        ior: 1.0,
        emissive: Vec3::default(),
        texture: None,
        normal_map: None,
    }
}
