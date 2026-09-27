use super::{Ray, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub target: Vec3,
    pub yaw: f64,
    pub pitch: f64,
    pub distance: f64,
    pub fov_y: f64,
}

impl Camera {
    pub fn new(target: Vec3, yaw: f64, pitch: f64, distance: f64, fov_y: f64) -> Self {
        Self {
            target,
            yaw,
            pitch,
            distance,
            fov_y,
        }
    }
    pub fn orbit(&mut self, delta_yaw: f64, delta_pitch: f64) {
        self.yaw += delta_yaw;
        self.pitch = (self.pitch + delta_pitch).clamp(-1.5, 1.5);
    }
    pub fn dolly(&mut self, delta: f64) {
        self.distance = (self.distance + delta).max(0.25);
    }
    pub fn ray_for_pixel(&self, _x: u32, _y: u32, _width: u32, _height: u32) -> Ray {
        // La proyección completa se implementará en el prompt de cámara.
        Ray::new(
            self.target + Vec3::new(0.0, 0.0, self.distance),
            Vec3::new(0.0, 0.0, -1.0),
        )
    }
}
