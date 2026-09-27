use crate::core::Vec3;

pub fn blinn_phong(albedo: Vec3, normal: Vec3, light_direction: Vec3) -> Vec3 {
    albedo * normal.dot(light_direction).max(0.0)
}
