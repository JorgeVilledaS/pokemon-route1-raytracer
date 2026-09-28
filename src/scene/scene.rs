use super::Voxel;

pub struct Scene {
    pub actor: Option<super::adventure::Actor>,
    pub encounters: Vec<super::adventure::Actor>,
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    voxels: Vec<Option<Voxel>>,
}

impl Scene {
    pub fn new(width: usize, height: usize, depth: usize) -> Self {
        let voxel_count = width
            .checked_mul(height)
            .and_then(|area| area.checked_mul(depth))
            .expect("scene dimensions overflow usize");

        Self {
            actor: None,
            encounters: Vec::new(),
            width,
            height,
            depth,
            voxels: vec![None; voxel_count],
        }
    }

    pub fn get(&self, x: i32, y: i32, z: i32) -> Option<&Voxel> {
        self.index(x, y, z)
            .and_then(|index| self.voxels[index].as_ref())
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32, voxel: Option<Voxel>) -> bool {
        let Some(index) = self.index(x, y, z) else {
            return false;
        };

        self.voxels[index] = voxel;
        true
    }

    pub const fn voxel_count(&self) -> usize {
        self.voxels.len()
    }

    fn index(&self, x: i32, y: i32, z: i32) -> Option<usize> {
        if x < 0
            || y < 0
            || z < 0
            || x as usize >= self.width
            || y as usize >= self.height
            || z as usize >= self.depth
        {
            return None;
        }

        Some(x as usize + y as usize * self.width + z as usize * self.width * self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::Scene;
    use crate::materials::MaterialId;
    use crate::scene::{RotationY, Voxel};

    #[test]
    fn get_checks_bounds_and_returns_references() {
        let mut scene = Scene::new(4, 2, 3);
        let grass = Voxel::new(MaterialId::Grass, RotationY::Deg0);

        assert!(scene.set(1, 0, 2, Some(grass)));
        assert_eq!(scene.get(1, 0, 2), Some(&grass));
        assert_eq!(scene.get(-1, 0, 0), None);
        assert_eq!(scene.get(4, 0, 0), None);
        assert!(!scene.set(0, 2, 0, Some(grass)));
    }
}
