use super::{Ray, Vec3};
use std::f64::consts::TAU;

const DEFAULT_PITCH: f64 = 25.0_f64.to_radians();
const DEFAULT_DISTANCE: f64 = 8.0;
const DOLLY_AMPLITUDE: f64 = 2.0;
const DEFAULT_FOV_DEG: f64 = 60.0;
const DEFAULT_ASPECT_RATIO: f64 = 4.0 / 3.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub target: Vec3,
    pub yaw: f64,
    pub pitch: f64,
    pub distance: f64,
    pub fov_deg: f64,
    pub aspect_ratio: f64,
}

impl Camera {
    pub const fn new(
        target: Vec3,
        yaw: f64,
        pitch: f64,
        distance: f64,
        fov_deg: f64,
        aspect_ratio: f64,
    ) -> Self {
        Self {
            target,
            yaw,
            pitch,
            distance,
            fov_deg,
            aspect_ratio,
        }
    }

    /// Devuelve la posición orbital. `yaw = 0` sitúa la cámara sobre +Z.
    pub fn position(&self) -> Vec3 {
        let horizontal_distance = self.distance * self.pitch.cos();
        let offset = Vec3::new(
            horizontal_distance * self.yaw.sin(),
            self.distance * self.pitch.sin(),
            horizontal_distance * self.yaw.cos(),
        );

        self.target + offset
    }

    /// Genera un rayo para coordenadas de pantalla normalizadas en [-1, 1].
    /// `u` crece hacia la derecha y `v` hacia arriba.
    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        let origin = self.position();
        let forward = (self.target - origin).normalize();

        // Evita una base degenerada si la cámara se coloca exactamente en un polo.
        let reference_up = if forward.y.abs() > 1.0 - 1.0e-12 {
            Vec3::new(0.0, 0.0, 1.0)
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        };
        let right = forward.cross(reference_up).normalize();
        let up = right.cross(forward).normalize();

        let half_height = (0.5 * self.fov_deg.to_radians()).tan();
        let half_width = self.aspect_ratio * half_height;
        let direction = (forward + right * (u * half_width) + up * (v * half_height)).normalize();

        Ray::new(origin, direction)
    }

    /// Construye una cámara para un instante normalizado de la animación.
    /// Un intervalo `t = 0..=1` completa una órbita y un ciclo de dolly.
    pub fn at_time(t: f64, target: Vec3) -> Camera {
        let progress = t.clamp(0.0, 1.0);

        Camera::new(
            target,
            TAU * progress,
            DEFAULT_PITCH,
            DEFAULT_DISTANCE + DOLLY_AMPLITUDE * (TAU * progress).sin(),
            DEFAULT_FOV_DEG,
            DEFAULT_ASPECT_RATIO,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Camera, Vec3, DEFAULT_DISTANCE, DOLLY_AMPLITUDE};
    use std::f64::consts::{FRAC_PI_2, PI, TAU};

    const EPSILON: f64 = 1.0e-12;

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < EPSILON,
            "expected {expected}, got {actual}"
        );
    }

    fn assert_vec3_close(actual: Vec3, expected: Vec3) {
        assert_close(actual.x, expected.x);
        assert_close(actual.y, expected.y);
        assert_close(actual.z, expected.z);
    }

    #[test]
    fn position_uses_spherical_orbit_coordinates() {
        let target = Vec3::new(1.0, 2.0, 3.0);
        let camera = Camera::new(target, FRAC_PI_2, 0.0, 5.0, 60.0, 1.0);

        assert_vec3_close(camera.position(), Vec3::new(6.0, 2.0, 3.0));
    }

    #[test]
    fn center_ray_points_at_target() {
        let camera = Camera::new(Vec3::default(), 0.7, 0.3, 5.0, 60.0, 16.0 / 9.0);
        let ray = camera.get_ray(0.0, 0.0);
        let expected_direction = (camera.target - camera.position()).normalize();

        assert_vec3_close(ray.origin, camera.position());
        assert_vec3_close(ray.dir, expected_direction);
        assert_close(ray.dir.length(), 1.0);
    }

    #[test]
    fn corner_rays_respect_vertical_fov_and_aspect_ratio() {
        let camera = Camera::new(Vec3::default(), 0.0, 0.0, 5.0, 90.0, 2.0);
        let top_right = camera.get_ray(1.0, 1.0).dir;

        // Con FOV vertical de 90°, half-height = 1 y half-width = aspect * 1.
        assert_close(top_right.y / -top_right.z, 1.0);
        assert_close(top_right.x / -top_right.z, 2.0);
    }

    #[test]
    fn animation_completes_orbit_and_dolly_cycle() {
        let target = Vec3::default();
        let start = Camera::at_time(0.0, target);
        let near_quarter = Camera::at_time(0.25, target);
        let halfway = Camera::at_time(0.5, target);
        let far_quarter = Camera::at_time(0.75, target);
        let end = Camera::at_time(1.0, target);

        assert_close(start.yaw, 0.0);
        assert_close(near_quarter.yaw, FRAC_PI_2);
        assert_close(halfway.yaw, PI);
        assert_close(far_quarter.yaw, 3.0 * FRAC_PI_2);
        assert_close(end.yaw, TAU);

        assert_close(start.distance, DEFAULT_DISTANCE);
        assert_close(near_quarter.distance, DEFAULT_DISTANCE + DOLLY_AMPLITUDE);
        assert_close(halfway.distance, DEFAULT_DISTANCE);
        assert_close(far_quarter.distance, DEFAULT_DISTANCE - DOLLY_AMPLITUDE);
        assert_close(end.distance, DEFAULT_DISTANCE);
    }

    #[test]
    fn rays_remain_valid_at_orbit_poles() {
        let camera = Camera::new(Vec3::default(), 0.0, FRAC_PI_2, 5.0, 60.0, 1.0);
        let ray = camera.get_ray(1.0, 1.0);

        assert_close(ray.dir.length(), 1.0);
        assert!(ray.dir.x.is_finite() && ray.dir.y.is_finite() && ray.dir.z.is_finite());
    }
}
