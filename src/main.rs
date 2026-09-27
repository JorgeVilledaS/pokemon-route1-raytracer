// Los módulos de prompts futuros están declarados desde el inicio como scaffold.
#![allow(dead_code, unused_imports)]

mod core;
mod materials;
mod parallel;
mod platform;
mod render;
mod scene;
mod utils;

use crate::core::Vec3;
use crate::materials::MaterialId;
use crate::scene::{RotationY, Scene, Voxel};
use std::io;

fn main() -> io::Result<()> {
    let scene = build_test_scene();
    let target = Vec3::new(12.0, 1.5, 12.0);
    platform::run(scene, target)
}

fn build_test_scene() -> Scene {
    let mut scene = Scene::new(24, 8, 24);
    let grass = Voxel::new(MaterialId::Grass, RotationY::Deg0);

    for z in 8..16 {
        for x in 8..16 {
            scene.set(x, 0, z, Some(grass));
        }
    }

    scene.set(
        12,
        1,
        12,
        Some(Voxel::new(MaterialId::Wood, RotationY::Deg0)),
    );
    scene.set(
        12,
        2,
        12,
        Some(Voxel::new(MaterialId::Wood, RotationY::Deg90)),
    );
    scene.set(
        12,
        3,
        12,
        Some(Voxel::new(MaterialId::Leaves, RotationY::Deg0)),
    );

    scene
}
