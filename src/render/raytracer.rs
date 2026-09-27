use super::shading::{shade, Light, RAY_BIAS};
use super::{intersect_with_steps, Skybox};
use crate::core::{Ray, Vec3};
use crate::materials::MaterialLibrary;
use crate::scene::Scene;

pub struct Raytracer<'a> {
    scene: &'a Scene,
    materials: &'a MaterialLibrary,
    light: Light,
    pub max_depth: u32,
}

impl<'a> Raytracer<'a> {
    pub const fn new(
        scene: &'a Scene,
        materials: &'a MaterialLibrary,
        light: Light,
        max_depth: u32,
    ) -> Self {
        Self {
            scene,
            materials,
            light,
            max_depth,
        }
    }

    pub fn trace(&self, ray: &Ray, depth: u32) -> Vec3 {
        self.trace_with_steps(ray, depth).0
    }

    pub fn trace_with_steps(&self, ray: &Ray, depth: u32) -> (Vec3, usize) {
        let (hit, mut steps) = intersect_with_steps(self.scene, ray);
        let Some(hit) = hit else {
            return (Skybox.sample(ray.dir), steps);
        };

        let material = self.materials.get(hit.material);
        let local_color = shade(&hit, material, &self.light, self.scene, -ray.dir);

        if depth == 0 || material.kr <= 0.0 {
            return (local_color, steps);
        }

        let reflected_direction = ray.dir.reflect(hit.normal).normalize();
        let bias_normal = if reflected_direction.dot(hit.normal) >= 0.0 {
            hit.normal
        } else {
            -hit.normal
        };
        let reflected_origin = hit.point + bias_normal * RAY_BIAS;
        let reflected_ray = Ray::new(reflected_origin, reflected_direction);
        let (reflected_color, reflected_steps) = self.trace_with_steps(&reflected_ray, depth - 1);
        steps += reflected_steps;

        (
            local_color * (1.0 - material.kr) + reflected_color * material.kr,
            steps,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Raytracer;
    use crate::core::{Ray, Vec3};
    use crate::materials::{MaterialId, MaterialLibrary};
    use crate::render::shading::Light;
    use crate::scene::{RotationY, Scene, Voxel};

    #[test]
    fn explicit_depth_stops_between_two_reflective_surfaces() {
        let mut scene = Scene::new(3, 1, 1);
        let mirror = Voxel::new(MaterialId::Water, RotationY::Deg0);
        scene.set(0, 0, 0, Some(mirror));
        scene.set(2, 0, 0, Some(mirror));

        let materials = MaterialLibrary::load_all().expect("project textures should load");
        let raytracer = Raytracer::new(&scene, &materials, Light::route_one_sun(), 4);
        let ray = Ray::new(Vec3::new(1.5, 0.5, 0.5), Vec3::new(1.0, 0.0, 0.0));
        let (color, steps) = raytracer.trace_with_steps(&ray, 4);

        assert!(color.x.is_finite() && color.y.is_finite() && color.z.is_finite());
        assert!(steps <= 15, "unexpected traversal count: {steps}");
    }

    #[test]
    fn zero_depth_returns_without_reflection_recursion() {
        let mut scene = Scene::new(3, 1, 1);
        scene.set(
            2,
            0,
            0,
            Some(Voxel::new(MaterialId::Water, RotationY::Deg0)),
        );
        let materials = MaterialLibrary::load_all().expect("project textures should load");
        let raytracer = Raytracer::new(&scene, &materials, Light::route_one_sun(), 0);
        let ray = Ray::new(Vec3::new(1.5, 0.5, 0.5), Vec3::new(1.0, 0.0, 0.0));
        let (_, steps) = raytracer.trace_with_steps(&ray, 0);

        assert_eq!(steps, 2);
    }

    #[test]
    fn reflective_water_blends_the_sky() {
        let mut scene = Scene::new(1, 1, 1);
        scene.set(
            0,
            0,
            0,
            Some(Voxel::new(MaterialId::Water, RotationY::Deg0)),
        );
        let materials = MaterialLibrary::load_all().expect("project textures should load");
        let raytracer = Raytracer::new(&scene, &materials, Light::route_one_sun(), 1);
        let ray = Ray::new(Vec3::new(0.5, 2.0, 0.5), Vec3::new(0.0, -1.0, 0.0));

        let local_only = raytracer.trace(&ray, 0);
        let reflected = raytracer.trace(&ray, 1);

        assert!((reflected - local_only).length() > 0.01);
    }
}
