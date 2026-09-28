/// Ruido de valor continuo y determinista en el plano XZ.
///
/// La interpolación cúbica conserva el coste bajo del hash de rejilla, pero
/// evita que el terreno y las nubes revelen celdas cuadradas.
pub fn value_noise_2d(x: f64, z: f64, seed: u64) -> f64 {
    let x0 = x.floor() as i32;
    let z0 = z.floor() as i32;
    let tx = smooth_curve(x - x.floor());
    let tz = smooth_curve(z - z.floor());

    let lower = lerp(
        lattice_value(x0, z0, seed),
        lattice_value(x0 + 1, z0, seed),
        tx,
    );
    let upper = lerp(
        lattice_value(x0, z0 + 1, seed),
        lattice_value(x0 + 1, z0 + 1, seed),
        tx,
    );
    lerp(lower, upper, tz)
}

fn lattice_value(x: i32, z: i32, seed: u64) -> f64 {
    let mut n = (x as i64 as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ (z as i64 as u64).wrapping_mul(0xbf58_476d_1ce4_e5b9)
        ^ seed;
    n = (n ^ (n >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    n = (n ^ (n >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    n ^= n >> 31;
    (n >> 11) as f64 / ((1_u64 << 53) - 1) as f64
}

fn smooth_curve(value: f64) -> f64 {
    value * value * (3.0 - 2.0 * value)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::value_noise_2d;

    #[test]
    fn value_noise_is_deterministic_and_bounded() {
        let value = value_noise_2d(1.25, -3.75, 7);
        assert_eq!(value, value_noise_2d(1.25, -3.75, 7));
        assert!((0.0..=1.0).contains(&value));
    }

    #[test]
    fn value_noise_is_continuous_across_cell_boundaries() {
        let left = value_noise_2d(0.999, 2.4, 11);
        let right = value_noise_2d(1.001, 2.4, 11);
        assert!((left - right).abs() < 0.01);
    }
}
