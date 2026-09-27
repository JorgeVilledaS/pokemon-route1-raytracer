use crate::core::{Ray, Vec3};
use crate::materials::MaterialId;
use crate::scene::Scene;

#[derive(Clone, Copy, Debug)]
pub struct Hit { pub distance: f32, pub point: Vec3, pub normal: Vec3, pub uv: (f32, f32), pub material: MaterialId }

pub fn intersect(_scene: &Scene, _ray: Ray) -> Option<Hit> {
    // DDA se implementará en el prompt correspondiente.
    None
}
