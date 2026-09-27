# Texturas y parámetros de materiales

Todas las texturas de esta entrega son pixel-art original creado específicamente para este proyecto. Las referencias visuales de Ruta 1 se usaron para orientar la paleta nostálgica, el contraste y la legibilidad; no se copiaron tiles ni sprites de los juegos.

El código fuente de los materiales está en `src/materials/material_library.rs`. Se usa un array indexado por `MaterialId as usize` para evitar hashing en el hot path del raytracer.

| Material | Textura | `ks` | Shininess | `kr` | `kt` | IOR | Justificación |
|---|---|---:|---:|---:|---:|---:|---|
| Césped | `grass_top.ppm` | 0.08 | 8 | 0.00 | 0.00 | 1.00 | Superficie vegetal mayormente difusa, verde menta con pequeñas agrupaciones de hojas. |
| Tierra/camino | `dirt_path.ppm` | 0.03 | 4 | 0.00 | 0.00 | 1.00 | Camino seco, claro y casi completamente mate. |
| Madera | `bark.ppm` | 0.25 | 32 | 0.05 | 0.00 | 1.00 | Vetas verticales, brillo suave y reflexión mínima; usa `bark_normal.ppm`. |
| Agua | `water.ppm` | 0.85 | 128 | 0.20 | 0.85 | 1.33 | Ondas azules, highlight estrecho y parámetros físicos preparados para reflexión/refracción. |
| Roca | `rock.ppm` | 0.55 | 64 | 0.10 | 0.00 | 1.00 | Fragmentos gris-azulados con specular visible y reflexión moderada. |
| Letrero | `sign_wood.ppm` | 0.15 | 16 | 0.00 | 0.00 | 1.00 | Tablones cálidos con una emisión tenue preparada para escenas nocturnas. |

## Formato

Los assets se almacenan como PPM P3 de 8×8 para que sean auditables y editables sin herramientas externas. El loader propio también acepta PPM P6. El sampling usa wrap repetido e interpolación bilineal.
