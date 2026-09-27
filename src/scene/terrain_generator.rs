use super::{noise::value_noise_2d, Scene};

pub struct TerrainGenerator {
    pub seed: u32,
}

impl TerrainGenerator {
    pub const fn new(seed: u32) -> Self {
        Self { seed }
    }
    pub fn sample_height(&self, x: i32, z: i32) -> usize {
        1 + (value_noise_2d(x, z, self.seed) * 3.0) as usize
    }
    pub fn generate_16x16(&self) -> Scene {
        // El poblado de materiales se añadirá con la generación procedural completa.
        Scene::new(16, 8, 16)
    }
}
