use super::{intersect, Hit};
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
        Self {
            direction: Vec3::new(-0.6, 1.0, -0.35).normalize(),
            color: Vec3::new(1.0, 0.94, 0.82),
            intensity: 1.0,
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
    let albedo = material.albedo.sample(hit.uv.0, hit.uv.1);
    let ambient = albedo * 0.2;
    let light_direction = light.direction.normalize();
    let normal_dot_light = hit.normal.dot(light_direction).max(0.0);

    if normal_dot_light == 0.0 || is_shadowed(hit, light, scene) {
        return ambient + material.emissive;
    }

    let diffuse = component_mul(albedo, light.color) * (0.8 * light.intensity * normal_dot_light);

    let half_vector = (light_direction + view_direction.normalize()).normalize();
    let specular_strength = if normal_dot_light > 0.0 {
        material.ks
            * hit
                .normal
                .dot(half_vector)
                .max(0.0)
                .powf(material.shininess)
            * light.intensity
    } else {
        0.0
    };
    let specular = light.color * specular_strength;

    ambient + diffuse + specular + material.emissive
}

pub fn is_shadowed(hit: &Hit, light: &Light, scene: &Scene) -> bool {
    let shadow_origin = hit.point + hit.normal * RAY_BIAS;
    let shadow_ray = Ray::new(shadow_origin, light.direction.normalize());
    intersect(scene, &shadow_ray).is_some()
}

fn component_mul(left: Vec3, right: Vec3) -> Vec3 {
    Vec3::new(left.x * right.x, left.y * right.y, left.z * right.z)
}

#[cfg(test)]
mod tests {
    use super::{is_shadowed, shade, Light};
    use crate::core::Vec3;
    use crate::materials::{MaterialId, MaterialLibrary};
    use crate::render::Hit;
    use crate::scene::{RotationY, Scene, Voxel};

    fn top_hit() -> Hit {
        Hit {
            point: Vec3::new(0.5, 1.0, 0.5),
            normal: Vec3::new(0.0, 1.0, 0.0),
            uv: (0.5, 0.5),
            material: MaterialId::Grass,
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
}
