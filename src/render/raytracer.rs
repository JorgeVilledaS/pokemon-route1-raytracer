use super::fresnel::schlick;
use super::shading::{shade, Light, RAY_BIAS};
use super::{intersect_with_steps, sample_skybox, SceneMode};
use crate::core::{Ray, Vec3};
use crate::materials::MaterialLibrary;
use crate::scene::Scene;

pub struct Raytracer<'a> {
    scene: &'a Scene,
    materials: &'a MaterialLibrary,
    light: Light,
    mode: SceneMode,
    pub max_depth: u32,
}

impl<'a> Raytracer<'a> {
    pub fn new(
        scene: &'a Scene,
        materials: &'a MaterialLibrary,
        mode: SceneMode,
        max_depth: u32,
    ) -> Self {
        let light = Light::for_mode(mode);
        Self {
            scene,
            materials,
            light,
            mode,
            max_depth,
        }
    }

    pub fn trace(&self, ray: &Ray, depth: u32) -> Vec3 {
        self.trace_with_steps(ray, depth).0
    }

    pub fn trace_with_steps(&self, ray: &Ray, depth: u32) -> (Vec3, usize) {
        let (hit, mut steps) = intersect_with_steps(self.scene, ray);
        let mut closest = hit.map_or(f64::INFINITY, |h| (h.point - ray.origin).length());
        let mut actor_color = None;
        for actor in self.scene.actor.iter().chain(self.scene.encounters.iter()) {
            if let Some((t, n, face, u, v)) = super::avatar::hit(ray, *actor) {
                if t * ray.dir.length() < closest {
                    closest = t * ray.dir.length();
                    let c = self.materials.avatars.skins[actor.skin][face].sample(u, v);
                    let light = (0.42 + 0.58 * n.dot(self.light.direction).max(0.0))
                        * if self.mode == SceneMode::Night {
                            0.3
                        } else {
                            1.0
                        };
                    actor_color = Some(c * light);
                }
            }
        }
        if let Some(color) = actor_color {
            return (color, steps);
        }
        let Some(hit) = hit else {
            return (sample_skybox(ray.dir, self.mode), steps);
        };

        let material = self.materials.get(hit.material);
        let local_color = shade(&hit, material, &self.light, self.scene, -ray.dir);
        if depth == 0 {
            return (local_color, steps);
        }

        let cos_theta = (-ray.dir.normalize()).dot(hit.normal).abs().min(1.0);
        let fresnel = if material.kt > 0.0 {
            schlick(cos_theta, material.ior)
        } else {
            0.0
        };
        let mut reflection_weight = if material.kt > 0.0 {
            material.kr + (1.0 - material.kr) * fresnel
        } else {
            material.kr
        };
        let mut refraction_weight = material.kt * (1.0 - reflection_weight);

        let refracted_ray = if refraction_weight > 0.0 {
            // `Option` representa de forma explícita la reflexión total interna.
            match self.refracted_through_voxel(&hit, ray.dir) {
                Some(refracted) => Some(refracted),
                None => {
                    reflection_weight += refraction_weight;
                    refraction_weight = 0.0;
                    None
                }
            }
        } else {
            None
        };

        let (reflected_color, reflected_steps) = if reflection_weight > 0.0 {
            let reflected_direction = ray.dir.reflect(hit.normal).normalize();
            let bias_normal = if reflected_direction.dot(hit.normal) >= 0.0 {
                hit.normal
            } else {
                -hit.normal
            };
            let reflected_ray = Ray::new(hit.point + bias_normal * RAY_BIAS, reflected_direction);
            self.trace_with_steps(&reflected_ray, depth - 1)
        } else {
            (Vec3::default(), 0)
        };
        steps += reflected_steps;

        let (refracted_color, refracted_steps) = if let Some(refracted_ray) = refracted_ray {
            self.trace_with_steps(&refracted_ray, depth - 1)
        } else {
            (Vec3::default(), 0)
        };
        steps += refracted_steps;

        let local_weight = (1.0 - reflection_weight - refraction_weight).max(0.0);
        (
            local_color * local_weight
                + reflected_color * reflection_weight
                + refracted_color * refraction_weight,
            steps,
        )
    }

    fn refracted_through_voxel(&self, hit: &super::Hit, incident: Vec3) -> Option<Ray> {
        let incident = incident.normalize();
        let entry_normal = if incident.dot(hit.normal) < 0.0 {
            hit.normal
        } else {
            -hit.normal
        };
        let inside_direction =
            incident.refract(entry_normal, 1.0 / self.materials.get(hit.material).ior)?;
        let (exit_point, exit_normal) = self.exit_material_volume(hit, inside_direction)?;
        let outside_direction =
            inside_direction.refract(-exit_normal, self.materials.get(hit.material).ior)?;

        Some(Ray::new(
            exit_point + outside_direction * RAY_BIAS,
            outside_direction.normalize(),
        ))
    }

    fn exit_material_volume(&self, hit: &super::Hit, direction: Vec3) -> Option<(Vec3, Vec3)> {
        let mut cell = hit.cell;
        let mut point = hit.point + direction * RAY_BIAS;
        let max_cells = self.scene.width + self.scene.height + self.scene.depth;

        for _ in 0..max_cells {
            let (exit_point, exit_normal) = exit_cell(point, direction, cell)?;
            let beyond = exit_point + direction * RAY_BIAS;
            let next_cell = (
                beyond.x.floor() as i32,
                beyond.y.floor() as i32,
                beyond.z.floor() as i32,
            );

            if self
                .scene
                .get(next_cell.0, next_cell.1, next_cell.2)
                .is_some_and(|voxel| voxel.material == hit.material)
            {
                cell = next_cell;
                point = beyond;
            } else {
                return Some((exit_point, exit_normal));
            }
        }

        None
    }
}

fn exit_cell(point: Vec3, direction: Vec3, cell: (i32, i32, i32)) -> Option<(Vec3, Vec3)> {
    let min = Vec3::new(f64::from(cell.0), f64::from(cell.1), f64::from(cell.2));
    let max = min + Vec3::new(1.0, 1.0, 1.0);
    let mut best_t = f64::INFINITY;
    let mut normal = Vec3::default();

    for (distance, candidate_normal) in [
        axis_exit(point.x, direction.x, min.x, max.x, Vec3::new(1.0, 0.0, 0.0)),
        axis_exit(point.y, direction.y, min.y, max.y, Vec3::new(0.0, 1.0, 0.0)),
        axis_exit(point.z, direction.z, min.z, max.z, Vec3::new(0.0, 0.0, 1.0)),
    ] {
        if distance > 0.0 && distance < best_t {
            best_t = distance;
            normal = candidate_normal;
        }
    }

    best_t
        .is_finite()
        .then_some((point + direction * best_t, normal))
}

fn axis_exit(point: f64, direction: f64, min: f64, max: f64, axis: Vec3) -> (f64, Vec3) {
    if direction > 1.0e-12 {
        ((max - point) / direction, axis)
    } else if direction < -1.0e-12 {
        ((min - point) / direction, -axis)
    } else {
        (f64::INFINITY, Vec3::default())
    }
}

#[cfg(test)]
mod tests {
    use super::Raytracer;
    use crate::core::{Ray, Vec3};
    use crate::materials::{MaterialId, MaterialLibrary};
    use crate::render::SceneMode;
    use crate::scene::{RotationY, Scene, Voxel};

    #[test]
    fn explicit_depth_stops_between_two_reflective_surfaces() {
        let mut scene = Scene::new(3, 1, 1);
        let mirror = Voxel::new(MaterialId::Water, RotationY::Deg0);
        scene.set(0, 0, 0, Some(mirror));
        scene.set(2, 0, 0, Some(mirror));

        let materials = MaterialLibrary::load_all().expect("project textures should load");
        let raytracer = Raytracer::new(&scene, &materials, SceneMode::Day, 4);
        let ray = Ray::new(Vec3::new(1.5, 0.5, 0.5), Vec3::new(1.0, 0.0, 0.0));
        let (color, steps) = raytracer.trace_with_steps(&ray, 4);

        assert!(color.x.is_finite() && color.y.is_finite() && color.z.is_finite());
        assert!(steps < 100, "unexpected traversal count: {steps}");
    }

    #[test]
    fn zero_depth_returns_without_secondary_rays() {
        let mut scene = Scene::new(3, 1, 1);
        scene.set(
            2,
            0,
            0,
            Some(Voxel::new(MaterialId::Water, RotationY::Deg0)),
        );
        let materials = MaterialLibrary::load_all().expect("project textures should load");
        let raytracer = Raytracer::new(&scene, &materials, SceneMode::Day, 0);
        let ray = Ray::new(Vec3::new(1.5, 0.5, 0.5), Vec3::new(1.0, 0.0, 0.0));
        let (_, steps) = raytracer.trace_with_steps(&ray, 0);

        assert_eq!(steps, 2);
    }

    #[test]
    fn water_refracts_to_the_voxel_below() {
        let mut scene = Scene::new(1, 2, 1);
        scene.set(0, 0, 0, Some(Voxel::new(MaterialId::Dirt, RotationY::Deg0)));
        scene.set(
            0,
            1,
            0,
            Some(Voxel::new(MaterialId::Water, RotationY::Deg0)),
        );
        let materials = MaterialLibrary::load_all().expect("project textures should load");
        let raytracer = Raytracer::new(&scene, &materials, SceneMode::Day, 2);
        let ray = Ray::new(
            Vec3::new(0.25, 3.0, 0.5),
            Vec3::new(0.12, -1.0, 0.0).normalize(),
        );

        let color = raytracer.trace(&ray, 2);
        let sky = super::sample_skybox(Vec3::new(0.0, 1.0, 0.0), SceneMode::Day);

        assert!(color.x.is_finite() && color.y.is_finite() && color.z.is_finite());
        assert!((color - sky).length() > 0.1);
    }
}
