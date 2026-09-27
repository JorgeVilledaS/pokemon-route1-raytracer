use crate::core::Vec3;

#[derive(Default)]
pub struct Skybox;
impl Skybox {
    pub fn sample(&self, direction: Vec3) -> Vec3 {
        let t = 0.5 * (direction.normalized().y + 1.0);
        Vec3::new(0.88, 0.94, 1.0) * (1.0 - t) + Vec3::new(0.35, 0.60, 0.95) * t
    }
}
