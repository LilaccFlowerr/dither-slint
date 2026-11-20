# dither-slint

Thanks to MVKweb for the idea and allowing me to make such a thing

Dithered UI components for [Slint](https://slint.dev), based on the dither work of [Mvkweb](https://github.com/Mvkweb).

> Work in progress. The dither engine is ported, with `DitherGradient` and `DitherButton` as the first components. More are on the way.

## Install

```toml
[dependencies]
dither-slint = "0.3"

[build-dependencies]
dither-slint = "0.3"
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

Run the gallery with `cargo run --example gallery`.

### DitherGradient

| Property | Default | |
| --- | --- | --- |
| `angle` | `180deg` | Direction, as in CSS. 180deg runs top to bottom. |
| `from` | `0` | Tone at the start, from 0 (paper) to 1 (ink). |
| `to` | `1` | Tone at the end. |
| `matrix` | `bayer4` | `bayer2`, `bayer4`, `bayer8` or `blue-noise`. |
| `cell` | `3px` | Size of one dither cell, rounded to whole device pixels. |
| `palette` | background, foreground | Two or more colours, from paper (tone 0) to ink (tone 1). The tone is spread over the list and each cell is dithered between two neighbours. |
| `levels` | `0` | Snaps the tone to this many steps for a posterised look. `0` keeps every tone the matrix can draw. |

### DitherButton

A button with a lit, dithered surface: a highlight dithered down from the top and a shade dithered up from the bottom. Hover brightens it, press shades it from the top.

```slint
DitherButton {
    text: "Deploy";
    bloom: DitherBloom.low;
    clicked => {
        debug("clicked");
    }
}

DitherButton {
    text: "View logs";
    variant: DitherButtonVariant.outline;
    fill: DitherFill.dotted;
}
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
| `animated` | `DitherTheme.animated` | A dithered sweep on hover and a ripple from where you press. |

Turn the motion off everywhere with `DitherTheme.animated = false`, or per button with `animated: false`. `DitherFx` is the layer behind it, so other components can use the same sweep and ripple.

### DitherTabs

Tabs whose active indicator is a dithered pill or underline that glides to the chosen tab, dissolving out of the old one and in at the new one through Bayer matrix levels.

```slint
DitherTabs {
    tabs: ["Overview", "Deploys", "Settings"];
    current: 0;
    variant: DitherTabsVariant.pill;
    color: DitherPalette.blue;
    bloom: DitherBloom.low;
}
```

| Property | Default | |
| --- | --- | --- |
| `tabs` | `["Overview", "Deploys", "Settings"]` | Array of tab labels. |
| `current` | `0` | Index of the active tab. |
| `variant` | `pill` | `pill` lights a dithered surface behind the label; `underline` draws a dithered bar with a rising glow. |
| `color` | `DitherTheme.ink` | Indicator colour, e.g. from `DitherPalette`. |
| `matrix` | `bayer4` | `bayer2`, `bayer4`, `bayer8` or `blue-noise`. |
| `bloom` | `off` | `off`, `low` or `high` adds a soft glow in the indicator's colour. |
| `cell` | `2px` | Size of one dither cell. |
| `animated` | `DitherTheme.animated` | Glides the indicator between tabs over 260ms. Arrow keys step without lag. |
| `enabled` | `true` | Allows mouse and keyboard navigation. |

## What is a dither

A dither draws a smooth tone using only a few colours. Dots of ink are placed at a density that matches the tone, so your eye blends them into a gradient. It started as a workaround for limited screens and printers and now reads as a deliberate texture, a bit like risograph print.

## Credits

- [Mvkweb](https://github.com/Mvkweb), [dither-sdk.vercel.app](https://dither-sdk.vercel.app). This library is a port of dither-sdk by mvk

## License

GPL 3.0
