use super::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3,
}

impl Ray {
    pub const fn new(origin: Vec3, dir: Vec3) -> Self {
        Self { origin, dir }
    }

    pub fn at(&self, t: f64) -> Vec3 {
        self.origin + self.dir * t
    }
}

#[cfg(test)]
mod tests {
    use super::{Ray, Vec3};

    #[test]
    fn point_along_ray() {
        let ray = Ray::new(Vec3::new(1.0, 2.0, 3.0), Vec3::new(0.0, 1.0, 0.0));

        assert_eq!(ray.at(2.5), Vec3::new(1.0, 4.5, 3.0));
    }
}
