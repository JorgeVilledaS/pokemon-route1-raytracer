# Prompts del proyecto

Guardar en esta carpeta los prompts 00–11 para conservar la trazabilidad del desarrollo.

# Estructura del proyecto — Diorama Ruta 1 (Raytracer CPU, Rust)

## Decisión de lenguaje
**Rust**, edición 2021. Solo la **std** (`std::thread`, `std::sync`, `std::time`, `std::fs`, `std::io`).
**Nada de crates externos**: sin `rayon`, sin `image`, sin `glam`/`cgmath`, sin `noise`. Eso significa `Cargo.toml` prácticamente sin sección `[dependencies]`.

```toml
[package]
name = "pokemon-route1-raytracer"
version = "0.1.0"
edition = "2021"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1

[dependencies]
# intencionalmente vacío — solo std
```

## Árbol de directorios

```
pokemon-route1-raytracer/
├── README.md                      # Incluye el video final embebido/linkeado
├── .gitignore
├── Cargo.toml
├── Cargo.lock
├── assets/
│   ├── textures/
│   │   ├── grass_top.ppm
│   │   ├── grass_tall.ppm
│   │   ├── dirt_path.ppm
│   │   ├── wood_fence.ppm
│   │   ├── wood_fence_normal.ppm
│   │   ├── bark.ppm
│   │   ├── bark_normal.ppm
│   │   ├── leaves.ppm
│   │   ├── sign_wood.ppm
│   │   ├── sign_glow.ppm
│   │   └── water_normal.ppm
│   └── TEXTURE_CREDITS.md
├── src/
│   ├── main.rs
│   ├── core/
│   │   ├── mod.rs
│   │   ├── vec3.rs                 # struct Vec3, operator overloading (Add/Sub/Mul via traits)
│   │   ├── ray.rs
│   │   ├── camera.rs
│   │   ├── framebuffer.rs
│   │   └── image_io.rs             # escritura/lectura PPM propia
│   ├── scene/
│   │   ├── mod.rs
│   │   ├── voxel.rs
│   │   ├── scene.rs                # Vec<Voxel> denso + acceso por índice
│   │   ├── noise.rs                # value/Perlin noise propio
│   │   └── terrain_generator.rs
│   ├── materials/
│   │   ├── mod.rs
│   │   ├── material.rs             # struct Material: albedo, ks, kr, kt, ior, emissive, normal_map
│   │   └── material_library.rs     # enum MaterialId + tabla de instancias
│   ├── render/
│   │   ├── mod.rs
│   │   ├── raytracer.rs            # loop principal de trazado + rebotes recursivos
│   │   ├── voxel_intersect.rs      # DDA voxel traversal
│   │   ├── shading.rs              # Blinn-Phong + sombras
│   │   ├── fresnel.rs              # Schlick approx
│   │   └── skybox.rs
│   ├── parallel/
│   │   ├── mod.rs
│   │   ├── thread_pool.rs          # opcional: pool manual con mpsc
│   │   └── tile_scheduler.rs       # reparto de tiles, idealmente con std::thread::scope
│   └── utils/
│       ├── mod.rs
│       ├── texture.rs              # sampling con wrap/bilinear
│       └── stopwatch.rs            # std::time::Instant
├── scripts/
│   ├── render_animation.sh
│   └── frames_to_video.sh          # ffmpeg (herramienta externa, no crate)
├── output/
│   ├── frames/
│   └── final_video.mp4
└── docs/
    ├── prompts/
    └── bitacora.md
```

## Nota clave sobre paralelismo en Rust
Rust tiene una ventaja real aquí sobre C++: **`std::thread::scope`** (estable desde 1.63, es parte de la std, no un crate) permite lanzar hilos que **piden prestada** (`borrow`) la escena de forma inmutable sin necesitar `Arc`, y el compilador garantiza en tiempo de compilación que no hay data races al escribir cada quien en su porción del framebuffer. Esto es tanto una ventaja de productividad como un punto fuerte para el criterio subjetivo de "implementación de progra paralela": pueden mostrar en el código que el propio compilador certifica la ausencia de data races, algo que en C++ hay que garantizar a mano.

```rust
std::thread::scope(|s| {
    for tile in tiles.chunks_mut(tile_size) {
        s.spawn(|| render_tile(&scene, &camera, tile));
    }
});
```

## Clases/structs núcleo y sus responsabilidades

| Módulo | Responsabilidad |
|---|---|
| `Vec3` | álgebra vectorial (dot, cross, normalize, reflect, refract) vía structs + traits (`Add`, `Sub`, `Mul`, `Neg`) |
| `Ray` | origen + dirección |
| `Camera` | genera rayos por pixel; soporta orbit (yaw/pitch) y dolly (distancia al target) |
| `Voxel` | posición en grid + `MaterialId` (enum) + rotación de textura |
| `Scene` | `Vec<Voxel>` denso; acceso O(1) por coordenada, indexado `x + y*W + z*W*H` |
| `Material` | albedo (color/textura), `ks`, `kr`, `kt`, `ior`, `emissive`, `normal_map: Option<Texture>` |
| `voxel_intersect` | función libre: recorre el grid con DDA, regresa `Hit { point, normal, uv, face }` |
| `Raytracer` | recursión: sombra directa, reflexión, refracción, Fresnel, profundidad máxima (parámetro, no recursión infinita) |
| `Skybox` | color de fondo procedural (gradiente + noise) |
| `tile_scheduler` | reparte tiles del framebuffer entre hilos con `std::thread::scope` |
| `TerrainGenerator` | usa `noise.rs` para decidir tipo de bloque en el área 16x16 |

## Mapeo directo a la rúbrica

- Cada material **debe verse** en al menos un cubo visible en el video, no solo existir en código.
- Reflexión/refracción: el estanque de agua debe estar en cámara con algo detrás/debajo que se note refractado o reflejado.
- Normal map: usar ángulo de luz rasante en al menos un frame para que se note el relieve.
- Emisivo: frame de "noche" (skybox oscuro) donde el letrero/Poké Ball brille sin luz externa.
- Terreno procedural: encuadrar la cámara para que se vea el parche 16x16 con variación clara.
- Paralelismo: guardar en la bitácora tiempos antes/después de threads (y mencionar `std::thread::scope` como garantía de seguridad de memoria) como evidencia para el criterio subjetivo.

## Formato de imagen sin crates externos
**PPM (P6)** para escritura vía `std::fs::File` + `std::io::Write`, convertido a PNG/MP4 con `ffmpeg` fuera del programa (herramienta externa al lenguaje, no crate — permitido).

