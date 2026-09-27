use crate::core::Vec3;

#[derive(Clone, Debug)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Vec3>,
}

impl Texture {
    pub fn solid(color: Vec3) -> Self {
        Self {
            width: 1,
            height: 1,
            pixels: vec![color],
        }
    }
    pub fn sample(&self, u: f32, v: f32) -> Vec3 {
        let x = ((u.rem_euclid(1.0) * self.width as f32) as u32).min(self.width - 1);
        let y = ((v.rem_euclid(1.0) * self.height as f32) as u32).min(self.height - 1);
        self.pixels[(x + y * self.width) as usize]
    }
}
