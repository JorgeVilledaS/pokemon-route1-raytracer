use crate::core::Vec3;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SceneMode {
    #[default]
    Day,
    Night,
}

pub struct Skybox {
    pub mode: SceneMode,
}

impl Skybox {
    pub const fn new(mode: SceneMode) -> Self {
        Self { mode }
    }

    pub fn sample(&self, direction: Vec3) -> Vec3 {
        let t = 0.5 * (direction.normalize().y + 1.0);
        match self.mode {
            SceneMode::Day => {
                Vec3::new(0.88, 0.94, 1.0) * (1.0 - t) + Vec3::new(0.35, 0.60, 0.95) * t
            }
            SceneMode::Night => {
                Vec3::new(0.035, 0.055, 0.11) * (1.0 - t) + Vec3::new(0.004, 0.008, 0.028) * t
            }
        }
    }
}
