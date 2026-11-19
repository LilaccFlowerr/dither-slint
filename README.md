# dither-slint

Thanks to MVKweb for the idea and allowing me to make such a thing

Dithered UI components for [Slint](https://slint.dev), based on the dither work of [Mvkweb](https://github.com/Mvkweb).

> Work in progress. The dither engine is ported, with `DitherGradient` and `DitherButton` as the first components. More are on the way.

## Install

```toml
[dependencies]
dither-slint = "0.1"

[build-dependencies]
dither-slint = "0.1"
slint-build = "1.18"
```

Point `slint-build` at the library in your `build.rs`:

```rust
fn main() {
    let config = slint_build::CompilerConfiguration::new()
        .with_library_paths(dither_slint::library_paths());
    slint_build::compile_with_config("ui/app.slint", config).unwrap();
}
```

## Usage

```slint
import { DitherGradient, DitherMatrix } from "@dither/lib.slint";
export { DitherEngine } from "@dither/lib.slint";

export component App inherits Window {
    DitherGradient {
        angle: 90deg;
        matrix: DitherMatrix.blue-noise;
    }
}
```

The gradients are drawn in Rust, so connect the engine once after creating your window:

```rust
slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    dither_slint::install!(app);
    app.run()
}
```

Run the demos with `cargo run --example dither_gradient` and `cargo run --example dither_button`.

### DitherGradient

| Property | Default | |
| --- | --- | --- |
| `angle` | `180deg` | Direction, as in CSS. 180deg runs top to bottom. |
| `from` | `0` | Tone at the start, from 0 (paper) to 1 (ink). |
| `to` | `1` | Tone at the end. |
| `matrix` | `bayer4` | `bayer2`, `bayer4`, `bayer8` or `blue-noise`. |
| `cell` | `3px` | Size of one dither cell, rounded to whole device pixels. |
| `ink` | `DitherTheme.foreground` | Colour of the dots. |
| `paper` | `DitherTheme.background` | Colour behind them. |

### DitherButton

A button with a lit, dithered surface: a highlight dithered down from the top and a shade dithered up from the bottom. Hover brightens it, press shades it from the top.

```slint
DitherButton { text: "Deploy"; bloom: DitherBloom.low; clicked => { deploy(); } }
DitherButton { text: "View logs"; variant: DitherButtonVariant.outline; fill: DitherFill.dotted; }
```

| Property | Default | |
| --- | --- | --- |
| `text` | | The label. |
| `icon` | | An image shown before the label, tinted like the text. |
| `variant` | `filled` | `filled` is the lit colour surface, `outline` a faint tint that fills in on hover. |
| `fill` | `gradient` | Texture of the shade: `gradient`, `dotted`, `hatched` or `solid`. |
| `color` | `DitherTheme.ink` | Any colour, or one from `DitherPalette`. |
| `size` | `md` | `sm`, `md` and `lg` are 32, 36 and 44px high. `icon` is a 36px square. |
| `bloom` | `off` | `low` or `high` adds a glow in the button's colour. |
| `cell` | `2px` | Size of one dither cell. |
| `loading` | `false` | Steps three cells before the label and ignores clicks. |
| `enabled` | `true` | |

## What is a dither

A dither draws a smooth tone using only a few colours. Dots of ink are placed at a density that matches the tone, so your eye blends them into a gradient. It started as a workaround for limited screens and printers and now reads as a deliberate texture, a bit like risograph print.

## Credits

- [Mvkweb](https://github.com/Mvkweb), [dither-sdk.vercel.app](https://dither-sdk.vercel.app). This library is a port of dither-sdk by mvk

## License

GPL 3.0
