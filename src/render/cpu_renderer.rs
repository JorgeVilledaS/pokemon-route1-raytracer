use super::{intersect_with_steps, Skybox};
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

pub struct CpuRenderer {
    width: usize,
    height: usize,
    worker_count: usize,
    rows_per_worker: usize,
}

impl CpuRenderer {
    pub fn new(width: usize, height: usize) -> Self {
        assert!(
            width > 0 && height > 0,
            "render dimensions must be positive"
        );

        let available_workers = thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1);
        let worker_count = available_workers.min(height);
        let rows_per_worker = height.div_ceil(worker_count);

        Self {
            width,
            height,
            worker_count,
            rows_per_worker,
        }
    }

    pub const fn width(&self) -> usize {
        self.width
    }

    pub const fn height(&self) -> usize {
        self.height
    }

    pub const fn worker_count(&self) -> usize {
        self.worker_count
    }

    pub const fn rows_per_worker(&self) -> usize {
        self.rows_per_worker
    }

    pub fn render(
        &self,
        scene: &Scene,
        camera: &Camera,
        materials: &MaterialLibrary,
    ) -> RenderedFrame {
        let started = Instant::now();
        let mut pixels = vec![0; self.width * self.height];
        let chunk_size = self.rows_per_worker * self.width;

        let total_dda_steps = thread::scope(|scope| {
            let handles: Vec<_> = pixels
                .chunks_mut(chunk_size)
                .enumerate()
                .map(|(worker, chunk)| {
                    let first_row = worker * self.rows_per_worker;
                    scope.spawn(move || {
                        render_rows(
                            chunk,
                            first_row,
                            self.width,
                            self.height,
                            scene,
                            camera,
                            materials,
                        )
                    })
                })
                .collect();

            handles
                .into_iter()
                .map(|handle| handle.join().expect("render worker panicked"))
                .sum()
        });

        RenderedFrame {
            pixels,
            total_dda_steps,
            elapsed: started.elapsed(),
        }
    }
}

fn render_rows(
    pixels: &mut [u32],
    first_row: usize,
    width: usize,
    height: usize,
    scene: &Scene,
    camera: &Camera,
    materials: &MaterialLibrary,
) -> usize {
    let mut total_dda_steps = 0;

    for (local_y, row) in pixels.chunks_mut(width).enumerate() {
        let y = first_row + local_y;
        let v = 1.0 - 2.0 * (y as f64 + 0.5) / height as f64;

        for (x, pixel) in row.iter_mut().enumerate() {
            let u = 2.0 * (x as f64 + 0.5) / width as f64 - 1.0;
            let ray = camera.get_ray(u, v);
            let (hit, dda_steps) = intersect_with_steps(scene, &ray);
            total_dda_steps += dda_steps;

            let color = if let Some(hit) = hit {
                let material = materials.get(hit.material);
                super::shading::shade(material, hit.uv, hit.normal, -ray.dir)
            } else {
                Skybox.sample(ray.dir)
            };
            *pixel = color_to_bgrx(color);
        }
    }

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
    use crate::scene::Scene;

    #[test]
    fn parallel_renderer_covers_every_pixel() {
        let renderer = CpuRenderer::new(17, 11);
        let scene = Scene::new(4, 4, 4);
        let materials = MaterialLibrary::load_all().expect("project textures should load");
        let camera = Camera::new(Vec3::new(2.0, 1.0, 2.0), 0.0, 0.2, 6.0, 60.0, 17.0 / 11.0);

        let frame = renderer.render(&scene, &camera, &materials);

        assert_eq!(frame.pixels.len(), 17 * 11);
        assert!(renderer.worker_count() <= 11);
        assert!(renderer.rows_per_worker() * renderer.worker_count() >= 11);
    }
}
