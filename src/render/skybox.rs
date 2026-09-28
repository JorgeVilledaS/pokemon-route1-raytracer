use crate::core::Vec3;
use crate::scene::value_noise_2d;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SceneMode {
    #[default]
    Day,
    Night,
}

/// Cielo procedural puro: el resultado depende únicamente de la dirección y
/// del modo. Por eso sirve por igual para rayos primarios y secundarios.
pub fn sample_skybox(direction: Vec3, mode: SceneMode) -> Vec3 {
    let dir = direction.normalize();
    // El remapeo hemisférico mantiene variación detrás del diorama incluso
    // cuando una cámara elevada lanza rayos de fondo algo descendentes.
    let height = (dir.y * 0.5 + 0.5).clamp(0.0, 1.0).powf(0.72);

    match mode {
        SceneMode::Day => {
            let sky = day_gradient(height);
            let cloud = cloud_density(dir);
            sky * (1.0 - cloud) + Vec3::new(1.0, 0.99, 0.96) * cloud
        }
        SceneMode::Night => {
            let horizon = Vec3::new(0.045, 0.065, 0.14);
            let zenith = Vec3::new(0.004, 0.008, 0.032);
            horizon * (1.0 - height) + zenith * height
        }
    }
}

fn day_gradient(height: f64) -> Vec3 {
    let horizon = Vec3::new(0.72, 0.89, 1.0);
    let zenith = Vec3::new(0.10, 0.35, 0.82);
    horizon * (1.0 - height) + zenith * height
}

fn cloud_density(dir: Vec3) -> f64 {
    // Proyección sobre un plano de cielo. El denominador mantiene el patrón
    // estable cerca del horizonte sin introducir una singularidad.
    let projection = 1.0 / (dir.y.abs() + 0.32);
    let x = dir.x * projection * 2.7;
    let z = dir.z * projection * 2.7;
    let broad = value_noise_2d(x, z, 0x4b41_4e54);
    let detail = value_noise_2d(x * 2.1 + 13.7, z * 2.1 - 8.2, 0x434c_4f55);
    let noise = broad * 0.72 + detail * 0.28;

    let shape = smoothstep(0.50, 0.68, noise);
    let horizon_fade = smoothstep(-0.82, -0.18, dir.y);
    shape * horizon_fade * (0.82 - 0.22 * dir.y.clamp(0.0, 1.0))
}

fn smoothstep(edge0: f64, edge1: f64, value: f64) -> f64 {
    let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::{cloud_density, day_gradient, sample_skybox, SceneMode};
    use crate::core::Vec3;

    #[test]
    fn day_has_a_vertical_gradient() {
        let horizon = day_gradient(0.0);
        let zenith = day_gradient(1.0);
        assert!(horizon.x > zenith.x);
        assert!(horizon.y > zenith.y);
    }

    #[test]
    fn cloud_field_has_soft_spatial_variation() {
        let mut minimum: f64 = 1.0;
        let mut maximum: f64 = 0.0;
        for index in 0..32 {
            let angle = f64::from(index) * std::f64::consts::TAU / 32.0;
            let direction = Vec3::new(angle.cos(), 0.35, angle.sin()).normalize();
            let density = cloud_density(direction);
            minimum = minimum.min(density);
            maximum = maximum.max(density);
        }
        assert!(maximum - minimum > 0.15);
    }

    #[test]
    fn night_is_darker_and_sampling_is_deterministic() {
        let direction = Vec3::new(0.4, 0.6, -0.2);
        let day = sample_skybox(direction, SceneMode::Day);
        let night = sample_skybox(direction, SceneMode::Night);
        assert_eq!(day, sample_skybox(direction, SceneMode::Day));
        assert!(night.length() < day.length() * 0.25);
    }
}
