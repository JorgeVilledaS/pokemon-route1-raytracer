use crate::{
    core::{Ray, Vec3},
    scene::adventure::Actor,
    utils::Texture,
};
use std::{io, path::Path};

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn small_avatar_has_correct_faces_and_local_uv() {
        let actor = Actor {
            position: Vec3::new(2.5, 3.0, 2.5),
            skin: 0,
        };
        let ray = Ray::new(Vec3::new(2.5, 3.31, 5.0), Vec3::new(0.0, 0.0, -1.0));
        let (t, normal, face, u, v) = hit(&ray, actor).unwrap();
        assert!((t - 2.19).abs() < 1e-9);
        assert_eq!(face, 4);
        assert_eq!(normal, Vec3::new(0.0, 0.0, 1.0));
        assert!((u - 0.5).abs() < 1e-9 && (v - 0.5).abs() < 1e-9);
        let miss = Ray::new(Vec3::new(2.9, 3.31, 5.0), Vec3::new(0.0, 0.0, -1.0));
        assert!(hit(&miss, actor).is_none());
    }
}

pub struct AvatarTextures {
    pub skins: Vec<Vec<Texture>>,
}
pub const FACES: [&str; 6] = ["right", "left", "top", "bottom", "front", "back"];
impl AvatarTextures {
    pub fn load() -> io::Result<Self> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/textures/characters");
        let mut skins = Vec::new();
        for skin in 0..4 {
            let name = if skin == 0 {
                "pokeball".to_string()
            } else {
                format!("starter_{skin}")
            };
            let base = root.join(format!("{name}.ppm"));
            let base = if base.exists() {
                Some(Texture::load_ppm(base)?)
            } else {
                None
            };
            let mut faces = Vec::new();
            for (face, suffix) in FACES.iter().enumerate() {
                let path = root.join(format!("{name}_{suffix}.ppm"));
                faces.push(if path.exists() {
                    Texture::load_ppm(path)?
                } else {
                    base.clone().unwrap_or_else(|| placeholder(skin, face))
                });
            }
            skins.push(faces);
        }
        Ok(Self { skins })
    }
}
fn placeholder(skin: usize, face: usize) -> Texture {
    let mut pixels = Vec::new();
    for y in 0..32 {
        for x in 0..32 {
            let border = x < 2 || y < 2 || x > 29 || y > 29;
            let mut c = if skin == 0 {
                if face == 2 || (face != 3 && y < 15) {
                    Vec3::new(0.85, 0.035, 0.07)
                } else {
                    Vec3::new(0.9, 0.93, 0.85)
                }
            } else {
                [
                    Vec3::default(),
                    Vec3::new(0.16, 0.68, 0.27),
                    Vec3::new(0.96, 0.29, 0.06),
                    Vec3::new(0.13, 0.51, 0.9),
                ][skin]
            };
            if border {
                c = c * 0.45;
            }
            if skin == 0 && face != 2 && face != 3 {
                if (14..18).contains(&y) {
                    c = Vec3::new(0.04, 0.055, 0.09);
                }
                if (x as f64 - 15.5).hypot(y as f64 - 15.5) < 5.0 {
                    c = Vec3::new(0.03, 0.04, 0.06);
                }
                if (x as f64 - 15.5).hypot(y as f64 - 15.5) < 2.5 {
                    c = Vec3::new(0.95, 0.97, 0.9);
                }
            } else if skin > 0 && face >= 4 {
                if (8..12).contains(&x) || (21..25).contains(&x) {
                    if (10..18).contains(&y) {
                        c = Vec3::new(0.025, 0.035, 0.06);
                    }
                    if y == 11 {
                        c = Vec3::new(1.0, 1.0, 0.9);
                    }
                }
                if (13..20).contains(&x) && y == 23 {
                    c = Vec3::new(0.07, 0.06, 0.06);
                }
            }
            pixels.push(c);
        }
    }
    Texture {
        width: 32,
        height: 32,
        pixels,
    }
}
/// Una caja dinámica analítica; no redondea la posición al grid del DDA.
pub fn hit(ray: &Ray, actor: Actor) -> Option<(f64, Vec3, usize, f64, f64)> {
    let min = actor.position + Vec3::new(-0.31, 0.0, -0.31);
    let max = min + Vec3::new(0.62, 0.62, 0.62);
    let mut near = f64::NEG_INFINITY;
    let mut far = f64::INFINITY;
    let mut face = 0;
    for (axis, (o, d, lo, hi)) in [
        (ray.origin.x, ray.dir.x, min.x, max.x),
        (ray.origin.y, ray.dir.y, min.y, max.y),
        (ray.origin.z, ray.dir.z, min.z, max.z),
    ]
    .into_iter()
    .enumerate()
    {
        if d.abs() < 1e-12 {
            if o < lo || o > hi {
                return None;
            }
            continue;
        }
        let a = (lo - o) / d;
        let b = (hi - o) / d;
        if a.min(b) > near {
            near = a.min(b);
            face = axis * 2 + usize::from(d > 0.0);
        }
        far = far.min(a.max(b));
        if far < near {
            return None;
        }
    }
    if near < 0.0 {
        return None;
    }
    let p = (ray.at(near) - min) * (1.0 / 0.62);
    let (n, u, v) = match face {
        0 => (Vec3::new(1.0, 0.0, 0.0), 1.0 - p.z, 1.0 - p.y),
        1 => (Vec3::new(-1.0, 0.0, 0.0), p.z, 1.0 - p.y),
        2 => (Vec3::new(0.0, 1.0, 0.0), p.x, p.z),
        3 => (Vec3::new(0.0, -1.0, 0.0), p.x, 1.0 - p.z),
        4 => (Vec3::new(0.0, 0.0, 1.0), p.x, 1.0 - p.y),
        _ => (Vec3::new(0.0, 0.0, -1.0), 1.0 - p.x, 1.0 - p.y),
    };
    Some((near, n, face, u, v))
}
