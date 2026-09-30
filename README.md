# Ruta 1 — Raytracer en Rust

Proyecto académico que recrea una ruta inspirada en la Ruta 1 de Pokémon como un diorama de bloques. Todo se dibuja con un raytracer hecho en Rust y ejecutado en CPU.

La aplicación abre una ventana interactiva donde se puede recorrer la ruta, mover la cámara en 360 grados, acercar o alejar la vista y cambiar entre día y noche.

## Video del funcionamiento
[ Ver video del funcionamiento](https://www.youtube.com/watch?v=ynFw0PkSKcc)

## Características

- Ruta alargada de `48 x 80` bloques, con entrada, salida y camino en zigzag.
- Árboles, hierba alta, flores, cercas, desniveles, agua y letreros.
- Personaje pequeño con apariencia inicial de Poké Ball.
- Nueve encuentros colocados a lo largo del recorrido.
- Al tocar un encuentro, el personaje captura y puede usar esa nueva apariencia.
- Cámara orbital libre, zoom y modo de seguimiento.
- Ciclo visual de día y noche.
- Texturas pixel art con filtrado bilineal y repetición.
- Sombras, iluminación Blinn-Phong, reflejos, refracción, Fresnel, mapas normales y materiales emisivos.
- Cielo procedural con gradiente y nubes.
- Terreno procedural determinista: la misma semilla produce el mismo resultado.
- Intersección de voxeles mediante DDA, sin revisar todos los cubos uno por uno.
- Render paralelo por franjas usando todos los núcleos disponibles del procesador.

## Requisitos

- Windows 10 u 11.
- Rust estable con Cargo.
- No se usan crates externos; el proyecto depende únicamente de la biblioteca estándar.

## Ejecutar

Desde la carpeta del proyecto:

```powershell
cargo run --release
```

Se recomienda usar siempre `--release`, porque el raytracer realiza muchos cálculos por cuadro.

## Controles

| Control | Acción |
| --- | --- |
| `W`, `A`, `S`, `D` | Mover al personaje con respecto a la cámara |
| `Espacio` | Activar o detener el recorrido automático |
| `1`, `2`, `3` | Usar una apariencia capturada |
| Arrastrar con el mouse | Girar la cámara libremente |
| Flechas | Girar la cámara con el teclado |
| Rueda del mouse | Acercar o alejar la vista |
| `Q`, `E` | Zoom con el teclado |
| `Tab` | Alternar entre cámara de seguimiento y vista general |
| `N` | Cambiar entre día y noche |
| `P` | Reiniciar la partida |
| `R` | Restablecer la cámara |
| `Esc` | Cerrar la aplicación |

El objetivo sencillo es entrar por la parte inferior, recorrer la ruta hasta la salida superior y encontrar las nueve criaturas.

## Apariencias personalizadas

Las imágenes del personaje se colocan en:

```text
assets/textures/characters/
```

Los archivos principales son:

```text
pokeball.ppm
starter_1.ppm
starter_2.ppm
starter_3.ppm
```

Formato recomendado: PPM `P6`, cuadrado, de `16 x 16` o `32 x 32` píxeles.


Ejemplo: `starter_1_front.ppm`. Si una cara no tiene archivo propio, se usa la imagen principal o una apariencia provisional generada por el programa.

## Comandos útiles

Generar una vista previa de la ruta:

```powershell
cargo run --release -- --preview --overview
```

Generar una vista previa nocturna:

```powershell
cargo run --release -- --preview --night
```

Generar una lámina con las texturas:

```powershell
cargo run --release -- --texture-preview
```

Medir el rendimiento monohilo y multihilo:

```powershell
cargo run --release -- --benchmark
```

Ejecutar las pruebas y revisar el código:

```powershell
cargo test --release
cargo clippy --release -- -D warnings
```

La entrega fue verificada con 44 pruebas unitarias. El benchmark también comprueba que el resultado monohilo y multihilo sea idéntico.

## Estructura principal

```text
src/core/       Vectores, rayos, cámara e imagen PPM
src/scene/      Voxeles, ruta, terreno y jugabilidad
src/materials/  Materiales y biblioteca de texturas
src/render/     DDA, iluminación, trazado y paralelismo
src/platform/   Ventana y controles de Windows
src/utils/      Carga de texturas y ruido procedural
assets/         Texturas y apariencias del personaje
tools/          Herramientas auxiliares para los assets
```

## Decisiones de rendimiento

- El render trabaja a una resolución interna menor y escala la imagen en la ventana.
- Cada hilo escribe solamente sus propias líneas del framebuffer, por lo que no necesita `Mutex`.
- El DDA entra primero al límite de la escena y termina apenas encuentra un bloque.
- Los rayos de sombra se detienen en el primer obstáculo.
- No se crean vectores ni buffers nuevos dentro del ciclo de cada píxel.
- La cámara y la escena se comparten como referencias inmutables entre hilos.

## Limitaciones

- La ventana interactiva usa la API nativa de Windows.
- El personaje camina sobre el terreno, pero no incluye saltos ni física compleja.
- El recorrido automático está pensado para el camino principal; el movimiento manual permite explorar libremente.
- El rendimiento depende de la cantidad de núcleos y de la velocidad del procesador.