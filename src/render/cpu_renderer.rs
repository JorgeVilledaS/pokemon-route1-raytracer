use super::tile_scheduler::TileScheduler;
use super::{Raytracer, SceneMode};
use crate::core::{Camera, Vec3};
use crate::materials::MaterialLibrary;
use crate::scene::Scene;
use std::thread;
use std::time::{Duration, Instant};

pub struct RenderedFrame {
    pub pixels: Vec<u32>,
    pub total_dda_steps: usize,
    pub elapsed: Duration,
}

pub struct RenderStats {
    pub total_dda_steps: usize,
    pub elapsed: Duration,
}

pub struct CpuRenderer {
    scheduler: TileScheduler,
}

#[derive(Clone, Copy)]
struct RenderContext<'a> {
    width: usize,
    height: usize,
    scene: &'a Scene,
    camera: &'a Camera,
    materials: &'a MaterialLibrary,
    mode: SceneMode,
}

impl CpuRenderer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            scheduler: TileScheduler::new(width, height),
        }
    }

    pub fn single_threaded(width: usize, height: usize) -> Self {
        Self {
            scheduler: TileScheduler::with_worker_limit(width, height, 1),
        }
    }

    pub const fn width(&self) -> usize {
        self.scheduler.width()
    }

    pub const fn height(&self) -> usize {
        self.scheduler.height()
    }

    pub const fn worker_count(&self) -> usize {
        self.scheduler.worker_count()
    }

    pub const fn rows_per_worker(&self) -> usize {
        self.scheduler.rows_per_worker()
    }

    pub fn render(
        &self,
        scene: &Scene,
        camera: &Camera,
        materials: &MaterialLibrary,
        mode: SceneMode,
    ) -> RenderedFrame {
        let mut pixels = vec![0; self.width() * self.height()];
        let stats = self.render_into(&mut pixels, scene, camera, materials, mode);

        RenderedFrame {
            pixels,
            total_dda_steps: stats.total_dda_steps,
            elapsed: stats.elapsed,
        }
    }

    /// Renderiza sobre almacenamiento del llamador. La ventana conserva este
    /// buffer entre frames: hay cero asignaciones por pixel/rayo y no se crea
    /// un framebuffer nuevo al mover la cámara.
    pub fn render_into(
        &self,
        pixels: &mut [u32],
        scene: &Scene,
        camera: &Camera,
        materials: &MaterialLibrary,
        mode: SceneMode,
    ) -> RenderStats {
        assert_eq!(pixels.len(), self.width() * self.height());
        let started = Instant::now();
        let chunk_size = self.rows_per_worker() * self.width();
        let context = RenderContext {
            width: self.width(),
            height: self.height(),
            scene,
            camera,
            materials,
            mode,
        };

        let total_dda_steps = thread::scope(|scope| {
            let handles: Vec<_> = pixels
                .chunks_mut(chunk_size)
                .enumerate()
                .map(|(worker, chunk)| {
                    let first_row = worker * self.rows_per_worker();
                    let scheduler = &self.scheduler;
                    scope.spawn(move || render_tiles(chunk, first_row, context, scheduler))
                })
                .collect();

            handles
                .into_iter()
                .map(|handle| handle.join().expect("render worker panicked"))
                .sum()
        });

        RenderStats {
            total_dda_steps,
            elapsed: started.elapsed(),
        }
    }
}

fn render_tiles(
    pixels: &mut [u32],
    first_row: usize,
    context: RenderContext<'_>,
    scheduler: &TileScheduler,
) -> usize {
    let mut total_dda_steps = 0;
    let raytracer = Raytracer::new(context.scene, context.materials, context.mode, 3);
    let row_count = pixels.len() / context.width;

    scheduler.for_each_tile(first_row, row_count, |tile| {
        for y in tile.y..tile.y + tile.height {
            let v = 1.0 - 2.0 * (y as f64 + 0.5) / context.height as f64;
            let local_row = y - first_row;
            for x in tile.x..tile.x + tile.width {
                let u = 2.0 * (x as f64 + 0.5) / context.width as f64 - 1.0;
                let ray = context.camera.get_ray(u, v);
                let (color, dda_steps) = raytracer.trace_with_steps(&ray, raytracer.max_depth);
                total_dda_steps += dda_steps;
                pixels[local_row * context.width + x] = color_to_bgrx(color);
            }
        }
    });

    total_dda_steps
}

fn color_to_bgrx(color: Vec3) -> u32 {
    let color = color.clamp(0.0, 1.0);
    let red = gamma_to_byte(color.x);
    let green = gamma_to_byte(color.y);
    let blue = gamma_to_byte(color.z);

    u32::from(blue) | (u32::from(green) << 8) | (u32::from(red) << 16)
}

fn gamma_to_byte(value: f64) -> u8 {
    (value.powf(1.0 / 2.2) * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::CpuRenderer;
    use crate::core::{Camera, Vec3};
    use crate::materials::MaterialLibrary;
    use crate::render::SceneMode;
    use crate::scene::Scene;

    #[test]
    fn parallel_renderer_covers_every_pixel() {
        let renderer = CpuRenderer::new(17, 11);
        let scene = Scene::new(4, 4, 4);
        let materials = MaterialLibrary::load_all().expect("project textures should load");
        let camera = Camera::new(Vec3::new(2.0, 1.0, 2.0), 0.0, 0.2, 6.0, 60.0, 17.0 / 11.0);

        let frame = renderer.render(&scene, &camera, &materials, SceneMode::Day);

        assert_eq!(frame.pixels.len(), 17 * 11);
        assert!(renderer.worker_count() <= 11_usize.div_ceil(32));
        assert!(renderer.rows_per_worker() * renderer.worker_count() >= 11);
    }

    #[test]
    fn mono_and_parallel_renderers_are_byte_identical() {
        let parallel = CpuRenderer::new(65, 40);
        let serial = CpuRenderer::single_threaded(65, 40);
        let scene = Scene::new(4, 4, 4);
        let materials = MaterialLibrary::load_all().expect("project textures should load");
        let camera = Camera::new(Vec3::new(2.0, 1.0, 2.0), 0.4, 0.3, 6.0, 60.0, 65.0 / 40.0);

        let parallel_frame = parallel.render(&scene, &camera, &materials, SceneMode::Day);
        let serial_frame = serial.render(&scene, &camera, &materials, SceneMode::Day);

        assert_eq!(parallel_frame.pixels, serial_frame.pixels);
        assert_eq!(parallel_frame.total_dda_steps, serial_frame.total_dda_steps);
    }
}
