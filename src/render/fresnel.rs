pub fn schlick(cos_theta: f64, ior: f64) -> f64 {
    let r0 = ((1.0 - ior) / (1.0 + ior)).powi(2);
    r0 + (1.0 - r0) * (1.0 - cos_theta.clamp(0.0, 1.0)).powi(5)
}

#[cfg(test)]
mod tests {
    use super::schlick;

    #[test]
    fn reflectance_increases_at_grazing_angles() {
        let straight_on = schlick(1.0, 1.33);
        let grazing = schlick(0.05, 1.33);

        assert!((straight_on - 0.020_059_312).abs() < 1.0e-8);
        assert!(grazing > straight_on);
        assert!(grazing < 1.0);
    }
}
