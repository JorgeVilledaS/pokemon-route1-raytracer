use super::{value_noise_2d, RotationY, Voxel};
use crate::materials::MaterialId;

/// Altura del volumen denso devuelto por [`generate`].
pub const GENERATED_HEIGHT: usize = 5;

/// Genera un volumen X × 5 × Z. `MaterialId::Air` representa las celdas vacías.
/// La disposición coincide con Scene: x + y * size_x + z * size_x * height.
pub fn generate(seed: u64, size_x: usize, size_z: usize) -> Vec<Voxel> {
    let count = size_x
        .checked_mul(GENERATED_HEIGHT)
        .and_then(|area| area.checked_mul(size_z))
        .expect("terrain dimensions overflow usize");
    let air = Voxel::new(MaterialId::Air, RotationY::Deg0);
    let mut voxels = vec![air; count];
    let mut tree_cells = vec![false; size_x * size_z];

    for z in 0..size_z {
        for x in 0..size_x {
            let world_x = x as f64;
            let world_z = z as f64;
            let terrain_noise = fractal_noise(world_x * 0.24, world_z * 0.24, seed);
            let detail_noise = value_noise_2d(
                world_x * 0.71 + 19.0,
                world_z * 0.71 - 7.0,
                seed ^ 0xa076_1d64_78bd_642f,
            );
            let rotation = rotation_from_noise(detail_noise);

            set_voxel(
                &mut voxels,
                size_x,
                x,
                0,
                z,
                Voxel::new(MaterialId::Dirt, rotation),
            );

            let surface = if terrain_noise < 0.3 {
                MaterialId::Dirt
            } else {
                MaterialId::Grass
            };
            set_voxel(&mut voxels, size_x, x, 1, z, Voxel::new(surface, rotation));

            if terrain_noise > 0.7 {
                // Un segundo cubo verde crea parches de césped alto claramente
                // distinguibles sin añadir un material artificial a la rúbrica.
                set_voxel(
                    &mut voxels,
                    size_x,
                    x,
                    2,
                    z,
                    Voxel::new(MaterialId::Grass, rotation),
                );
            }

            let tree_noise = value_noise_2d(
                world_x * 0.83 - 31.0,
                world_z * 0.83 + 23.0,
                seed ^ 0xe703_7ed1_a0b4_28db,
            );
            if surface == MaterialId::Grass
                && terrain_noise <= 0.7
                && tree_noise > 0.80
                && !has_neighboring_tree(&tree_cells, size_x, size_z, x, z)
            {
                tree_cells[x + z * size_x] = true;
                place_tree(&mut voxels, size_x, size_z, x, z, rotation);
            }
        }
    }

    voxels
}

pub const fn voxel_index(size_x: usize, x: usize, y: usize, z: usize) -> usize {
    x + y * size_x + z * size_x * GENERATED_HEIGHT
}

fn fractal_noise(x: f64, z: f64, seed: u64) -> f64 {
    value_noise_2d(x, z, seed) * 0.72
        + value_noise_2d(x * 2.0 + 5.3, z * 2.0 - 9.1, seed ^ 0x8ebc_6af0_9c88_c6e3) * 0.28
}

fn rotation_from_noise(value: f64) -> RotationY {
    match (value * 4.0) as u8 {
        0 => RotationY::Deg0,
        1 => RotationY::Deg90,
        2 => RotationY::Deg180,
        _ => RotationY::Deg270,
    }
}

fn set_voxel(voxels: &mut [Voxel], size_x: usize, x: usize, y: usize, z: usize, voxel: Voxel) {
    voxels[voxel_index(size_x, x, y, z)] = voxel;
}

fn has_neighboring_tree(trees: &[bool], size_x: usize, size_z: usize, x: usize, z: usize) -> bool {
    let min_x = x.saturating_sub(2);
    let max_x = (x + 2).min(size_x.saturating_sub(1));
    let min_z = z.saturating_sub(2);
    let max_z = (z + 2).min(size_z.saturating_sub(1));
    (min_z..=max_z).any(|nz| (min_x..=max_x).any(|nx| trees[nx + nz * size_x]))
}

fn place_tree(
    voxels: &mut [Voxel],
    size_x: usize,
    size_z: usize,
    x: usize,
    z: usize,
    rotation: RotationY,
) {
    set_voxel(
        voxels,
        size_x,
        x,
        2,
        z,
        Voxel::new(MaterialId::Wood, rotation),
    );
    set_voxel(
        voxels,
        size_x,
        x,
        3,
        z,
        Voxel::new(MaterialId::Wood, rotation),
    );

    for (dx, dz) in [(0_isize, 0_isize), (-1, 0), (1, 0), (0, -1), (0, 1)] {
        let Some(nx) = x.checked_add_signed(dx) else {
            continue;
        };
        let Some(nz) = z.checked_add_signed(dz) else {
            continue;
        };
        if nx < size_x && nz < size_z {
            set_voxel(
                voxels,
                size_x,
                nx,
                4,
                nz,
                Voxel::new(MaterialId::Grass, rotation),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{generate, voxel_index, GENERATED_HEIGHT};
    use crate::materials::MaterialId;

    #[test]
    fn generation_is_reproducible_and_seeded() {
        let first = generate(42, 16, 16);
        assert_eq!(first, generate(42, 16, 16));
        assert_ne!(first, generate(43, 16, 16));
    }

    #[test]
    fn generated_volume_contains_route_variation() {
        let terrain = generate(0x524f_5554_4531, 16, 16);
        assert_eq!(terrain.len(), 16 * GENERATED_HEIGHT * 16);

        let mut dirt = 0;
        let mut grass = 0;
        let mut tall_grass = 0;
        for z in 0..16 {
            for x in 0..16 {
                match terrain[voxel_index(16, x, 1, z)].material {
                    MaterialId::Dirt => dirt += 1,
                    MaterialId::Grass => grass += 1,
                    material => panic!("unexpected surface material: {material:?}"),
                }
                if terrain[voxel_index(16, x, 2, z)].material == MaterialId::Grass {
                    tall_grass += 1;
                }
            }
        }

        assert!(dirt > 0, "seed should produce a visible dirt route");
        assert!(grass > 0, "seed should produce grass");
        assert!(tall_grass > 0, "seed should produce tall grass patches");
    }

    #[test]
    fn tree_trunks_never_start_on_dirt() {
        let terrain = generate(0x524f_5554_4531, 16, 16);
        for z in 0..16 {
            for x in 0..16 {
                if terrain[voxel_index(16, x, 2, z)].material == MaterialId::Wood {
                    assert_eq!(
                        terrain[voxel_index(16, x, 1, z)].material,
                        MaterialId::Grass
                    );
                }
            }
        }
    }
}
