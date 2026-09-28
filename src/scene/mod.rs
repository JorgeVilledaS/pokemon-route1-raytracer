mod noise;
#[allow(clippy::module_inception)]
mod scene;
mod terrain_generator;
mod voxel;

pub(crate) use noise::value_noise_2d;
pub use scene::Scene;
pub use terrain_generator::{generate, voxel_index, GENERATED_HEIGHT};
pub use voxel::{RotationY, Voxel};
