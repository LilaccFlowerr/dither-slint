---
title: Quick start
description: A running dither-slint app in four files.
---

dither-slint is a Slint library with a small Rust engine behind it, so it needs four things: the crate, a `build.rs` that tells the Slint compiler where the library is, one line in your `.slint` file, and one line in `main.rs`. Create a new project with `cargo new my-app` and use these files.

## Cargo.toml

```toml
[dependencies]
dither-slint = "1"
slint = "1.18"

[build-dependencies]
dither-slint = "1"
slint-build = "1.18"

[profile.dev.package."*"]
opt-level = 3
```

The crate is in both lists. Your app uses the engine, and your `build.rs` uses the path to the `.slint` files. The last block turns optimisation on for dependencies: the dots are computed on the CPU, and a debug build without it is slow.

## build.rs

```rust
fn main() {
    let config = slint_build::CompilerConfiguration::new()
        .with_library_paths(dither_slint::library_paths());
    slint_build::compile_with_config("ui/app.slint", config).unwrap();
}
```

`library_paths()` makes `@dither` mean "the dither-slint library" in your `.slint` files.

## ui/app.slint

```slint
import { DitherButton } from "@dither/lib.slint";
export { DitherEngine } from "@dither/lib.slint";

export component AppWindow inherits Window {
    width: 320px;
    height: 160px;

    DitherButton {
        text: "Deploy";
        clicked => {
            debug("deployed");
        }
    }
}
```

You import every name you use, including enums such as `DitherFill`. [Importing](../importing/) explains how, and every component page starts with the line to copy. The `export { DitherEngine }` line is what the next step needs.

## src/main.rs

```rust
slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let app = AppWindow::new()?;
    dither_slint::install!(app);
    app.run()
}
```

The dots are drawn in Rust, so `install!` connects the engine to your window. Call it once, right after you create the window and before `run()`.

## Run it

```bash
cargo run
```

You should see a button with a band of dots along its top and bottom edge. If the button is flat and has no dots, the engine is not connected: check the `install!` line and the `export { DitherEngine }` line. [Troubleshooting](../troubleshooting/) lists the other messages you can run into.

## Next

[Concepts](../concepts/) explains what cell, matrix, level, fill and bloom mean. [Theming](../theming/) sets the colours and the motion of the whole app in one place.
