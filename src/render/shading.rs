use super::{occluded, Face, Hit, SceneMode};
use crate::core::{Ray, Vec3};
use crate::materials::Material;
use crate::scene::Scene;

pub const RAY_BIAS: f64 = 1.0e-4;

/// `direction` apunta desde la superficie hacia el sol.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Light {
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: f64,
}

impl Light {
    pub fn route_one_sun() -> Self {
        Self::for_mode(SceneMode::Day)
    }

    pub fn for_mode(mode: SceneMode) -> Self {
        match mode {
            SceneMode::Day => Self {
                direction: Vec3::new(-0.6, 1.0, -0.35).normalize(),
                color: Vec3::new(1.0, 0.94, 0.82),
                intensity: 1.0,
            },
            SceneMode::Night => Self {
                direction: Vec3::new(-0.35, 1.0, 0.2).normalize(),
                color: Vec3::new(0.32, 0.40, 0.68),
                intensity: 0.08,
            },
        }
    }
}

/// Blinn-Phong puro. `view_direction` apunta del hit hacia la cámara.
pub fn shade(
    hit: &Hit,
    material: &Material,
    light: &Light,
    scene: &Scene,
    view_direction: Vec3,
) -> Vec3 {
    let mut albedo = material.albedo.sample(hit.uv.0, hit.uv.1);
    // Motivos de superficie en espacio UV: contornos y grupos de hojas grandes.
    // Conservan el albedo PPM como base y evitan ruido fino tipo grava.
    use crate::materials::MaterialId;
    let (u, v) = hit.uv;
    if hit.material == MaterialId::Flowers {
        let x = u - 0.5;
        let y = v - 0.5;
        let r = x * x + y * y;
        albedo = if r < 0.014 {
            Vec3::new(1.0, 0.72, 0.08)
        } else if r < 0.11 {
            Vec3::new(0.95, 0.12, 0.27)
        } else {
            Vec3::new(0.11, 0.49, 0.22)
        };
    } else if hit.material == MaterialId::Leaves {
        let x = u;
        let y = v;
        let leaf = (x - 0.5).abs() * 1.1 + (y - 0.5).abs();
        albedo = if leaf > 0.55 {
            Vec3::new(0.035, 0.22, 0.12)
        } else if x + y < 0.85 {
            Vec3::new(0.35, 0.72, 0.19)
        } else {
            Vec3::new(0.13, 0.49, 0.15)
        };
    } else if hit.material == MaterialId::TallGrass {
        let blade = ((u * 2.0 + v).fract() - 0.5).abs();
        albedo = if blade < 0.12 {
            Vec3::new(0.035, 0.28, 0.12)
        } else {
            Vec3::new(0.23, 0.62, 0.23)
        };
    } else if hit.material == MaterialId::Grass {
        albedo = Vec3::new(0.24, 0.62, 0.32) * 0.85 + albedo * 0.15;
    }
    let emissive = material
        .emissive_map
        .as_ref()
        .map_or(material.emissive, |map| {
            component_mul(map.sample(hit.uv.0, hit.uv.1), material.emissive)
        });
    let normal = mapped_normal(hit, material);
    let ambient = albedo * (0.03 + 0.17 * light.intensity.min(1.0));
    let light_direction = light.direction.normalize();
    let normal_dot_light = normal.dot(light_direction).max(0.0);

    if normal_dot_light == 0.0 || is_shadowed(hit, light, scene) {
        return ambient + emissive;
    }

    let diffuse = component_mul(albedo, light.color) * (0.8 * light.intensity * normal_dot_light);

    let half_vector = (light_direction + view_direction.normalize()).normalize();
    let specular_strength = if normal_dot_light > 0.0 {
        material.ks * normal.dot(half_vector).max(0.0).powf(material.shininess) * light.intensity
    } else {
        0.0
    };
    let specular = light.color * specular_strength;

    ambient + diffuse + specular + emissive
}

pub fn mapped_normal(hit: &Hit, material: &Material) -> Vec3 {
    let Some(normal_map) = &material.normal_map else {
        return hit.normal;
    };

    let encoded = normal_map.sample(hit.uv.0, hit.uv.1);
    let tangent_space = Vec3::new(
        encoded.x * 2.0 - 1.0,
        encoded.y * 2.0 - 1.0,
        encoded.z * 2.0 - 1.0,
    )
    .normalize();
    let (tangent, bitangent) = tangent_basis(hit.face);

    (tangent * tangent_space.x + bitangent * tangent_space.y + hit.normal * tangent_space.z)
        .normalize()
}

fn tangent_basis(face: Face) -> (Vec3, Vec3) {
    match face {
        Face::PositiveX => (Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 1.0, 0.0)),
        Face::NegativeX => (Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 0.0)),
        Face::PositiveY => (Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0)),
        Face::NegativeY => (Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0)),
        Face::PositiveZ => (Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
        Face::NegativeZ => (Vec3::new(-1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
    }
}

pub fn is_shadowed(hit: &Hit, light: &Light, scene: &Scene) -> bool {
    let shadow_origin = hit.point + hit.normal * RAY_BIAS;
    let shadow_ray = Ray::new(shadow_origin, light.direction.normalize());
    scene
        .actor
        .iter()
        .chain(scene.encounters.iter())
        .any(|a| super::avatar::hit(&shadow_ray, *a).is_some())
        || occluded(scene, &shadow_ray)
}

fn component_mul(left: Vec3, right: Vec3) -> Vec3 {
    Vec3::new(left.x * right.x, left.y * right.y, left.z * right.z)
}

#[cfg(test)]
mod tests {
    use super::{is_shadowed, mapped_normal, shade, Light};
    use crate::core::Vec3;
    use crate::materials::{MaterialId, MaterialLibrary};
    use crate::render::{Face, Hit};
    use crate::scene::{RotationY, Scene, Voxel};

    fn top_hit() -> Hit {
        Hit {
            point: Vec3::new(0.5, 1.0, 0.5),
            normal: Vec3::new(0.0, 1.0, 0.0),
            uv: (0.5, 0.5),
            material: MaterialId::Grass,
            face: Face::PositiveY,
            cell: (0, 0, 0),
        }
    }

    #[test]
    fn bias_prevents_surface_from_shadowing_itself() {
        let mut scene = Scene::new(3, 3, 1);
        scene.set(
            0,
            0,
            0,
            Some(Voxel::new(MaterialId::Grass, RotationY::Deg0)),
        );
        let light = Light {
            direction: Vec3::new(1.0, 1.0, 0.0).normalize(),
            color: Vec3::new(1.0, 1.0, 1.0),
            intensity: 1.0,
        };

        assert!(!is_shadowed(&top_hit(), &light, &scene));
    }

    #[test]
    fn blocker_removes_direct_light() {
        let library = MaterialLibrary::load_all().expect("project textures should load");
        let material = library.get(MaterialId::Grass);
        let light = Light {
            direction: Vec3::new(1.0, 1.0, 0.0).normalize(),
            color: Vec3::new(1.0, 1.0, 1.0),
            intensity: 1.0,
        };
        let clear_scene = Scene::new(3, 3, 1);
        let mut blocked_scene = Scene::new(3, 3, 1);
        blocked_scene.set(1, 1, 0, Some(Voxel::new(MaterialId::Rock, RotationY::Deg0)));

        let clear = shade(
            &top_hit(),
            material,
            &light,
            &clear_scene,
            Vec3::new(0.0, 1.0, 0.0),
        );
        let blocked = shade(
            &top_hit(),
            material,
            &light,
            &blocked_scene,
            Vec3::new(0.0, 1.0, 0.0),
        );

        assert!(is_shadowed(&top_hit(), &light, &blocked_scene));
        assert!(clear.length() > blocked.length());
    }

    #[test]
    fn wood_normal_map_perturbs_axis_aligned_face() {
        let library = MaterialLibrary::load_all().expect("project textures should load");
        let material = library.get(MaterialId::Wood);
        let mut hit = top_hit();
        hit.face = Face::PositiveZ;
        hit.normal = Vec3::new(0.0, 0.0, 1.0);
        hit.uv = (0.1, 0.1);

        let normal = mapped_normal(&hit, material);

        assert!((normal - hit.normal).length() > 0.02);
        assert!((normal.length() - 1.0).abs() < 1.0e-12);
    }

    #[test]
    fn emissive_sign_stays_bright_at_night() {
        let library = MaterialLibrary::load_all().expect("project textures should load");
        let scene = Scene::new(1, 1, 1);
        let light = Light::for_mode(crate::render::SceneMode::Night);
        let hit = top_hit();

        let grass = shade(
            &hit,
            library.get(MaterialId::Grass),
            &light,
            &scene,
            Vec3::new(0.0, 1.0, 0.0),
        );
        let sign = shade(
            &hit,
            library.get(MaterialId::Sign),
            &light,
            &scene,
            Vec3::new(0.0, 1.0, 0.0),
        );

        assert!(sign.length() > grass.length() * 2.0);
    }
}
