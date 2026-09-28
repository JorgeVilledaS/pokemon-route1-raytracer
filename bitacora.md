# Bitácora de paralelismo y optimización

## Aventura ampliada (32×80)

Medición posterior con ruta larga, avatares analíticos, encuentros y sombras de
personajes, a 640×360, mismo protocolo de calentamiento y cinco frames:

| Workers | Tiempo promedio | Speedup |
| ---: | ---: | ---: |
| 1 | 222.77 ms | 1.00× |
| 12 | 33.86 ms | 6.58× |

Framebuffers idénticos byte por byte. Construcción de la ruta: 0.235 ms.
Son mediciones de esa cámara y equipo, no una garantía para cualquier ángulo.
La ventana continúa a 480×270. Se conserva el DDA para el terreno; solo los diez
actores como máximo usan cajas analíticas independientes. La simulación modifica
sus posiciones antes del render y los workers reciben la escena inmutable.

La prueba de aventura ejecuta el paseo hasta la salida y comprueba las nueve
capturas. Otra prueba comprueba bloqueo de agua/borde, y otra verifica tamaño,
cara y UV del avatar pequeño.

Medición del 27 de septiembre de 2026 con perfil `release`, resolución 640×360 y exactamente la misma cámara, escena, materiales y profundidad recursiva. Se ejecutó un frame de calentamiento y luego se promediaron cinco frames. El equipo expone 12 procesadores lógicos mediante `std::thread::available_parallelism()`.

| Workers | Tiempo promedio | Speedup |
| ---: | ---: | ---: |
| 1 | 106.61 ms | 1.00× |
| 12 | 15.54 ms | 6.86× |

El benchmark también compara ambos framebuffers completos: el resultado mono-hilo y multihilo fue idéntico byte por byte. Se puede repetir con:

```text
cargo run --release -- --benchmark
```

## Diseño elegido

El framebuffer se divide en tiles de 32×32. El `TileScheduler` agrupa filas completas de tiles y `std::thread::scope` presta a cada worker una porción mutable, contigua y disjunta del framebuffer. La escena, cámara y biblioteca de materiales se comparten únicamente mediante referencias inmutables. No hacen falta `Arc`, `Mutex` ni código `unsafe`; el compilador comprueba que dos workers no puedan escribir el mismo pixel.

Se eligió planificación estática con `scope` porque un frame tiene miles de rayos independientes y tiles de tamaño uniforme. Un pool persistente con `mpsc` reduciría creación de hilos y equilibraría mejor escenas extremadamente desiguales, pero añadiría una cola, sincronización y mayor complejidad. Para este render académico, el reparto estático ya consigue 6.86× y mantiene el código auditable.

## Optimizaciones activas

- El rayo se recorta primero contra el AABB de toda la escena; si lo falla, el DDA no visita ninguna celda.
- Los shadow rays usan `occluded`: terminan con el primer voxel opaco y no construyen `Hit`, UV ni color.
- No se crean `Vec`, locks ni objetos dinámicos dentro del ciclo por pixel/rayo.
- La ventana reutiliza su framebuffer mediante `render_into`; mover la cámara no vuelve a reservarlo.
- La biblioteca de materiales es un array indexado por `MaterialId`, evitando hashing en el hot path.
- El DDA recorre únicamente las celdas atravesadas por el rayo, no el volumen voxel completo.

El speedup no llega a 12× porque hay coste de creación/coordinación de hilos, trabajo desigual entre tiles y partes seriales como la presentación de la ventana. Aun así, reduce el frame medido de 106.61 ms a 15.54 ms.
