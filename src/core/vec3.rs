use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
    pub fn dot(self, rhs: Self) -> f32 { self.x * rhs.x + self.y * rhs.y + self.z * rhs.z }
    pub fn cross(self, rhs: Self) -> Self {
        Self::new(self.y * rhs.z - self.z * rhs.y, self.z * rhs.x - self.x * rhs.z, self.x * rhs.y - self.y * rhs.x)
    }
    pub fn length(self) -> f32 { self.dot(self).sqrt() }
    pub fn normalized(self) -> Self { let len = self.length(); if len > 0.0 { self / len } else { self } }
    pub fn reflect(self, normal: Self) -> Self { self - normal * (2.0 * self.dot(normal)) }
    pub fn clamp(self, min: f32, max: f32) -> Self {
        Self::new(self.x.clamp(min, max), self.y.clamp(min, max), self.z.clamp(min, max))
    }
}

impl Add for Vec3 { type Output = Self; fn add(self, r: Self) -> Self { Self::new(self.x+r.x, self.y+r.y, self.z+r.z) } }
impl Sub for Vec3 { type Output = Self; fn sub(self, r: Self) -> Self { Self::new(self.x-r.x, self.y-r.y, self.z-r.z) } }
impl Mul<f32> for Vec3 { type Output = Self; fn mul(self, r: f32) -> Self { Self::new(self.x*r, self.y*r, self.z*r) } }
impl Div<f32> for Vec3 { type Output = Self; fn div(self, r: f32) -> Self { self * (1.0/r) } }
impl Neg for Vec3 { type Output = Self; fn neg(self) -> Self { Self::new(-self.x, -self.y, -self.z) } }
