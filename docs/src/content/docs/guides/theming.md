---
title: Theming
description: Set the colours and the motion of the whole app in one place.
---

Three globals hold what every component shares. Set them once and the whole app follows.

| Global | Holds |
| --- | --- |
| `DitherTheme` | The paper (`background`), the text (`foreground`), the default ink (`ink`) and the switch for motion (`animated`). |
| `DitherPalette` | Ready-made ink colours: `blue`, `purple`, `green`, `pink`, `orange`, `red` and `grey`. You read them, you do not set them. |
| `DitherMotion` | The default hover, press and idle effect, the matrix of the motion effects, and its `speed` and `intensity`. |

Every property is listed on the [Theme](../../reference/theme/) page.

## Set them in Slint

Assign them in the `init` callback of your window. It runs once, before the first frame:

```slint
import { DitherHover, DitherMotion, DitherPalette, DitherTheme } from "@dither/lib.slint";

export component AppWindow inherits Window {
    init => {
        DitherTheme.ink = DitherPalette.green;
        DitherMotion.hover = DitherHover.spotlight;
    }
}
```

## Set them from Rust

Export the global from your window, then set it after you create the window:

```slint
export { DitherEngine, DitherTheme } from "@dither/lib.slint";
```

```rust
use slint::ComponentHandle;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let app = AppWindow::new()?;
    dither_slint::install!(app);

    let theme = app.global::<DitherTheme>();
    theme.set_ink(slint::Color::from_rgb_u8(0x32, 0xb3, 0x64));
    theme.set_animated(false);

    app.run()
}
```

Use this when the colours come from a settings file or from the system.

## A light theme

The defaults are dark. For a light app, flip the paper and the text, and pick an ink that is dark enough on a light page:

```slint
init => {
    DitherTheme.background = #f6f3ee;
    DitherTheme.foreground = #1b1a19;
    DitherTheme.ink = #3b4fd8;
}
```

## A component can still choose

A property on a component beats the global. `DitherTheme.ink` is the colour a component uses when you do not say otherwise, and `color: DitherPalette.pink` on one button changes only that button. The same goes for `animated`: the global switch stills everything, and `animated: true` on one component turns that one back on.

## Turning motion off

Set `DitherTheme.animated` to `false` and every component stands still, including the hover, press and idle effects. This is the one switch to use for a reduced-motion setting.
