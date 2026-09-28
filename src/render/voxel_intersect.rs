use crate::core::{Ray, Vec3};
use crate::materials::MaterialId;
use crate::scene::{Scene, Voxel};

const DIRECTION_EPSILON: f64 = 1.0e-12;
const BOUNDARY_NUDGE: f64 = 1.0e-9;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Face {
    NegativeX,
    PositiveX,
    NegativeY,
    PositiveY,
    NegativeZ,
    PositiveZ,
}

impl Face {
    fn from_normal(normal: Vec3) -> Self {
        if normal.x < -0.5 {
            Face::NegativeX
        } else if normal.x > 0.5 {
            Face::PositiveX
        } else if normal.y < -0.5 {
            Face::NegativeY
        } else if normal.y > 0.5 {
            Face::PositiveY
        } else if normal.z < -0.5 {
            Face::NegativeZ
        } else {
            Face::PositiveZ
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    pub point: Vec3,
    pub normal: Vec3,
    pub uv: (f64, f64),
    pub material: MaterialId,
    pub face: Face,
    pub cell: (i32, i32, i32),
}

pub fn intersect(scene: &Scene, ray: &Ray) -> Option<Hit> {
    intersect_with_steps(scene, ray).0
}

/// Consulta especializada para sombras: atraviesa únicamente el volumen
/// acotado de la escena y sale en cuanto encuentra un voxel opaco. No calcula
/// punto, cara, UV ni color del bloqueador.
pub fn occluded(scene: &Scene, ray: &Ray) -> bool {
    if scene.width == 0 || scene.height == 0 || scene.depth == 0 {
        return false;
    }

    let bounds_max = Vec3::new(scene.width as f64, scene.height as f64, scene.depth as f64);
    let Some((entry_t, exit_t, _)) = intersect_aabb(ray, Vec3::default(), bounds_max) else {
        return false;
    };
    let mut current_t = entry_t.max(0.0);
    if exit_t < current_t {
        return false;
    }

    let sample_point = ray.at(current_t) + ray.dir * BOUNDARY_NUDGE;
    let mut cell = [
        sample_point.x.floor() as i32,
        sample_point.y.floor() as i32,
        sample_point.z.floor() as i32,
    ];
    let step = [
        step_for(ray.dir.x),
        step_for(ray.dir.y),
        step_for(ray.dir.z),
    ];
    let delta_t = [
        delta_for(ray.dir.x),
        delta_for(ray.dir.y),
        delta_for(ray.dir.z),
    ];
    let mut next_t = [
        next_boundary_t(ray.origin.x, ray.dir.x, cell[0], step[0]),
        next_boundary_t(ray.origin.y, ray.dir.y, cell[1], step[1]),
        next_boundary_t(ray.origin.z, ray.dir.z, cell[2], step[2]),
    ];

    while current_t <= exit_t {
        if cell[0] < 0
            || cell[1] < 0
            || cell[2] < 0
            || cell[0] >= scene.width as i32
            || cell[1] >= scene.height as i32
            || cell[2] >= scene.depth as i32
        {
            return false;
        }

        if scene
            .get(cell[0], cell[1], cell[2])
            .is_some_and(|voxel| voxel.material != MaterialId::Water)
        {
            return true;
        }

        let axis = smallest_axis(next_t);
        current_t = next_t[axis];
        cell[axis] += step[axis];
        next_t[axis] += delta_t[axis];
    }

    false
}

/// Variante instrumentada: el contador indica cuántas celdas visitó el DDA.
pub fn intersect_with_steps(scene: &Scene, ray: &Ray) -> (Option<Hit>, usize) {
    if scene.width == 0 || scene.height == 0 || scene.depth == 0 {
        return (None, 0);
    }

    let bounds_max = Vec3::new(scene.width as f64, scene.height as f64, scene.depth as f64);
    let Some((entry_t, exit_t, entry_normal)) = intersect_aabb(ray, Vec3::default(), bounds_max)
    else {
        return (None, 0);
    };

    let current_t = entry_t.max(0.0);
    if exit_t < current_t {
        return (None, 0);
    }

    // El pequeño avance coloca inequívocamente el punto dentro de la primera celda.
    let sample_point = ray.at(current_t) + ray.dir * BOUNDARY_NUDGE;
    let mut cell = [
        sample_point.x.floor() as i32,
        sample_point.y.floor() as i32,
        sample_point.z.floor() as i32,
    ];
    let step = [
        step_for(ray.dir.x),
        step_for(ray.dir.y),
        step_for(ray.dir.z),
    ];
    let delta_t = [
        delta_for(ray.dir.x),
        delta_for(ray.dir.y),
        delta_for(ray.dir.z),
    ];
    let mut next_t = [
        next_boundary_t(ray.origin.x, ray.dir.x, cell[0], step[0]),
        next_boundary_t(ray.origin.y, ray.dir.y, cell[1], step[1]),
        next_boundary_t(ray.origin.z, ray.dir.z, cell[2], step[2]),
    ];

    let mut hit_t = current_t;
    let mut hit_normal = if entry_t >= 0.0 {
        entry_normal
    } else {
        opposing_axis_normal(ray.dir)
    };
    let mut steps = 0;

    while hit_t <= exit_t {
        if cell[0] < 0
            || cell[1] < 0
            || cell[2] < 0
            || cell[0] >= scene.width as i32
            || cell[1] >= scene.height as i32
            || cell[2] >= scene.depth as i32
        {
            break;
        }

        steps += 1;
        if let Some(voxel) = scene.get(cell[0], cell[1], cell[2]) {
            let point = ray.at(hit_t);
            let face = Face::from_normal(hit_normal);
            return (
                Some(Hit {
                    point,
                    normal: hit_normal,
                    uv: voxel.rotation_y.rotate_uv(face_uv(point, face)),
                    material: voxel.material,
                    face,
                    cell: (cell[0], cell[1], cell[2]),
                }),
                steps,
            );
        }

        let axis = smallest_axis(next_t);
        hit_t = next_t[axis];
        if hit_t > exit_t {
            break;
        }

        cell[axis] += step[axis];
        next_t[axis] += delta_t[axis];
        hit_normal = axis_normal(axis, step[axis]);
    }

    (None, steps)
}

fn step_for(direction: f64) -> i32 {
    if direction > DIRECTION_EPSILON {
        1
    } else if direction < -DIRECTION_EPSILON {
        -1
    } else {
        0
    }
}

fn delta_for(direction: f64) -> f64 {
    if direction.abs() <= DIRECTION_EPSILON {
        f64::INFINITY
    } else {
        direction.recip().abs()
    }
}

fn next_boundary_t(origin: f64, direction: f64, cell: i32, step: i32) -> f64 {
    match step {
        1 => (f64::from(cell + 1) - origin) / direction,
        -1 => (f64::from(cell) - origin) / direction,
        _ => f64::INFINITY,
    }
}

fn smallest_axis(values: [f64; 3]) -> usize {
    if values[0] <= values[1] && values[0] <= values[2] {
        0
    } else if values[1] <= values[2] {
        1
    } else {
        2
    }
}

fn axis_normal(axis: usize, step: i32) -> Vec3 {
    let component = -f64::from(step);
    match axis {
        0 => Vec3::new(component, 0.0, 0.0),
        1 => Vec3::new(0.0, component, 0.0),
        _ => Vec3::new(0.0, 0.0, component),
    }
}

fn opposing_axis_normal(direction: Vec3) -> Vec3 {
    let absolute = Vec3::new(direction.x.abs(), direction.y.abs(), direction.z.abs());
    if absolute.x >= absolute.y && absolute.x >= absolute.z {
        Vec3::new(-direction.x.signum(), 0.0, 0.0)
    } else if absolute.y >= absolute.z {
        Vec3::new(0.0, -direction.y.signum(), 0.0)
    } else {
        Vec3::new(0.0, 0.0, -direction.z.signum())
    }
}

fn face_uv(point: Vec3, face: Face) -> (f64, f64) {
    match face {
        Face::NegativeX | Face::PositiveX => (point.z.rem_euclid(1.0), point.y.rem_euclid(1.0)),
        Face::NegativeY | Face::PositiveY => (point.x.rem_euclid(1.0), point.z.rem_euclid(1.0)),
        Face::NegativeZ | Face::PositiveZ => (point.x.rem_euclid(1.0), point.y.rem_euclid(1.0)),
    }
}

fn intersect_aabb(ray: &Ray, min: Vec3, max: Vec3) -> Option<(f64, f64, Vec3)> {
    let mut near = f64::NEG_INFINITY;
    let mut far = f64::INFINITY;
    let mut near_normal = Vec3::default();

    update_slab(
        ray.origin.x,
        ray.dir.x,
        min.x,
        max.x,
        Vec3::new(-1.0, 0.0, 0.0),
        &mut near,
        &mut far,
        &mut near_normal,
    )?;
    update_slab(
        ray.origin.y,
        ray.dir.y,
        min.y,
        max.y,
        Vec3::new(0.0, -1.0, 0.0),
        &mut near,
        &mut far,
        &mut near_normal,
    )?;
    update_slab(
        ray.origin.z,
        ray.dir.z,
        min.z,
        max.z,
        Vec3::new(0.0, 0.0, -1.0),
        &mut near,
        &mut far,
        &mut near_normal,
    )?;

    (far >= near.max(0.0)).then_some((near, far, near_normal))
}

#[allow(clippy::too_many_arguments)]
fn update_slab(
    origin: f64,
    direction: f64,
    min: f64,
    max: f64,
    min_normal: Vec3,
    near: &mut f64,
    far: &mut f64,
    near_normal: &mut Vec3,
) -> Option<()> {
    if direction.abs() <= DIRECTION_EPSILON {
        return (origin >= min && origin <= max).then_some(());
    }

    let mut axis_near = (min - origin) / direction;
    let mut axis_far = (max - origin) / direction;
    let mut axis_normal = min_normal;
    if axis_near > axis_far {
        std::mem::swap(&mut axis_near, &mut axis_far);
        axis_normal = -min_normal;
    }

    if axis_near > *near {
        *near = axis_near;
        *near_normal = axis_normal;
    }
    *far = far.min(axis_far);

    (*near <= *far).then_some(())
}

#[cfg(test)]
mod tests {
    use super::{face_uv, intersect, intersect_aabb, intersect_with_steps, occluded, Face, Hit};
    use crate::core::{Ray, Vec3};
    use crate::materials::MaterialId;
    use crate::scene::{RotationY, Scene, Voxel};

    const EPSILON: f64 = 1.0e-10;

    fn grass() -> Voxel {
        Voxel::new(MaterialId::Grass, RotationY::Deg0)
    }

    fn assert_vec3_close(actual: Vec3, expected: Vec3) {
        assert!((actual.x - expected.x).abs() < EPSILON);
        assert!((actual.y - expected.y).abs() < EPSILON);
        assert!((actual.z - expected.z).abs() < EPSILON);
    }

    #[test]
    fn dda_returns_point_normal_uv_and_material() {
        let mut scene = Scene::new(4, 4, 4);
        scene.set(1, 1, 1, Some(grass()));
        let ray = Ray::new(Vec3::new(1.5, 1.5, 4.5), Vec3::new(0.0, 0.0, -1.0));

        let hit = intersect(&scene, &ray).expect("ray should hit voxel");

        assert_vec3_close(hit.point, Vec3::new(1.5, 1.5, 2.0));
        assert_vec3_close(hit.normal, Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(hit.uv, (0.5, 0.5));
        assert_eq!(hit.material, MaterialId::Grass);
    }

    #[test]
    fn dda_returns_none_when_ray_misses_scene() {
        let scene = Scene::new(4, 4, 4);
        let ray = Ray::new(Vec3::new(-1.0, 5.0, 2.0), Vec3::new(1.0, 0.0, 0.0));

        assert_eq!(intersect(&scene, &ray), None);
    }

    #[test]
    fn shadow_query_ignores_water_and_exits_on_first_opaque_voxel() {
        let mut scene = Scene::new(4, 1, 1);
        scene.set(
            1,
            0,
            0,
            Some(Voxel::new(MaterialId::Water, RotationY::Deg0)),
        );
        scene.set(2, 0, 0, Some(grass()));
        let ray = Ray::new(Vec3::new(-1.0, 0.5, 0.5), Vec3::new(1.0, 0.0, 0.0));

        assert!(occluded(&scene, &ray));
        scene.set(2, 0, 0, None);
        assert!(!occluded(&scene, &ray));
    }

    #[test]
    fn uv_respects_voxel_rotation() {
        let point = Vec3::new(1.25, 2.75, 3.0);
        let uv = face_uv(point, Face::PositiveZ);

        assert_eq!(RotationY::Deg90.rotate_uv(uv), (0.75, 0.75));
    }

    #[test]
    fn dda_visits_a_line_not_the_entire_voxel_grid() {
        let mut scene = Scene::new(128, 8, 128);
        scene.set(127, 0, 0, Some(grass()));
        let ray = Ray::new(Vec3::new(-1.0, 0.5, 0.5), Vec3::new(1.0, 0.0, 0.0));

        let (dda_hit, dda_steps) = intersect_with_steps(&scene, &ray);
        let (brute_hit, brute_checks) = intersect_bruteforce(&scene, &ray);

        assert_eq!(dda_hit.map(|hit| hit.material), Some(MaterialId::Grass));
        assert_eq!(brute_hit.map(|hit| hit.material), Some(MaterialId::Grass));
        assert_eq!(brute_checks, scene.voxel_count());
        assert_eq!(dda_steps, 128);
        assert!(dda_steps * 100 < brute_checks);
    }

    // Control de referencia: comprueba cada celda y escala con el volumen total.
    fn intersect_bruteforce(scene: &Scene, ray: &Ray) -> (Option<Hit>, usize) {
        let mut closest: Option<(f64, Hit)> = None;
        let mut checks = 0;

        for z in 0..scene.depth as i32 {
            for y in 0..scene.height as i32 {
                for x in 0..scene.width as i32 {
                    checks += 1;
                    let Some(voxel) = scene.get(x, y, z) else {
                        continue;
                    };
                    let min = Vec3::new(f64::from(x), f64::from(y), f64::from(z));
                    let max = min + Vec3::new(1.0, 1.0, 1.0);
                    let Some((near, far, normal)) = intersect_aabb(ray, min, max) else {
                        continue;
                    };
                    let hit_t = near.max(0.0);
                    if hit_t > far || closest.is_some_and(|(distance, _)| hit_t >= distance) {
                        continue;
                    }
                    let point = ray.at(hit_t);
                    let face = Face::from_normal(normal);
                    closest = Some((
                        hit_t,
                        Hit {
                            point,
                            normal,
                            uv: voxel.rotation_y.rotate_uv(face_uv(point, face)),
                            material: voxel.material,
                            face,
                            cell: (x, y, z),
                        },
                    ));
                }
            }
        }

        (closest.map(|(_, hit)| hit), checks)
    }
}
