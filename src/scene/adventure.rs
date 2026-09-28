use super::{value_noise_2d, RotationY, Scene, Voxel};
use crate::core::Vec3;
use crate::materials::MaterialId;

pub const WIDTH: usize = 48;
pub const DEPTH: usize = 80;
pub const START: f64 = 75.0;
pub const FINISH: f64 = 3.0;
/// Coordenadas de la imagen cenital: X derecha, Z hacia abajo.
/// Entrada sur y salida norte; el camino amarillo recorre el lado derecho.
const WAYPOINTS: [(f64, f64); 14] = [
    (24.0, 75.0),
    (24.0, 65.0),
    (24.0, 60.0),
    (41.0, 60.0),
    (41.0, 44.0),
    (12.0, 44.0),
    (12.0, 38.0),
    (29.0, 38.0),
    (29.0, 24.0),
    (41.0, 24.0),
    (41.0, 9.0),
    (25.0, 9.0),
    (24.0, 7.0),
    (24.0, 3.0),
];
fn point(p: (f64, f64)) -> Vec3 {
    Vec3::new(p.0, 3.0, p.1)
}
pub fn start_position() -> Vec3 {
    point(WAYPOINTS[0])
}
pub fn overview_target() -> Vec3 {
    Vec3::new(24.0, 2.0, 40.0)
}
fn total_length() -> f64 {
    WAYPOINTS
        .windows(2)
        .map(|w| (point(w[1]) - point(w[0])).length())
        .sum()
}
fn route_point(mut distance: f64) -> Vec3 {
    for w in WAYPOINTS.windows(2) {
        let a = point(w[0]);
        let d = point(w[1]) - a;
        let length = d.length();
        if distance <= length {
            return a + d * (distance / length);
        }
        distance -= length;
    }
    point(*WAYPOINTS.last().unwrap())
}
fn nearest_route(p: Vec3) -> (f64, f64) {
    let mut nearest = f64::INFINITY;
    let mut along = 0.0;
    let mut base = 0.0;
    for w in WAYPOINTS.windows(2) {
        let a = point(w[0]);
        let d = point(w[1]) - a;
        let len = d.length();
        let t = ((p - a).dot(d) / (len * len)).clamp(0.0, 1.0);
        let distance = (p - (a + d * t)).length();
        if distance < nearest {
            nearest = distance;
            along = base + t * len;
        }
        base += len;
    }
    (nearest, along)
}
fn road_distance(x: i32, z: i32) -> f64 {
    nearest_route(Vec3::new(x as f64 + 0.5, 3.0, z as f64 + 0.5)).0
}
fn put(s: &mut Scene, x: i32, y: i32, z: i32, m: MaterialId) {
    s.set(x, y, z, Some(Voxel::new(m, RotationY::Deg0)));
}
fn patch(s: &mut Scene, x0: i32, x1: i32, z0: i32, z1: i32, m: MaterialId) {
    for z in z0..z1 {
        for x in x0..x1 {
            if road_distance(x, z) > 2.0 {
                put(s, x, 2, z, m);
            }
        }
    }
}
fn ledge(s: &mut Scene, x0: i32, x1: i32, z: i32) {
    for x in x0..x1 {
        if road_distance(x, z) > 2.1 {
            put(s, x, 3, z, MaterialId::Wood);
            // Franja verde superior, con el canto de tierra visible al sur.
            put(s, x, 3, z - 1, MaterialId::Grass);
        }
    }
}
fn tree(s: &mut Scene, x: i32, z: i32) {
    if road_distance(x, z) < 3.5 {
        return;
    }
    for y in 3..6 {
        put(s, x, y, z, MaterialId::Wood);
    }
    for (y, r) in [(5, 2_i32), (6, 2), (7, 1), (8, 0)] {
        for dz in -r..=r {
            for dx in -r..=r {
                if dx * dx + dz * dz <= r * r + 1 {
                    put(s, x + dx, y, z + dz, MaterialId::Leaves);
                }
            }
        }
    }
}
pub fn build_route() -> Scene {
    let mut s = Scene::new(WIDTH, 12, DEPTH);
    for z in 0..DEPTH as i32 {
        for x in 0..WIDTH as i32 {
            put(&mut s, x, 0, z, MaterialId::Rock);
            put(&mut s, x, 1, z, MaterialId::Wood);
            put(&mut s, x, 2, z, MaterialId::Grass);
        }
    }
    // Sectores de hierba de la referencia, con un bloque procedural 16x16 arriba.
    let region = super::generate(0x524f_5554_4531, 16, 16);
    for z in 0..16 {
        for x in 0..16 {
            let wx = 22 + x as i32;
            let wz = 12 + z as i32;
            if road_distance(wx, wz) > 2.0 {
                let generated = region[super::voxel_index(16, x, 2, z)].material;
                put(
                    &mut s,
                    wx,
                    2,
                    wz,
                    if generated == MaterialId::TallGrass {
                        MaterialId::TallGrass
                    } else {
                        MaterialId::Grass
                    },
                );
            }
        }
    }
    for (x0, x1, z0, z1) in [
        (20, 44, 12, 22),
        (32, 44, 26, 32),
        (24, 35, 49, 58),
        (4, 21, 64, 72),
        (31, 42, 65, 72),
        (22, 28, 72, 78),
    ] {
        patch(&mut s, x0, x1, z0, z1, MaterialId::TallGrass);
    }
    // Sendero amarillo: entrada corta, vuelta inferior, giro central y salida superior.
    for z in 0..80 {
        for x in 0..48 {
            let main = road_distance(x, z) < 1.65 && (3..70).contains(&z);
            let lower_branch = (5..41).contains(&x) && (59..62).contains(&z);
            let entrance = (21..27).contains(&x) && (63..69).contains(&z);
            if main || lower_branch || entrance {
                put(&mut s, x, 2, z, MaterialId::Dirt);
            }
        }
    }
    for (x0, x1, z) in [
        (4, 21, 63),
        (27, 44, 63),
        (36, 44, 53),
        (4, 8, 41),
        (11, 19, 41),
        (23, 44, 41),
        (7, 21, 31),
        (4, 17, 21),
        (4, 17, 11),
    ] {
        ledge(&mut s, x0, x1, z);
    }
    // Flores en grupos, respetando las zonas despejadas de la referencia.
    for (cx, cz) in [
        (6, 14),
        (7, 18),
        (6, 24),
        (38, 35),
        (41, 37),
        (9, 55),
        (14, 56),
        (18, 69),
        (40, 70),
        (42, 73),
        (38, 5),
    ] {
        for (dx, dz) in [(0, 0), (2, 1), (-1, 2)] {
            let x = cx + dx;
            let z = cz + dz;
            if road_distance(x, z) > 2.2 {
                put(&mut s, x, 2, z, MaterialId::Flowers);
            }
        }
    }
    for z in (3..78).step_by(4) {
        tree(&mut s, 2, z);
        tree(&mut s, 45, z);
    }
    for x in (6..44).step_by(4) {
        tree(&mut s, x, 2);
        tree(&mut s, x, 77);
    }
    for x in [6, 10, 14, 18, 22] {
        tree(&mut s, x, 49);
    }
    for (x, z) in [
        (6, 30),
        (22, 30),
        (27, 30),
        (18, 8),
        (18, 12),
        (18, 16),
        (18, 20),
    ] {
        tree(&mut s, x, z);
    }
    // Cerca de entrada con abertura central, tal como la imagen.
    for x in 3..45 {
        if !(22..27).contains(&x) {
            put(&mut s, x, 3, 74, MaterialId::Fence);
        }
    }
    put(&mut s, 19, 3, 63, MaterialId::Wood);
    put(&mut s, 19, 4, 63, MaterialId::Sign);
    // Pequeño estanque lateral fuera del sendero para conservar la demostración óptica.
    for z in 33..36 {
        for x in 5..8 {
            put(
                &mut s,
                x,
                1,
                z,
                if (x + z) % 2 == 0 {
                    MaterialId::Rock
                } else {
                    MaterialId::Dirt
                },
            );
            put(&mut s, x, 2, z, MaterialId::Water);
        }
    }
    s
}
#[derive(Clone, Copy, Debug)]
pub struct Actor {
    pub position: Vec3,
    pub skin: usize,
}
pub struct Encounter {
    pub position: Vec3,
    pub skin: usize,
    pub captured: bool,
}
pub struct Adventure {
    pub position: Vec3,
    pub skin: usize,
    pub auto: bool,
    pub finished: bool,
    pub encounters: Vec<Encounter>,
    pub time: f64,
    furthest: f64,
}
impl Adventure {
    pub fn new(seed: u64) -> Self {
        let length = total_length();
        let encounters = (0..9)
            .map(|i| {
                let distance = 10.0
                    + i as f64 * (length - 20.0) / 9.0
                    + value_noise_2d(i as f64, 0.0, seed) * 1.5;
                Encounter {
                    position: route_point(distance),
                    skin: 1 + i % 3,
                    captured: false,
                }
            })
            .collect();
        Self {
            position: start_position(),
            skin: 0,
            auto: false,
            finished: false,
            encounters,
            time: 0.0,
            furthest: 0.0,
        }
    }
    pub fn count(&self) -> usize {
        self.encounters.iter().filter(|e| e.captured).count()
    }
    pub fn progress(&self) -> f64 {
        (self.furthest / total_length()).clamp(0.0, 1.0)
    }
    pub fn update(&mut self, scene: &Scene, dt: f64, input: Vec3) {
        if self.finished {
            return;
        }
        self.time += dt;
        if input.length() > 0.0 {
            self.auto = false;
        }
        let along = nearest_route(self.position).1;
        let direction = if self.auto {
            (route_point((along + 0.55).min(total_length())) - self.position).normalize()
        } else {
            input.normalize()
        };
        let travel = direction * (dt.clamp(0.0, 0.1) * 4.0);
        for delta in [Vec3::new(travel.x, 0.0, 0.0), Vec3::new(0.0, 0.0, travel.z)] {
            let next = self.position + delta;
            if walkable(scene, next) {
                self.position = next;
            }
        }
        self.furthest = self.furthest.max(nearest_route(self.position).1);
        for e in &mut self.encounters {
            if !e.captured && (e.position - self.position).length() < 0.85 {
                e.captured = true;
                self.skin = e.skin;
            }
        }
        self.finished = (self.position - point(*WAYPOINTS.last().unwrap())).length() < 0.4;
    }
    pub fn sync(&self, scene: &mut Scene) {
        scene.actor = Some(Actor {
            position: self.position,
            skin: self.skin,
        });
        scene.encounters.clear();
        scene.encounters.extend(
            self.encounters
                .iter()
                .filter(|e| !e.captured)
                .map(|e| Actor {
                    position: e.position
                        + Vec3::new(
                            0.0,
                            0.12 + 0.08 * (self.time * 2.0 + e.position.z).sin(),
                            0.0,
                        ),
                    skin: e.skin,
                }),
        );
    }
}
fn walkable(scene: &Scene, p: Vec3) -> bool {
    for dx in [-0.31, 0.31] {
        for dz in [-0.31, 0.31] {
            let x = (p.x + dx).floor() as i32;
            let z = (p.z + dz).floor() as i32;
            if scene
                .get(x, 2, z)
                .is_none_or(|v| v.material == MaterialId::Water)
                || scene.get(x, 3, z).is_some()
            {
                return false;
            }
        }
    }
    true
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn automatic_route_reaches_end_and_collects_all() {
        let scene = build_route();
        let mut game = Adventure::new(42);
        game.auto = true;
        for _ in 0..6000 {
            game.update(&scene, 0.025, Vec3::default());
        }
        assert!(game.finished, "blocked at {:?}", game.position);
        assert_eq!(game.count(), 9);
    }
    #[test]
    fn movement_cannot_enter_water_or_leave_grid() {
        let s = build_route();
        assert!(!walkable(&s, Vec3::new(5.5, 3.0, 34.5)));
        assert!(!walkable(&s, Vec3::new(-1.0, 3.0, 0.0)));
    }
}
