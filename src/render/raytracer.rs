use crate::{
    core::{Ray, Vec3},
    scene::Scene,
};

pub struct Raytracer {
    pub max_depth: u8,
}
impl Raytracer {
    pub const fn new(max_depth: u8) -> Self {
        Self { max_depth }
    }
    pub fn trace(&self, _scene: &Scene, ray: Ray, _depth: u8) -> Vec3 {
        super::skybox::Skybox.sample(ray.dir)
    }
}
