# Diorama Ruta 1 — Raytracer CPU en Rust

Diorama inspirado en la Ruta 1 de Pokémon Verde Hoja, renderizado íntegramente en CPU mediante raytracing. El proyecto usa Rust 2021 y solamente la biblioteca estándar.

## Estado

El proyecto incluye matemática vectorial, cámara orbital, escena voxel, intersección DDA, materiales, reflexión/refracción, normal mapping, cielo procedural y una región de terreno Route 1 de 16×16. En Windows se ejecuta en una ventana interactiva con renderizado paralelo exclusivamente en CPU.

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

- La partida comienza directamente como Poké Ball, sin pantalla de selección.
- `WASD`: caminar relativo a la orientación de la cámara. En la vista inicial, `W` avanza hacia el final.
- `Espacio`: activar/desactivar recorrido automático por el camino; caminar manualmente lo cancela.
- Tocar un encuentro captura su apariencia. `1`/`2`/`3` equipan apariencias ya capturadas.
- Arrastrar con el botón izquierdo o flechas: orbitar libremente.
- Rueda del mouse o `Q`/`E`: acercar y alejar.
- `Tab`: alternar cámara de seguimiento y vista general de la ruta.
- `N`: alternar día y noche.
- `P`: reiniciar la partida y los encuentros.
- `R`: restablecer la cámara.
- `Esc`: cerrar.

La ruta mide 48×80 bloques y sigue la distribución de las referencias: camino en
zigzag, franjas de hierba, escalones laterales, árboles perimetrales y cerca inferior
con abertura central. Incluye entrada, salida, nueve encuentros reproducibles,
estanques, flores, copas escalonadas y una región procedural 16×16. El avatar mide
0.62 bloques y usa intersección analítica con una caja pequeña, además del DDA del
terreno. Sus colisiones consultan cuatro esquinas sobre el suelo; no hay saltos ni
física de cuerpos rígidos. Las terrazas decorativas laterales son obstáculos.
El paseo automático está diseñado para el corredor principal; si lo activas fuera
de él y encuentras un obstáculo, vuelve al camino con WASD.

Tus sprites van en `assets/textures/characters/starter_1.ppm`, `starter_2.ppm` y
`starter_3.ppm` (PPM RGB, preferiblemente 32×32). Puedes cambiar cada cara con
los sufijos `_front`, `_back`, `_left`, `_right`, `_top`, `_bottom`.
Consulta [las instrucciones de personajes](assets/textures/characters/README.md).
Hay apariencias provisionales si no colocas archivos.

Para convertir tus PNG opacos:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools/png_to_ppm.ps1 -InputPath mi_sprite.png -OutputPath assets/textures/characters/starter_1.ppm
```

Capturas: `--preview --overview` encuadra la ruta completa. `--preview --night` conserva la demostración nocturna.

Las filas del framebuffer se reparten entre los procesadores lógicos disponibles mediante `std::thread::scope`. Al iniciar, la consola muestra cuántos workers se usan y cuántas scanlines recibe cada uno.

El camino, los escalones y los árboles principales siguen la distribución de las imágenes de referencia. La región procedural usa la semilla fija `0x524f_5554_4531`; los encuentros usan `42`. El ejecutable imprime por separado el tiempo de generación del terreno y el tiempo de render del frame. Las mediciones históricas están en [`PERFORMANCE.md`](PERFORMANCE.md) y [`bitacora.md`](bitacora.md).

Para generar una captura PPM con el mismo renderer sin abrir la ventana:

```bash
cargo run --release -- --preview
```

Para verificar el material emisivo con el cielo y la luz nocturnos:

```bash
cargo run --release -- --preview --night
```

Para renderizar los nueve materiales aislados en una cuadrícula de cubos:

```bash
cargo run --release -- --texture-preview
```

El set original de texturas 32×32 se reconstruye de forma determinista con:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools/generate_textures.ps1
```

La procedencia completa está en [`assets/TEXTURE_CREDITS.md`](assets/TEXTURE_CREDITS.md).

Para medir un frame mono-hilo y multihilo y comprobar que ambos buffers sean idénticos:

```bash
cargo run --release -- --benchmark
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
