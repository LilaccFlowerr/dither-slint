---
title: Palettes
description: Using more than two colours.
---

A palette is the list of colours a dither can use, from paper (tone 0) to ink (tone 1). By default it has two colours. Components that draw gradients, images and backgrounds take a `palette`:

```slint
DitherGradient {
    palette: [DitherTheme.background, DitherPalette.blue, DitherPalette.green, DitherTheme.foreground];
}
```

The tone is spread across the list and each cell is dithered between its two neighbours. Order matters more than hue: put the colours in the order of how much they stand out against the paper.

## Levels

`levels` snaps the tone to a fixed number of steps before dithering. The default is every tone the matrix can draw (17 for `bayer4`). Fewer levels give visible bands, a posterised look:

```slint
DitherGradient { levels: 4; }
```

## Cells and pixel scale

One dither cell is a square of `cell` pixels. Small cells read as texture and large ones as pixel art. To keep the grid crisp the cell is rounded to whole device pixels, so a 3px cell is exactly 6 device pixels on a 2x screen.

| Cell | |
| --- | --- |
| 2 to 3px | Fine grain. A good default behind text. |
| 4 to 6px | Clearly a dither from across the room. |
| 8px and up | Pixel art. Pair it with fewer `levels`. |
