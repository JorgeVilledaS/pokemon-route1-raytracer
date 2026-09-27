// Los módulos de prompts futuros están declarados desde el inicio como scaffold.
#![allow(dead_code, unused_imports)]

mod core;
mod materials;
mod parallel;
mod render;
mod scene;
mod utils;

use crate::core::{write_ppm, Camera, Ray, Vec3};
use std::io;

const WIDTH: usize = 400;
const HEIGHT: usize = 300;
const FRAME_COUNT: usize = 10;
const SKY_COLOR: Vec3 = Vec3::new(0.18, 0.38, 0.72);

fn main() -> io::Result<()> {
    let target = Vec3::default();

    println!("frame,t,yaw,distance,position");
    for frame in 0..FRAME_COUNT {
        let t = frame as f64 / (FRAME_COUNT - 1) as f64;
        let camera = Camera::at_time(t, target);
        let pixels = render_empty_scene(&camera);
        let output_path = format!("output/frames/camera_{frame:02}.ppm");

        write_ppm(&output_path, WIDTH, HEIGHT, &pixels)?;

        let position = camera.position();
        println!(
            "{frame},{t:.3},{:.6},{:.6},({:.3} {:.3} {:.3})",
            camera.yaw, camera.distance, position.x, position.y, position.z
        );
    }

    Ok(())
}

fn render_empty_scene(camera: &Camera) -> Vec<Vec3> {
    let mut pixels = Vec::with_capacity(WIDTH * HEIGHT);

    for y in 0..HEIGHT {
        let v = 1.0 - 2.0 * (y as f64 + 0.5) / HEIGHT as f64;
        for x in 0..WIDTH {
            let u = 2.0 * (x as f64 + 0.5) / WIDTH as f64 - 1.0;
            let ray = camera.get_ray(u, v);
            pixels.push(sample_solid_skybox(ray));
        }
    }

    pixels
}

fn sample_solid_skybox(_ray: Ray) -> Vec3 {
    SKY_COLOR
}
