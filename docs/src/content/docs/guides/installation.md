---
title: Installation
description: Add dither-slint to a Slint app.
---

## Add the crate

```toml
[dependencies]
dither-slint = "0.4"

[build-dependencies]
dither-slint = "0.4"
slint-build = "1.18"
```

The dithering runs on the CPU in Rust, so turn optimisation on for your dependencies. Without it a debug build is slow:

```toml
[profile.dev.package."*"]
opt-level = 3
```

## Point the Slint compiler at the library

In your `build.rs`:

```rust
fn main() {
    let config = slint_build::CompilerConfiguration::new()
        .with_library_paths(dither_slint::library_paths());
    slint_build::compile_with_config("ui/app.slint", config).unwrap();
}
```

## Use a component

```slint
import { DitherButton } from "@dither/lib.slint";
export { DitherEngine } from "@dither/lib.slint";

export component App inherits Window {
    DitherButton { text: "Deploy"; }
}
```

## Connect the engine

The pictures are drawn in Rust, so connect the engine once after you create the window:

```rust
slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    dither_slint::install!(app);
    app.run()
}
```

Without `export { DitherEngine }` in your `.slint` file and the `install!` call, the components draw nothing.

## Colours

The components read their colours from `DitherTheme` and `DitherPalette`. Change them once at start-up:

```slint
DitherTheme.background = #1b1a19;
DitherTheme.foreground = #ebe5d9;
DitherTheme.ink = #5471f5;
```

Turn all motion off with `DitherTheme.animated = false`.
