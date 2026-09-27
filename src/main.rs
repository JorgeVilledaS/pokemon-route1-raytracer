// Los módulos de prompts futuros están declarados desde el inicio como scaffold.
#![allow(dead_code, unused_imports)]

mod core;
mod materials;
mod parallel;
mod render;
mod scene;
mod utils;

use crate::core::{write_ppm, Vec3};
use std::io;

const WIDTH: usize = 400;
const HEIGHT: usize = 300;

fn main() -> io::Result<()> {
    let mut pixels = Vec::with_capacity(WIDTH * HEIGHT);

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let red = x as f64 / (WIDTH - 1) as f64;
            let green = 1.0 - y as f64 / (HEIGHT - 1) as f64;
            let blue = 0.25;
            pixels.push(Vec3::new(red, green, blue));
        }
    }

    let output_path = "output/frames/test.ppm";
    write_ppm(output_path, WIDTH, HEIGHT, &pixels)?;
    println!("Gradient written to {output_path}");

    Ok(())
}
