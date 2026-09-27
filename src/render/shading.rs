use crate::core::Vec3;
use crate::materials::Material;

pub fn shade(material: &Material, uv: (f64, f64), normal: Vec3, view_direction: Vec3) -> Vec3 {
    let light_direction = Vec3::new(-0.4, 0.9, 0.25).normalize();
    let diffuse = normal.dot(light_direction).max(0.0);
    let half_vector = (light_direction + view_direction).normalize();
    let specular = material.ks * normal.dot(half_vector).max(0.0).powf(material.shininess);
    let albedo = material.albedo.sample(uv.0, uv.1);

    albedo * (0.3 + 0.7 * diffuse) + Vec3::new(1.0, 1.0, 1.0) * specular + material.emissive
}
