use super::{image_io, Vec3};
use std::{io, path::Path};

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Vec3>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![Vec3::default(); width * height],
        }
    }
    pub fn clear(&mut self, color: Vec3) {
        self.pixels.fill(color);
    }
    pub fn save_ppm(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref().to_str().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "path must be valid UTF-8")
        })?;
        image_io::write_ppm(path, self.width, self.height, &self.pixels)
    }
}
