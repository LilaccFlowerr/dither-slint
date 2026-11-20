---
title: Write your own dither
description: Build a background from a field.
---

A *field* is a Rust function that runs once for every cell and returns a tone from 0 to 1. 0 is paper, 1 is ink. The engine turns the tone into dots, colours them from the palette, and handles resizing, time and motion.

## 1. Register a field

```rust
use dither_slint::field::{self, Cell};
use dither_slint::glam::Vec2;
use dither_slint::noise::fbm;

field::register("rings", |cell: &Cell| {
    let aspect = cell.resolution.x / cell.resolution.y;
    let p = Vec2::new(cell.uv.x * aspect, cell.uv.y);
    let drop = Vec2::new(cell.params[0] * aspect, 1.0 - cell.params[1]);
    let wobble = fbm(p * 2.2 + Vec2::new(cell.time * 0.03, 0.0)) - 0.5;
    let d = p.distance(drop) + wobble * 0.08;
    let rings = 0.5 + 0.5 * (d * cell.params[2] - cell.time * 0.9).cos();
    cell.params[3] * rings * (-d * 2.6).exp()
});
```

A `Cell` has:

| Field | |
| --- | --- |
| `uv` | The cell centre from 0 to 1 on both axes, with y pointing up. |
| `resolution` | The size of the picture in cells. |
| `time` | Seconds, scaled by `speed`. |
| `params` | The numbers you pass from Slint. |
| `palette` | The palette, with `entry(i)` and `ramp(x)`. |

`uv` stretches with the box, so multiply x by the aspect ratio to get true distances.

## 2. Show it

```slint
DitherShader {
    field: "rings";
    params: [0.5, 0.42, 46, 0.85];
    palette: [DitherTheme.background, DitherPalette.blue];
    still-time: 4;
}
```

`DitherShader` also takes `matrix`, `levels`, `cell`, `speed`, `fps` (30 by default) and `animated`.

## Helpers

The `dither_slint::noise` module has `hash21`, `snoise` (simplex noise from -1 to 1), `fbm` (soft noise from 0 to 1) and `spectrum` (a rainbow). `dither_slint::glam` re-exports the vector types.

## A field with its own colours

Implement the `Field` trait to give the dots colours of their own. `tone` decides where the dots go, and `color` returns the colour of each dot. Cells without a dot are the first palette colour.

```rust
use dither_slint::field::{Cell, Field};
use dither_slint::glam::Vec4;

struct Rainbow;

impl Field for Rainbow {
    fn tone(&self, cell: &Cell) -> f32 { 0.5 }

    fn color(&self, cell: &Cell) -> Option<Vec4> {
        Some(Vec4::new(cell.uv.x, 0.2, 1.0 - cell.uv.x, 1.0))
    }
}
```

## Outside Slint

The engine also works on plain pixels. `dither_slint::diffuse::dither_image` dithers an RGBA picture, and `dither_slint::export` writes a dither as an SVG (`to_svg`) or a PNG (`to_png`).
