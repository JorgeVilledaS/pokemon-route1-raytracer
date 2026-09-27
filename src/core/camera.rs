use super::{Ray, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct Camera { pub target: Vec3, pub yaw: f32, pub pitch: f32, pub distance: f32, pub fov_y: f32 }

impl Camera {
    pub fn new(target: Vec3, yaw: f32, pitch: f32, distance: f32, fov_y: f32) -> Self {
        Self { target, yaw, pitch, distance, fov_y }
    }
    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw += delta_yaw;
        self.pitch = (self.pitch + delta_pitch).clamp(-1.5, 1.5);
    }
    pub fn dolly(&mut self, delta: f32) { self.distance = (self.distance + delta).max(0.25); }
    pub fn ray_for_pixel(&self, _x: u32, _y: u32, _width: u32, _height: u32) -> Ray {
        // La proyección completa se implementará en el prompt de cámara.
        Ray::new(self.target + Vec3::new(0.0, 0.0, self.distance), Vec3::new(0.0, 0.0, -1.0))
    }
}
