---
title: Introduction
description: "What dither-slint is, how it works and how these docs are organised."
---

dither-slint is a set of [Slint](https://slint.dev) components drawn in dither: a picture made of dots in one or a few colours, where density stands in for tone. There are buttons, inputs and sliders, charts, photos and logos, text effects and animated backgrounds, all in the same look.

## How it works

Every dithered picture is made in Rust. A component asks the engine for an image of the cells it needs, such as a gradient fading out over 40 by 12 cells, and Slint draws it with sharp edges. That is why you call `dither_slint::install!` once when you start your app, and why debug builds want `opt-level = 3` for dependencies. The [installation](../installation/) page has the steps.

The look is set by a few things you can change in one place:

- **Colours.** `DitherTheme` and `DitherPalette` hold them. Change them once and every component follows. See [Theme](../../reference/theme/).
- **Cell size.** Every component has a `cell` property for the size of one dot. Smaller is finer and slower, larger is chunkier.
- **Motion.** `animated` turns the motion off, per component or everywhere with `DitherTheme.animated`. The [motion](../../motion/overview/) page covers the hover, press and idle effects.
- **Pattern.** `matrix` picks the threshold pattern. See [Matrices](../../engine/matrices/).

## How the docs are organised

- **Components** are listed [by what they are for](../../components/). Each page opens with a live demo, then examples, then every property with its type, its default and what it does.
- **Backgrounds** are a [gallery](../../backgrounds/overview/) of animated fields that you place behind anything.
- **Engine** explains the pieces underneath and how to [write your own](../../engine/write-your-own/) field.
- **Reference** has the [types](../../reference/types/) that properties take and the [theme](../../reference/theme/) globals.

Property types in the property tables link to the type reference. A type written like `DitherFill` is an enum, so you write it as `DitherFill.dotted`.

## Credits

dither-slint is a port of Mvkweb's dither work. The [credits](../credits/) page has the details.
