use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn dot(&self, other: Vec3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn length(&self) -> f64 {
        self.dot(*self).sqrt()
    }

    pub fn normalize(&self) -> Vec3 {
        let length = self.length();
        if length > 0.0 {
            *self / length
        } else {
            *self
        }
    }

    pub fn reflect(&self, n: Vec3) -> Vec3 {
        *self - n * (2.0 * self.dot(n))
    }

    /// Refracta un vector incidente unitario usando una normal unitaria.
    /// `eta` es la razón entre los índices de refracción (n1 / n2).
    pub fn refract(&self, n: Vec3, eta: f64) -> Option<Vec3> {
        let cos_theta = (-*self).dot(n).min(1.0);
        let perpendicular = (*self + n * cos_theta) * eta;
        let parallel_squared = 1.0 - perpendicular.dot(perpendicular);

        if parallel_squared < 0.0 {
            None
        } else {
            Some(perpendicular - n * parallel_squared.sqrt())
        }
    }

    pub fn clamp(&self, min: f64, max: f64) -> Vec3 {
        Vec3::new(
            self.x.clamp(min, max),
            self.y.clamp(min, max),
            self.z.clamp(min, max),
        )
    }
}

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, scalar: f64) -> Vec3 {
        Vec3::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl Div<f64> for Vec3 {
    type Output = Vec3;

    fn div(self, scalar: f64) -> Vec3 {
        self * (1.0 / scalar)
    }
}

impl Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Vec3 {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}

#[cfg(test)]
mod tests {
    use super::Vec3;

    const EPSILON: f64 = 1.0e-12;

    fn assert_vec3_close(actual: Vec3, expected: Vec3) {
        assert!((actual.x - expected.x).abs() < EPSILON);
        assert!((actual.y - expected.y).abs() < EPSILON);
        assert!((actual.z - expected.z).abs() < EPSILON);
    }

    #[test]
    fn dot_product_matches_known_result() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, -5.0, 6.0);

        assert!((a.dot(b) - 12.0).abs() < EPSILON);
    }

    #[test]
    fn cross_product_matches_known_result() {
        let x = Vec3::new(1.0, 0.0, 0.0);
        let y = Vec3::new(0.0, 1.0, 0.0);

        assert_vec3_close(x.cross(y), Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn reflection_matches_known_result() {
        let incident = Vec3::new(1.0, -1.0, 0.0).normalize();
        let normal = Vec3::new(0.0, 1.0, 0.0);

        assert_vec3_close(
            incident.reflect(normal),
            Vec3::new(1.0, 1.0, 0.0).normalize(),
        );
    }

    #[test]
    fn refraction_reports_total_internal_reflection() {
        let incident = Vec3::new(3.0_f64.sqrt() / 2.0, -0.5, 0.0);
        let normal = Vec3::new(0.0, 1.0, 0.0);

        assert_eq!(incident.refract(normal, 1.5), None);
    }
}
