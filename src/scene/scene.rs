use super::Voxel;

pub struct Scene {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    voxels: Vec<Option<Voxel>>,
}

impl Scene {
    pub fn new(width: usize, height: usize, depth: usize) -> Self {
        Self {
            width,
            height,
            depth,
            voxels: vec![None; width * height * depth],
        }
    }
    fn index(&self, x: usize, y: usize, z: usize) -> usize {
        x + y * self.width + z * self.width * self.height
    }
    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<Voxel> {
        self.voxels.get(self.index(x, y, z)).copied().flatten()
    }
    pub fn set(&mut self, x: usize, y: usize, z: usize, voxel: Option<Voxel>) {
        let index = self.index(x, y, z);
        self.voxels[index] = voxel;
    }
}
