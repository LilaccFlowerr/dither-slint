---
title: Matrices
description: The threshold patterns behind every dither.
---

A dither draws a tone with dots. For each cell the engine compares the tone to a threshold from a small repeating grid, the matrix. If the tone is above the threshold the cell is ink.

| Matrix | |
| --- | --- |
| `bayer2`, `bayer4`, `bayer8` | Ordered patterns of 2, 4 and 8 cells square. Bigger tiles draw more tones, with a regular crosshatch that reads as printed. `bayer4` is the default. |
| `blue-noise` | A 64 by 64 tile with no visible grid. It looks like film grain and suits large soft surfaces. |

Ordered dithering is stable: the same tone always gives the same dots, so animations stay calm. A cell only changes when its tone crosses its threshold.

Every component that draws dots has a `matrix` property.
