#[derive(Clone, Copy, Debug)]
pub struct Tile {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub fn make_tiles(width: u32, height: u32, size: u32) -> Vec<Tile> {
    let mut tiles = Vec::new();
    for y in (0..height).step_by(size as usize) {
        for x in (0..width).step_by(size as usize) {
            tiles.push(Tile {
                x,
                y,
                width: size.min(width - x),
                height: size.min(height - y),
            });
        }
    }
    tiles
}
