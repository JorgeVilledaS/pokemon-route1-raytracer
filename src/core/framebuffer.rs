use super::{image_io, Vec3};
use std::{io, path::Path};

pub struct Framebuffer { pub width: u32, pub height: u32, pub pixels: Vec<Vec3> }

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self { Self { width, height, pixels: vec![Vec3::default(); (width * height) as usize] } }
    pub fn clear(&mut self, color: Vec3) { self.pixels.fill(color); }
    pub fn save_ppm(&self, path: impl AsRef<Path>) -> io::Result<()> { image_io::write_ppm(path, self.width, self.height, &self.pixels) }
}
