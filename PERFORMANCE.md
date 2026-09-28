# Bitácora de rendimiento

Medición de referencia del 27 de septiembre de 2026, ejecutando el perfil `release` en el equipo de desarrollo. Los tiempos internos excluyen la compilación y la escritura del PPM.

| Prueba | Tiempo |
| --- | ---: |
| Generación procedural 16×16 (día) | 0.030 ms |
| Render día 640×360 | 17.7 ms |
| Generación procedural 16×16 (noche) | 0.029 ms |
| Render noche 640×360 | 16.2 ms |

Comandos usados:

```text
cargo run --release -- --preview
cargo run --release -- --preview --night
```

La generación usa una semilla `u64` explícita y no consulta reloj, estado global ni un RNG externo. El render divide scanlines entre los procesadores lógicos disponibles; por eso el tiempo de render variará según el equipo, mientras que el terreno producido permanecerá idéntico.
