# Diorama Ruta 1 — Raytracer CPU en Rust

Diorama inspirado en la Ruta 1 de Pokémon Verde Hoja, renderizado íntegramente en CPU mediante raytracing. El proyecto usa Rust 2021 y solamente la biblioteca estándar.

## Estado

Prompt 00 completado: estructura base, módulos, assets placeholder y escritura PPM. Las características visuales se implementarán incrementalmente en los siguientes prompts.

## Requisitos

- Rust estable con Cargo.
- `ffmpeg` opcional, únicamente para convertir frames PPM a video.

## Uso

```bash
cargo run --release
```

El ejecutable escribe `output/preview.ppm`.

## Video final

> Se agregará aquí el enlace o reproductor del video final cuando termine el diorama.

## Restricciones de diseño

- Sin crates externos.
- Renderizado en CPU; no se usa GPU.
- Imágenes intermedias en formato PPM P6.
- Paralelismo con `std::thread`.

## Créditos

Pokémon y sus elementos visuales pertenecen a Nintendo, Game Freak y The Pokémon Company. Este es un proyecto académico no comercial.
