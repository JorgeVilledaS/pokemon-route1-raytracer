# Diorama Ruta 1 — Raytracer CPU en Rust

Diorama inspirado en la Ruta 1 de Pokémon Verde Hoja, renderizado íntegramente en CPU mediante raytracing. El proyecto usa Rust 2021 y solamente la biblioteca estándar.

## Estado

El proyecto incluye matemática vectorial, cámara orbital, escena voxel e intersección DDA. En Windows se ejecuta en una ventana interactiva con renderizado paralelo exclusivamente en CPU.

## Requisitos

- Rust estable con Cargo.
- Windows para la ventana interactiva nativa.
- `ffmpeg` opcional, únicamente para convertir frames PPM a video.

## Uso

```bash
cargo run --release
```

El ejecutable abre una ventana de 960×540. El raytracer trabaja internamente a 480×270 y escala el resultado con GDI para mantener una interacción fluida sin utilizar la GPU para el trazado.

Controles:

- Arrastrar con el botón izquierdo, flechas o `A`/`D`: orbitar libremente.
- Rueda del mouse o `W`/`S`: acercar y alejar.
- `N`: alternar instantáneamente entre día y noche.
- `R`: restablecer la cámara.
- `Esc`: cerrar.

Las filas del framebuffer se reparten entre los procesadores lógicos disponibles mediante `std::thread::scope`. Al iniciar, la consola muestra cuántos workers se usan y cuántas scanlines recibe cada uno.

Para generar una captura PPM con el mismo renderer sin abrir la ventana:

```bash
cargo run --release -- --preview
```

Para verificar el material emisivo con el cielo y la luz nocturnos:

```bash
cargo run --release -- --preview --night
```

## Video final

> Se agregará aquí el enlace o reproductor del video final cuando termine el diorama.

## Restricciones de diseño

- Sin crates externos.
- Renderizado en CPU; no se usa GPU para el raytracing.
- Presentación de la imagen mediante Win32/GDI enlazado directamente con FFI.
- Imágenes intermedias en formato PPM P6.
- Paralelismo con `std::thread`.

## Créditos

Pokémon y sus elementos visuales pertenecen a Nintendo, Game Freak y The Pokémon Company. Este es un proyecto académico no comercial.
