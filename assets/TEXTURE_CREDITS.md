# Créditos y procedencia de texturas

La aventura añade motivos UV originales calculados en `src/render/shading.rs`
(flores coral, hojas facetadas y hierba alta). Estos se combinan con las texturas
PPM existentes. Las apariencias provisionales de los avatares se calculan en
`src/render/avatar.rs`; los sprites externos del usuario son opcionales y se
documentan por separado en `textures/characters/README.md`.

Resolución oficial: **32×32 píxeles**, PPM P6 RGB. Todo el set usa una paleta común de verdes menta/bosque, arena cálida, umber, blanco crema y gris azulado.

Todas las texturas son arte procedural original creado específicamente para este proyecto por `tools/generate_textures.ps1`. El script contiene los patrones y la paleta completos; no descarga, abre, transforma ni copia imágenes externas.

| Archivo | Uso | Procedencia |
| --- | --- | --- |
| `grass_top.ppm` | Césped corto | Original; patrón de brotes y grano propio |
| `grass_tall.ppm` | Césped alto procedural | Original; briznas diagonales más oscuras |
| `dirt_path.ppm` | Camino | Original; arena clara con piedras dispersas |
| `wood_fence.ppm` | Cerca blanca | Original; tablones pintados y desgaste |
| `wood_fence_normal.ppm` | Relieve de cerca | Original; calculado desde altura periódica |
| `bark.ppm` | Troncos | Original; vetas verticales de corteza |
| `bark_normal.ppm` | Relieve de corteza | Original; calculado desde altura periódica |
| `leaves.ppm` | Copa de árboles | Original; grupos de hojas en cuatro tonos |
| `sign_wood.ppm` | Letrero | Original; tablones cálidos con marco oscuro |
| `sign_glow.ppm` | Canal emisivo | Original; máscara negra con marca naranja |
| `water.ppm` | Estanque | Original; ondas azul/cian |
| `water_normal.ppm` | Ondulación del agua | Original; calculado desde ondas periódicas |
| `rock.ppm` | Roca | Original; gris azulado con vetas |

Referencias consultadas:

- Licencia general de Faithful: <https://www.faithfulpack.net/license>
- Licencia publicada para Faithful PBR: <https://www.curseforge.com/minecraft/texture-packs/faithful-pbr-64x/license>

Las referencias generales de Route 1 se limitan a ambiente, contraste y paleta nostálgica. No se extrajeron tiles, sprites ni archivos de una ROM o juego oficial.

## Reproducción

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools/generate_textures.ps1
```

El resultado es determinista: ejecutar de nuevo el script produce los mismos bytes.
