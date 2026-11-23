---
title: Outside Slint
description: Dither a picture in plain Rust and save it as a PNG or an SVG.
---

The engine does not need a window. You can hand it a picture, get a dithered picture back, and write it to a file, in a build script, a command line tool or a server. Add the crate and a PNG reader to your `Cargo.toml`:

```toml
[dependencies]
dither-slint = "1"
png = "0.18"
```

## Dither a picture

`dither_image` takes pixels as bytes, red, green, blue and alpha for each pixel, row after row, and returns the dithered pixels in the same layout:

```rust
use std::fs::{self, File};
use std::io::BufReader;

use dither_slint::diffuse::{Method, Options, dither_image};
use dither_slint::export::{Source, SvgOptions, to_png, to_svg};

fn main() {
    let mut decoder = png::Decoder::new(BufReader::new(File::open("photo.png").unwrap()));
    decoder.set_transformations(png::Transformations::normalize_to_color8() | png::Transformations::ALPHA);
    let mut reader = decoder.read_info().unwrap();
    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
    let frame = reader.next_frame(&mut pixels).unwrap();
    let (width, height) = (frame.width as usize, frame.height as usize);

    let options = Options {
        method: Method::Atkinson,
        palette: vec![[0.1, 0.1, 0.1], [0.9, 0.89, 0.86]],
        ..Options::default()
    };
    let dithered = dither_image(&pixels, width, height, &options);

    fs::write("out.png", to_png(&dithered, width, height, 2).unwrap()).unwrap();

    let source = Source::Pixels { data: &dithered, width, height };
    fs::write("out.svg", to_svg(&source, &SvgOptions { cell: 2.0, ..SvgOptions::default() })).unwrap();
}
```

The `Transformations` line makes the PNG reader hand over 8-bit RGBA whatever the file contains, which is the layout the engine wants.

## Options

| Option | |
| --- | --- |
| `method` | `Bayer4`, `Bayer8`, `BlueNoise`, `FloydSteinberg`, `Atkinson` or `SierraLite`. The first three use a [matrix](../matrices/), the others spread the error to the neighbours and suit photos. The default is `FloydSteinberg`. |
| `palette` | The colours to dither to, each as red, green and blue from 0 to 1. Two colours is the classic look, more is allowed. |
| `threshold` | The brightness cut-off, 0.5 by default. Lower is lighter, higher is darker. |
| `serpentine` | Whether error diffusion runs left to right on one row and right to left on the next, which hides streaks. On by default. |

## Save it

`to_png` writes the picture as PNG bytes. The last number, here 2, scales every pixel up to a block of 2 by 2, which keeps the dots sharp when the picture is shown larger.

`to_svg` writes the same picture as vector shapes, one path per colour, with runs of the same colour merged into one shape, so the file stays small. `SvgOptions::cell` is the size of one pixel in the SVG, and `background` turns the most common colour into one big rectangle.
