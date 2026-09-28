use std::thread;

pub const TILE_SIZE: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Tile {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

/// Planificación estática por bandas de tiles. Cada worker recibe filas de
/// tiles completas, lo que permite prestar un `&mut [u32]` disjunto sin locks.
pub struct TileScheduler {
    width: usize,
    height: usize,
    worker_count: usize,
    rows_per_worker: usize,
}

impl TileScheduler {
    pub fn new(width: usize, height: usize) -> Self {
        let available = thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1);
        Self::with_worker_limit(width, height, available)
    }

    pub fn with_worker_limit(width: usize, height: usize, worker_limit: usize) -> Self {
        assert!(
            width > 0 && height > 0,
            "render dimensions must be positive"
        );
        assert!(worker_limit > 0, "worker limit must be positive");

        let tile_rows = height.div_ceil(TILE_SIZE);
        let worker_count = worker_limit.min(tile_rows);
        let tile_rows_per_worker = tile_rows.div_ceil(worker_count);

        Self {
            width,
            height,
            worker_count,
            rows_per_worker: tile_rows_per_worker * TILE_SIZE,
        }
    }

    pub const fn worker_count(&self) -> usize {
        self.worker_count
    }

    pub const fn rows_per_worker(&self) -> usize {
        self.rows_per_worker
    }

    pub const fn width(&self) -> usize {
        self.width
    }

    pub const fn height(&self) -> usize {
        self.height
    }

    pub fn for_each_tile(&self, first_row: usize, row_count: usize, mut visit: impl FnMut(Tile)) {
        let last_row = (first_row + row_count).min(self.height);
        for y in (first_row..last_row).step_by(TILE_SIZE) {
            let tile_height = TILE_SIZE.min(last_row - y);
            for x in (0..self.width).step_by(TILE_SIZE) {
                visit(Tile {
                    x,
                    y,
                    width: TILE_SIZE.min(self.width - x),
                    height: tile_height,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TileScheduler;

    #[test]
    fn tiles_cover_every_pixel_once() {
        let scheduler = TileScheduler::with_worker_limit(70, 65, 4);
        let mut visits = vec![0_u8; 70 * 65];
        let rows = scheduler.rows_per_worker();

        for first_row in (0..65).step_by(rows) {
            scheduler.for_each_tile(first_row, rows, |tile| {
                for y in tile.y..tile.y + tile.height {
                    for x in tile.x..tile.x + tile.width {
                        visits[x + y * 70] += 1;
                    }
                }
            });
        }

        assert!(visits.iter().all(|visits| *visits == 1));
    }

    #[test]
    fn worker_count_is_limited_by_tile_rows() {
        let scheduler = TileScheduler::with_worker_limit(640, 33, 16);
        assert_eq!(scheduler.worker_count(), 2);
    }
}
