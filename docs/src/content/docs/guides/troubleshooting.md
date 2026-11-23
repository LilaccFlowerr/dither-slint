---
title: Troubleshooting
description: What the common errors mean and how to fix them.
---

## The button or card is flat and has no dots

The engine is not connected. The component is drawn, but nothing fills it with dots, and there is no error. Check both halves:

```slint
export { DitherEngine } from "@dither/lib.slint";
```

in the `.slint` file of your window, and

```rust
dither_slint::install!(app);
```

in `main.rs`, after you create the window and before `run()`. See [Quick start](../installation/).

## Cannot access id 'DitherSomething'

```text
error: Cannot access id 'DitherScrambleMode'
```

The name is used in your `.slint` file but not imported. Add it to the import line, or copy the line from the component page. Check the spelling too: `DitherScrambleTrigger` is not `DitherScrambeTrigger`. See [Importing](../importing/).

## The Rust build fails at `install!`

```text
error[E0425]: cannot find type `DitherEngine` in this scope
    --> src/main.rs:5:5
     |
   5 |     dither_slint::install!(app);
```

Often followed by many warnings such as `use of deprecated type alias 'DitherProgressSpan': ... is not part of the public API`. They all have one cause: the `.slint` file of your window does not export the engine. Add this to it and the errors and the warnings go away:

```slint
export { DitherEngine } from "@dither/lib.slint";
```

## Cannot find requested import "@dither/lib.slint"

```text
error: Cannot find requested import "@dither/lib.slint" in the library search path
```

Your `build.rs` does not pass the library paths to the compiler. It has to look like this:

```rust
let config = slint_build::CompilerConfiguration::new()
    .with_library_paths(dither_slint::library_paths());
slint_build::compile_with_config("ui/app.slint", config).unwrap();
```

`dither-slint` must also be in `[build-dependencies]`, not only in `[dependencies]`.

## The debug build is slow

The dots are computed on the CPU, and a debug build without optimisation is slow. Turn it on for dependencies only:

```toml
[profile.dev.package."*"]
opt-level = 3
```

## Nothing moves

Motion follows `DitherTheme.animated`. If you set it to `false` somewhere, every component stands still unless it sets `animated: true` itself. See [Theming](../theming/).
