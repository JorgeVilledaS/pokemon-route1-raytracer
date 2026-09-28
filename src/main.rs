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
use crate::scene::{generate, voxel_index, RotationY, Scene, Voxel, GENERATED_HEIGHT};
use std::io;
use std::time::{Duration, Instant};

fn main() -> io::Result<()> {
    let (scene, terrain_elapsed) = build_test_scene();
    println!(
        "Terrain 16x16 generated in {:.3} ms",
        terrain_elapsed.as_secs_f64() * 1_000.0
    );
    let materials = MaterialLibrary::load_all()?;
    let target = Vec3::new(12.0, 2.5, 12.0);

    let arguments: Vec<_> = std::env::args().collect();
    let mode = if arguments.iter().any(|argument| argument == "--night") {
        SceneMode::Night
    } else {
        SceneMode::Day
    };
    if arguments.iter().any(|argument| argument == "--benchmark") {
        benchmark_render(&scene, target, &materials, mode);
        return Ok(());
    }
    if arguments.iter().any(|argument| argument == "--preview") {
        return render_preview(&scene, target, &materials, mode);
    }

    platform::run(scene, target, materials)
}

fn benchmark_render(scene: &Scene, target: Vec3, materials: &MaterialLibrary, mode: SceneMode) {
    const WIDTH: usize = 640;
    const HEIGHT: usize = 360;
    const RUNS: u32 = 5;

    let camera = Camera::new(
        target,
        45.0_f64.to_radians(),
        28.0_f64.to_radians(),
        23.0,
        60.0,
        WIDTH as f64 / HEIGHT as f64,
    );
    let serial = CpuRenderer::single_threaded(WIDTH, HEIGHT);
    let parallel = CpuRenderer::new(WIDTH, HEIGHT);
    let mut serial_pixels = vec![0; WIDTH * HEIGHT];
    let mut parallel_pixels = vec![0; WIDTH * HEIGHT];

    // Calentamiento: evita atribuir inicialización de páginas al primer modo.
    serial.render_into(&mut serial_pixels, scene, &camera, materials, mode);
    parallel.render_into(&mut parallel_pixels, scene, &camera, materials, mode);

    let mut serial_total = Duration::ZERO;
    let mut parallel_total = Duration::ZERO;
    for _ in 0..RUNS {
        serial_total += serial
            .render_into(&mut serial_pixels, scene, &camera, materials, mode)
            .elapsed;
        parallel_total += parallel
            .render_into(&mut parallel_pixels, scene, &camera, materials, mode)
            .elapsed;
    }

    assert_eq!(
        serial_pixels, parallel_pixels,
        "serial and parallel frames differ"
    );
    let serial_ms = serial_total.as_secs_f64() * 1_000.0 / f64::from(RUNS);
    let parallel_ms = parallel_total.as_secs_f64() * 1_000.0 / f64::from(RUNS);
    println!("Benchmark 640x360, average of {RUNS} measured frames:");
    println!("  1 worker : {serial_ms:.2} ms");
    println!("  {} workers: {parallel_ms:.2} ms", parallel.worker_count());
    println!("  speedup  : {:.2}x", serial_ms / parallel_ms);
    println!("  equality : byte-identical framebuffer");
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
        23.0,
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

fn build_test_scene() -> (Scene, Duration) {
    let mut scene = Scene::new(24, 8, 24);
    const TERRAIN_SIZE: usize = 16;
    const TERRAIN_ORIGIN: i32 = 4;

    let generation_started = Instant::now();
    let terrain = generate(0x524f_5554_4531, TERRAIN_SIZE, TERRAIN_SIZE);
    let terrain_elapsed = generation_started.elapsed();

    // Región procedural 16x16 centrada dentro de la escena 24x24.
    for z in 0..TERRAIN_SIZE {
        for y in 0..GENERATED_HEIGHT {
            for x in 0..TERRAIN_SIZE {
                let voxel = terrain[voxel_index(TERRAIN_SIZE, x, y, z)];
                if voxel.material != MaterialId::Air {
                    scene.set(
                        TERRAIN_ORIGIN + x as i32,
                        y as i32,
                        TERRAIN_ORIGIN + z as i32,
                        Some(voxel),
                    );
                }
            }
        }
    }

    // Estanque 3x3 con fondo alternado para hacer visible la refracción.
    for z in 13..16 {
        for x in 9..12 {
            for y in 2..GENERATED_HEIGHT as i32 {
                scene.set(x, y, z, None);
            }
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

    (scene, terrain_elapsed)
}
