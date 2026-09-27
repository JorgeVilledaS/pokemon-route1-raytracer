use super::Material;
use crate::core::Vec3;
use crate::utils::Texture;
use std::io;
use std::path::Path;

#[repr(usize)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MaterialId {
    Grass,
    Dirt,
    Wood,
    Water,
    Rock,
    Sign,
    Air,
}

impl MaterialId {
    pub const COUNT: usize = 7;
}

/// Array indexado por `MaterialId as usize`: evita hashing en el hot path por rayo.
pub struct MaterialLibrary {
    materials: [Material; MaterialId::COUNT],
}

impl MaterialLibrary {
    pub fn load_all() -> io::Result<Self> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/textures");
        let load = |name: &str| Texture::load_ppm(root.join(name));

        let grass = Material {
            albedo: load("grass_top.ppm")?,
            ks: 0.08,
            shininess: 8.0,
            kr: 0.0,
            kt: 0.0,
            ior: 1.0,
            emissive: Vec3::default(),
            normal_map: None,
        };
        let dirt = Material {
            albedo: load("dirt_path.ppm")?,
            ks: 0.03,
            shininess: 4.0,
            kr: 0.0,
            kt: 0.0,
            ior: 1.0,
            emissive: Vec3::default(),
            normal_map: None,
        };
        let wood = Material {
            albedo: load("bark.ppm")?,
            ks: 0.25,
            shininess: 32.0,
            kr: 0.05,
            kt: 0.0,
            ior: 1.0,
            emissive: Vec3::default(),
            normal_map: Some(load("bark_normal.ppm")?),
        };
        let water = Material {
            albedo: load("water.ppm")?,
            ks: 0.85,
            shininess: 128.0,
            kr: 0.2,
            kt: 0.85,
            ior: 1.33,
            emissive: Vec3::default(),
            normal_map: None,
        };
        let rock = Material {
            albedo: load("rock.ppm")?,
            ks: 0.55,
            shininess: 64.0,
            kr: 0.1,
            kt: 0.0,
            ior: 1.0,
            emissive: Vec3::default(),
            normal_map: None,
        };
        let sign = Material {
            albedo: load("sign_wood.ppm")?,
            ks: 0.15,
            shininess: 16.0,
            kr: 0.0,
            kt: 0.0,
            ior: 1.0,
            emissive: Vec3::new(0.03, 0.015, 0.005),
            normal_map: None,
        };
        let air = Material {
            albedo: Texture::solid(Vec3::default()),
            ks: 0.0,
            shininess: 1.0,
            kr: 0.0,
            kt: 1.0,
            ior: 1.0,
            emissive: Vec3::default(),
            normal_map: None,
        };

        Ok(Self {
            materials: [grass, dirt, wood, water, rock, sign, air],
        })
    }

    pub fn get(&self, id: MaterialId) -> &Material {
        &self.materials[id as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::{MaterialId, MaterialLibrary};

    #[test]
    fn loads_every_material_by_direct_index() {
        let library = MaterialLibrary::load_all().expect("project textures should load");

        assert_eq!(library.get(MaterialId::Grass).albedo.width, 8);
        assert_eq!(library.get(MaterialId::Water).ior, 1.33);
        assert!(library.get(MaterialId::Wood).normal_map.is_some());
        assert_eq!(library.get(MaterialId::Air).kt, 1.0);
    }
}
