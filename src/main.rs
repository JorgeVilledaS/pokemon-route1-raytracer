// Los módulos de prompts futuros están declarados desde el inicio como scaffold.
#![allow(dead_code, unused_imports)]

mod core;
mod materials;
mod parallel;
mod platform;
mod render;
mod scene;
mod utils;

use crate::core::{write_ppm, Camera, Vec3};
use crate::materials::{MaterialId, MaterialLibrary};
use crate::render::{CpuRenderer, SceneMode};
use crate::scene::{RotationY, Scene, Voxel};
use std::io;

fn main() -> io::Result<()> {
    let scene = build_test_scene();
    let materials = MaterialLibrary::load_all()?;
    let target = Vec3::new(12.0, 2.5, 12.0);

    let arguments: Vec<_> = std::env::args().collect();
    let mode = if arguments.iter().any(|argument| argument == "--night") {
        SceneMode::Night
    } else {
        SceneMode::Day
    };
    if arguments.iter().any(|argument| argument == "--preview") {
        return render_preview(&scene, target, &materials, mode);
    }

    platform::run(scene, target, materials)
}

fn render_preview(
    scene: &Scene,
    target: Vec3,
    materials: &MaterialLibrary,
    mode: SceneMode,
) -> io::Result<()> {
    const WIDTH: usize = 640;
    const HEIGHT: usize = 360;

    let renderer = CpuRenderer::new(WIDTH, HEIGHT);
    let camera = Camera::new(
        target,
        45.0_f64.to_radians(),
        28.0_f64.to_radians(),
        12.0,
        60.0,
        WIDTH as f64 / HEIGHT as f64,
    );
    let frame = renderer.render(scene, &camera, materials, mode);
    let pixels: Vec<_> = frame.pixels.iter().copied().map(bgrx_to_linear).collect();
    let output_path = match mode {
        SceneMode::Day => "output/frames/material_preview.ppm",
        SceneMode::Night => "output/frames/night_preview.ppm",
    };

    write_ppm(output_path, WIDTH, HEIGHT, &pixels)?;
    println!(
        "Material preview written to {output_path} in {:.1} ms",
        frame.elapsed.as_secs_f64() * 1_000.0
    );
    Ok(())
}

fn bgrx_to_linear(pixel: u32) -> Vec3 {
    let red = f64::from((pixel >> 16) as u8) / 255.0;
    let green = f64::from((pixel >> 8) as u8) / 255.0;
    let blue = f64::from(pixel as u8) / 255.0;

    Vec3::new(red.powf(2.2), green.powf(2.2), blue.powf(2.2))
}

fn build_test_scene() -> Scene {
    let mut scene = Scene::new(24, 8, 24);

    // Prado 8x8 con un camino claro que cruza la escena.
    for z in 8..16 {
        for x in 8..16 {
            let material = if z == 11 || z == 12 {
                MaterialId::Dirt
            } else {
                MaterialId::Grass
            };
            scene.set(x, 0, z, Some(Voxel::new(MaterialId::Dirt, RotationY::Deg0)));
            scene.set(x, 1, z, Some(Voxel::new(material, RotationY::Deg0)));
        }
    }

    // Estanque 3x3 con fondo alternado para hacer visible la refracción.
    for z in 13..16 {
        for x in 9..12 {
            let bottom = if (x + z) % 2 == 0 {
                MaterialId::Dirt
            } else {
                MaterialId::Rock
            };
            scene.set(x, 0, z, Some(Voxel::new(bottom, RotationY::Deg0)));
            scene.set(
                x,
                1,
                z,
                Some(Voxel::new(MaterialId::Water, RotationY::Deg0)),
            );
        }
    }

    // Árbol de prueba: tronco veteado y copa verde.
    scene.set(
        12,
        2,
        13,
        Some(Voxel::new(MaterialId::Wood, RotationY::Deg0)),
    );
    scene.set(
        12,
        3,
        13,
        Some(Voxel::new(MaterialId::Wood, RotationY::Deg90)),
    );
    scene.set(
        12,
        4,
        13,
        Some(Voxel::new(MaterialId::Grass, RotationY::Deg0)),
    );

    // Rocas y letrero hacen visibles los materiales restantes desde el inicio.
    scene.set(
        14,
        2,
        14,
        Some(Voxel::new(MaterialId::Rock, RotationY::Deg90)),
    );
    scene.set(
        15,
        2,
        14,
        Some(Voxel::new(MaterialId::Rock, RotationY::Deg180)),
    );
    scene.set(
        8,
        2,
        12,
        Some(Voxel::new(MaterialId::Sign, RotationY::Deg0)),
    );

    scene
}
