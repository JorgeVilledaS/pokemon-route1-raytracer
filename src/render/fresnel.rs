pub fn schlick(cosine: f32, ior_from: f32, ior_to: f32) -> f32 {
    let r0 = ((ior_from - ior_to) / (ior_from + ior_to)).powi(2);
    r0 + (1.0 - r0) * (1.0 - cosine.clamp(0.0, 1.0)).powi(5)
}
