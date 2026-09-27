mod core;
mod materials;
mod parallel;
mod render;
mod scene;
mod utils;

use crate::core::{Framebuffer, Vec3};
use std::io;

fn main() -> io::Result<()> {
    let mut framebuffer = Framebuffer::new(320, 180);
    framebuffer.clear(Vec3::new(0.45, 0.68, 0.95));
    framebuffer.save_ppm("output/preview.ppm")?;

    println!("Scaffold listo: output/preview.ppm");
    Ok(())
}
