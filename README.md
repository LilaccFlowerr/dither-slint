# dither-slint

Thanks to MVKweb for the idea and allowing me to make such a thing

Dithered UI components for [Slint](https://slint.dev), based on the dither work of [Mvkweb](https://github.com/Mvkweb).

> Work in progress. More components are on the way.

## docs
https://lilaccflowerr.github.io/dither-slint/
the docs can be found here

## Install

```toml
[dependencies]
dither-slint = "0.5"

[build-dependencies]
dither-slint = "0.5"
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

## Components

- **Basics:** `DitherGradient`, `DitherButton`, `DitherBadge`, `DitherCard`, `DitherDivider`
- **Controls:** `DitherSwitch`, `DitherSlider`, `DitherInput`, `DitherTabs`, `DitherDropdownMenu`
- **Identity and feedback:** `DitherAvatar`, `DitherLoader`, `DitherProgressBar`, `DitherSkeleton`, `DitherEmptyState`, `DitherStatCard`, `DitherSparkline`, `DitherUptimeBar`, `DitherGauge`, `DitherNumber`

There are also `DitherImage` for dithering a photo, `DitherImageCompare` for a before and after slider, `DitherScrollReveal` for a picture that resolves as it scrolls into view, `DitherLogo` and `DitherLogoMarquee` for logos, `DitherTreemap` for a treemap, `DitherLineChart`, `DitherAreaChart`, `DitherBarChart`, `DitherDonutChart`, `DitherHeatmap`, `DitherPixelArea` and `DitherRadarChart` for charts, `DitherText`, `DitherDissolveText` and `DitherScrambleText` for text, twenty-five animated backgrounds (`DitherAnimatedGradient`, `DitherAurora`, `DitherBeams`, `DitherContours`, `DitherDotMap`, `DitherEclipse`, `DitherFade`, `DitherGlobe`, `DitherHalftone`, `DitherHorizon`, `DitherLightRays`, `DitherMeshGradient`, `DitherMetaballs`, `DitherNoiseField`, `DitherOrb`, `DitherPlasma`, `DitherRetroGrid`, `DitherRidgelines`, `DitherRipples`, `DitherSilk`, `DitherSmoke`, `DitherSpotlight`, `DitherStarfield`, `DitherTunnel` and `DitherWaves`) with `DitherShader` to write your own, and `DitherFx` for hover, press and idle effects. The docs in `docs/` show every one of them live. Build the engine for the browser with `./engine-web/build.sh`, then run `npm install` and `npm run dev` in `docs/`.

Colours come from `DitherTheme` and `DitherPalette`. Turn motion off everywhere with `DitherTheme.animated = false`.

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

### DitherBadge

A compact label with a lit, dithered surface.

```slint
DitherBadge {
    text: "Online";
    dot: true;
    color: DitherPalette.green;
}

DitherBadge {
    text: "Tag";
    removable: true;
    removed => {
        debug("removed");
    }
}
```

| Property | Default | |
| --- | --- | --- |
| `text` | | The label. |
| `variant` | `default` | `default`, `secondary`, `destructive` or `outline`. Destructive turns red unless you set a colour. |
| `fill` | `gradient` | `gradient`, `dotted`, `hatched` or `solid`. |
| `color` | `DitherTheme.ink` | Any colour, or one from `DitherPalette`. |
| `size` | `md` | `sm`, `md` or `lg`, 20, 24 and 28px high. |
| `corners` | `md` | `none`, `sm`, `md` or `lg`. |
| `bloom` | `off` | `low` or `high`, for the filled variants. |
| `cell` | `2px` | Size of one dither cell. |
| `dot` | `false` | Shows a status dot. `dot-color` sets its colour. |
| `removable` | `false` | Shows a small cross. Clicking it calls `removed`. |
| `interactive` | `false` | Brightens on hover and calls `clicked`. |

### DitherCard

A card with a dithered glow in a corner or along an edge, and a dithered rim that is densest near the glow. Put your content inside.

```slint
DitherCard {
    glow: DitherGlow.top-left;
    interactive: true;

    Text { text: "Usage"; }
}
```

| Property | Default | |
| --- | --- | --- |
| `glow` | `top-left` | `top-left`, `top-right`, `bottom-left`, `bottom-right`, `top` or `bottom`. |
| `reach` | `240px` | How far the glow reaches from its origin. |
| `strength` | `0.5` | Densest level, at the origin, 0 to 1. |
| `interactive` | `false` | Grows the glow through the levels on hover. |
| `color` | `DitherTheme.ink` | Colour of the glow. |
| `fill` | `gradient` | Texture of the glow. |
| `matrix` | `bayer4` | The dither pattern. |
| `cell` | `3px` | Size of one dither cell. |
| `rim` | `true` | Light the border with dither. |
| `radius` | `14px` | Corner radius, which the rim follows. |
| `inset` | `24px` | Space between the edge and your content. |
| `spacing` | `16px` | Space between the children. |

### DitherImageCompare

A before and after slider over one photo: the left side is dithered, the right side is the original. Drag the divider, click anywhere, or use the arrow keys. The divider nudges once when it first appears.

```slint
DitherImageCompare {
    source: @image-url("cover.png");
    width: 400px;
    height: 225px;
    changed(value) => { debug(value); }
}
```

| Property | Default | |
| --- | --- | --- |
| `source` | | The picture. |
| `method` | `floyd-steinberg` | `bayer4`, `bayer8`, `blue-noise`, `floyd-steinberg`, `atkinson` or `sierra-lite`. |
| `threshold` | `0.5` | Brightness cut-off, 0 to 1. |
| `cell` | `2px` | Size of one dither cell. |
| `palette` | background, foreground | Two or more colours. |
| `before-label` | `DITHERED` | Label on the left side. |
| `after-label` | `ORIGINAL` | Label on the right side. |
| `labels` | `true` | Show the corner labels. |
| `hint` | `true` | Nudge the divider once when it first appears. |
| `value` | `50` | Divider position from the left, 0 to 100. |

`changed(float)` fires with the new position while dragging or on key presses.

### DitherScrollReveal

A picture that starts as a few coarse dithered blocks, sharpens step by step and then dissolves cell by cell into the photo as it scrolls into view.

```slint
DitherScrollReveal {
    source: @image-url("cover.png");
    viewport-height: root.height;
    width: 400px;
    height: 225px;
}
```

| Property | Default | |
| --- | --- | --- |
| `source` | | The picture. |
| `method` | `floyd-steinberg` | `bayer4`, `bayer8`, `blue-noise`, `floyd-steinberg`, `atkinson` or `sierra-lite`. |
| `threshold` | `0.5` | Brightness cut-off, 0 to 1. |
| `cell` | `2px` | Size of one dither cell. |
| `palette` | background, foreground | Two or more colours. |
| `end` | `photo` | `photo` ends on the picture itself, `dither` on the finest dither. |
| `coarsest` | `24` | How many cells wide the first blocks are. |
| `start` | `1` | Where the top edge is when the reveal starts, as a share of the scroll area from the top. |
| `finish` | `0.3` | Where the top edge is when the reveal is done. |
| `viewport-height` | `0` | Height of the scroll area. Set it and the reveal follows the scroll. |
| `viewport-top` | `0` | Where the scroll area starts in the window. |
| `progress` | `0` | The reveal by hand, 0 to 1. Used when `viewport-height` is `0`. |

### DitherLogo

A logo drawn in dithered ink. Edges land on whole cells, so the shapes stay sharp while the inside takes a dither texture, with an optional shimmer or scan line. Use a PNG with a transparent background.

```slint
DitherLogo {
    source: @image-url("logo.png");
    fill: DitherFill.dotted;
    effect: DitherLogoEffect.shimmer;
    width: 120px;
    height: 44px;
}
```

| Property | Default | |
| --- | --- | --- |
| `source` | | The logo. |
| `ink` | `DitherTheme.ink` | Colour of the ink when `tone` is `mono`. |
| `tone` | `mono` | `mono` is one ink, `original` keeps the file's own colours. |
| `logo-source` | `alpha` | `alpha` reads the transparent background, `dark` reads a dark logo on a light background. |
| `fill` | `gradient` | `gradient`, `dotted`, `hatched` or `solid`. |
| `outline` | `true` | Keep a solid outline one cell wide. |
| `floor` | `0.3` | Gradient fill: how much ink is left at the bottom, 0 to 1. |
| `matrix` | `bayer4` | Threshold pattern of the gradient fill. |
| `cell` | `1px` | Size of one cell. |
| `effect` | `none` | `none`, `shimmer` or `scan`. |
| `period` | `6s` | Time per sweep. |
| `phase` | `0` | Where in the sweep this logo starts, 0 to 1. |

### DitherLogoMarquee

A row of dithered logos that loops without a seam, coasts to a stop on hover and has a pause button. `variant: DitherMarqueeVariant.wall` makes a still grid with hairline dividers instead.

```slint
DitherLogoMarquee {
    logos: [
        { source: @image-url("ring.png") },
        { source: @image-url("peak.png") },
    ];
}
```

The logo properties above (`ink`, `tone`, `fill`, `outline`, `floor`, `matrix`, `cell`, `effect` and `period`) apply to every logo.

| Property | Default | |
| --- | --- | --- |
| `logos` | | The logos, each `{ source: image }`. |
| `variant` | `marquee` | `marquee` or `wall`. |
| `logo-height` | `28px` | Height of each logo. |
| `gap` | `64px` | Space between logos. |
| `speed` | `32px` | Scroll speed in pixels per second. |
| `direction` | `left` | `left` or `right`. |
| `fade` | `soft` | `soft`, `dither` or `none`. |
| `fade-width` | `96px` | Width of each end's fade. |
| `backdrop` | `DitherTheme.background` | The colour the ends fade to. |
| `pause-button` | `true` | A button on hover that stops the motion. |

### DitherTreemap

A treemap that shows how a whole splits into parts: headcount by team, revenue by region, disk space, the parts of a context window. Each item is a tile sized by its value, and the values grow in when it appears. Hover a tile for its share.

```slint
DitherTreemap {
    title: "Headcount by team";
    unit: " people";
    items: [
        { label: "Engineering", value: 48, color: DitherPalette.blue, fill: DitherFill.gradient, detail: "Platform and product" },
        { label: "Sales", value: 32, color: DitherPalette.purple, fill: DitherFill.gradient, detail: "Direct and partners" },
        { label: "Support", value: 14, color: DitherPalette.pink, fill: DitherFill.gradient, detail: "Customer success" },
    ];
}
```

Mark one item `remainder: true` and it takes the space left of the `capacity`, on the right, hatched. It shrinks while the other values grow in.

| Property | Default | |
| --- | --- | --- |
| `title` | | Headline on the top left. |
| `items` | a sample | The tiles, each `{ label, value, color, fill, detail, remainder }`. |
| `capacity` | the sum of the values | The whole, in the same unit as the values. |
| `unit` | | Shown after every number. Values of a thousand or more are shown in thousands. |
| `status` | | Text in the footer while nothing is hovered. |
| `replay-label` | `Replay` | Text of the replay button. |
| `duration` | `6s` | How long the values take to grow in. |
| `animated` | `DitherTheme.animated` | `false` shows the finished treemap at once. |
| `interactive` | `true` | Tooltip and dimming on hover. |
| `spotlight` | `opacity` | `opacity` dims the other tiles, `none` leaves them. |
| `cell` | `3px` | Size of one dither cell. |
| `matrix` | `bayer4` | The dither pattern of the gradient fills. |
| `header`, `readout`, `footer` | `true` | Show the title row, the used and total readout, and the status line with the replay button. |
| `stage-height` | `256px` | Height of the treemap. |

### DitherLineChart

One or more crisp lines, each with an optional dithered range band. Hover for a readout, and use the legend to spotlight a series.

```slint
DitherLineChart {
    categories: ["Jan", "Feb", "Mar", "Apr", "May", "Jun"];
    series: [
        { label: "Revenue", color: DitherTheme.ink, values: [12, 18, 15, 24, 28, 26], low: [], high: [], dashed: false },
        { label: "Costs", color: DitherPalette.orange, values: [10, 12, 13, 15, 17, 18], low: [], high: [], dashed: true },
    ];
}
```

| Property | Default | |
| --- | --- | --- |
| `categories` | | The label of every point on the x axis. |
| `series` | | The lines, each `{ label, color, values, low, high, dashed }`. Give `low` and `high` for a band. |
| `markers` | `last` | `all`, `last` or `none`. |
| `zero` | `false` | Start the value axis at zero. |
| `smooth` | `true` | A smooth curve that never overshoots, or straight segments. |
| `axes`, `tooltip`, `legend`, `interactive` | `true` (legend with two or more series) | Show the axes, the readout and the legend. |
| `animated` | `DitherTheme.animated` | Draw the lines in from the left. |
| `unit` | | Written after every value. Values are shortened to 1.2K. |
| `level`, `matrix`, `cell` | `0.25`, `bayer4`, `3px` | Coverage, pattern and cell size of the bands. |
| `chart-height` | `240px` | Height of the chart, axes included. |

### DitherAreaChart

Each series is a crisp line over a dithered fill that is densest under the line and thins out towards the axis. Series can overlap, stack, or stack and scale to 100%.

```slint
DitherAreaChart {
    categories: ["Jan", "Feb", "Mar", "Apr", "May", "Jun"];
    stack: DitherChartStack.stacked;
    series: [
        { label: "Web", color: DitherTheme.ink, values: [12, 18, 15, 24, 28, 26], low: [], high: [], dashed: false },
        { label: "Mobile", color: DitherPalette.orange, values: [8, 10, 12, 14, 16, 18], low: [], high: [], dashed: false },
    ];
}
```

| Property | Default | |
| --- | --- | --- |
| `categories`, `series` | | As for `DitherLineChart`. Later series draw in front. |
| `stack` | `none` | `none`, `stacked` or `percent`. |
| `fill` | `gradient` | `gradient`, `dotted`, `hatched` or `solid`. |
| `level` | `0.625` | Gradient only: coverage of the densest band, 0 to 1. |
| `matrix`, `cell` | `bayer4`, `3px` | The dither pattern and the cell size. |
| `smooth`, `axes`, `tooltip`, `legend`, `interactive`, `animated`, `unit`, `chart-height` | | As for `DitherLineChart`. |

### DitherBarChart

Dithered bars under a solid cap: grouped, stacked, or stacked and scaled to 100%. With the default `gradient` fill a bar is densest at its top and thins out towards the axis. The highlighted bar, or the one under the pointer, fills in solid.

```slint
DitherBarChart {
    categories: ["Q1", "Q2", "Q3", "Q4"];
    stack: DitherChartStack.stacked;
    series: [
        { label: "Web", color: DitherTheme.ink, values: [42, 58, 70, 94], low: [], high: [], dashed: false },
        { label: "Mobile", color: DitherPalette.orange, values: [30, 44, 56, 78], low: [], high: [], dashed: false },
    ];
}
```

| Property | Default | |
| --- | --- | --- |
| `categories`, `series` | | As for `DitherLineChart`. |
| `stack` | `none` | `none` puts the series side by side, `stacked` on top of each other, `percent` scaled to 100%. |
| `highlight` | `last` | The bar drawn solid while nothing is hovered: `last`, `max` or `none`. |
| `highlight-at` | `-1` | The index of the bar to highlight. Overrides `highlight` when it is 0 or more. |
| `fill` | `gradient` | `gradient`, `dotted`, `hatched` or `solid`. |
| `level` | `0.75` | Gradient only: coverage at the top of each bar, 0 to 1. |
| `matrix`, `cell` | `bayer4`, `3px` | The dither pattern and the cell size. |
| `axes`, `tooltip`, `legend`, `interactive`, `animated`, `unit`, `chart-height` | | As for `DitherLineChart`. |

### DitherDonutChart

A donut or pie drawn in dither cells. Slices run clockwise from 12 o'clock, each with a colour, a texture and a solid rim, and sweep in when the chart appears. Hover a slice or a legend row to step it out and dim the others, and click to keep it lit.

```slint
DitherDonutChart {
    label: "Requests today";
    slices: [
        { label: "Web", value: 18420, color: DitherTheme.ink, fill: DitherFill.gradient },
        { label: "Mobile", value: 12310, color: DitherPalette.orange, fill: DitherFill.gradient },
        { label: "API", value: 8650, color: DitherPalette.green, fill: DitherFill.dotted },
    ];
}
```

| Property | Default | |
| --- | --- | --- |
| `slices` | | The slices, each `{ label, value, color, fill }`. |
| `label` | | What the total counts, shown in the middle of a donut. |
| `mode` | `donut` | `donut` or `pie`. |
| `size` | `220px` | Diameter of the chart. |
| `thickness` | `0.36` | Donut only: ring thickness as a share of the radius, 0.1 to 0.9. |
| `gap` | `1` | Gap between slices, in dither cells. |
| `unit` | | Written after every value. |
| `legend`, `interactive`, `animated` | `true`, `true`, `DitherTheme.animated` | The legend, the hover and click, and the sweep. |
| `matrix`, `cell` | `bayer4`, `3px` | The dither pattern and the cell size. |

### DitherHeatmap

A calendar or a grid of values drawn as squares of 4×4 dither cells, so a square's density steps from dots to a checkerboard, a hollow frame and solid. Hover a square for its value and use the legend to spotlight a density. The grid shrinks its cells to fit, then drops the oldest columns.

```slint
DitherHeatmap {
    unit: "deploy";
    units: "deploys";
    entries: [
        { date: "2026-11-20", value: 3 },
        { date: "2026-11-21", value: 7 },
        { date: "2026-11-22", value: 10 },
    ];
}

DitherHeatmap {
    mode: DitherHeatmapMode.matrix;
    rows: ["Mon", "Tue"];
    columns: ["8", "9", "10"];
    grid: [{ values: [2, 6, 9] }, { values: [3, 7, 10] }];
}
```

| Property | Default | |
| --- | --- | --- |
| `mode` | `calendar` | `calendar` or `matrix`. |
| `entries`, `weeks`, `week-start` | | Calendar: the days with their values (`YYYY-MM-DD`), the columns shown (53) and the first row (0 Sunday, 1 Monday). |
| `grid`, `rows`, `columns` | | Matrix: the values by row, and the labels. |
| `steps` | `4` | How many densities values map to, above empty, 2 to 6. |
| `max` | the largest value | The value drawn solid. `0` takes the largest. |
| `unit`, `units` | | What a value counts, singular and plural. |
| `ink`, `cell` | `DitherTheme.ink`, `3px` | Colour of the squares and the largest dither cell. |
| `legend`, `interactive`, `animated` | `true`, `true`, `DitherTheme.animated` | The scale, the readout and the ripple in. |

### DitherPixelArea

A stepped area chart painted one pixel per dither cell: solid at the floor, dissolving upward to a soft edge at the value. Every pixel is the one colour at a different opacity, so it sits on light and dark backgrounds alike. Hovering lifts the fill and a marker reads the value. A few stars sit in the fill and wink while the page is animated.

```slint
DitherPixelArea {
    values: [12, 18, 15, 24, 28, 26, 34, 38, 36, 44, 48, 52];
    labels: ["Mon", "Tue", "Wed"];
    fill: DitherFill.dotted;
    ink: DitherPalette.orange;
}
```

| Property | Default | |
| --- | --- | --- |
| `values`, `labels` | | The values, oldest first, and a label per value for the readout. |
| `ink` | `DitherPalette.blue` | Colour of the area. |
| `fill` | `gradient` | `gradient`, `dotted`, `hatched` or `solid`. |
| `cell` | `2px` | Size of one dither cell. |
| `min`, `max` | `0`, the largest value | The bottom and top of the value axis. |
| `interactive`, `animated` | `true`, `DitherTheme.animated` | The hover lift and marker, and the sweep in. |
| `stars`, `winking` | `true`, `DitherTheme.animated` | The stars in the fill, and whether they wink. |

### DitherRadarChart

A radar chart drawn in dither cells. Each series is a polygon of whole cells with a solid outline, and the gradient fill steps up in density ring by ring so the grid reads straight through it. Move the pointer around the chart to read an axis, and use the legend to spotlight a series.

```slint
DitherRadarChart {
    axes: ["Speed", "Power", "Range", "Armor", "Stealth", "Agility"];
    series: [
        { label: "Falcon", values: [82, 60, 74, 40, 55, 90], color: DitherTheme.ink, fill: DitherFill.gradient },
        { label: "Bear", values: [40, 92, 50, 88, 20, 35], color: DitherPalette.orange, fill: DitherFill.dotted },
    ];
}
```

| Property | Default | |
| --- | --- | --- |
| `axes` | | The axis labels, clockwise from 12 o'clock. |
| `series` | | The series, each `{ label, values, color, fill }`. Later ones draw on top. |
| `max` | a round number above the largest value | The value at the outer ring. `0` picks it. |
| `rings`, `grid` | `4`, `polygon` | The grid rings, and `polygon` or `circle`. |
| `size` | `360px` | The largest width and height. |
| `unit`, `legend`, `interactive`, `animated` | | The unit in the readout, the legend, the hover and the grow-in. |
| `matrix`, `cell` | `bayer4`, `3px` | The dither pattern and the cell size. |

### Text effects

Three effects for display text. Slint cannot clip a picture to the shape of letters, so each draws its dither as a layer over the text that hides the cells that should be empty. That works on a flat background: set `backdrop` to the colour behind the text, which is the theme background by default.

`DitherText` fills the letters with a dither: a gradient that thins out in one direction, drifting noise, dots or hatching.

```slint
DitherText {
    text: "Gradient";
    fill: DitherTextFill.dotted;
    ink: DitherPalette.green;
}
```

`DitherDissolveText` dissolves in through the levels of a Bayer matrix, one character or word at a time, in reading order, reversed, from the middle out, from both ends in, or shuffled. `showing: false` dissolves it out again.

```slint
DitherDissolveText {
    text: "Dissolves in from the middle";
    order: DitherDissolveOrder.center;
    fill: DitherDissolveFill.dotted;
}
```

`DitherScrambleText` is a decode effect with no layout shift: each character resolves from a dithered block through a few glyphs to its letter. Changing `text` scrambles only the characters that differ.

```slint
DitherScrambleText {
    text: "Decoding the message";
    trigger: DitherScrambleTrigger.hover;
    completed => { debug("done"); }
}
```

| Property | | |
| --- | --- | --- |
| `text`, `font-size`, `font-weight`, `ink`, `backdrop` | | The text, the type, the colour and the flat colour behind it. |
| `fill`, `direction`, `from`, `to`, `scale`, `ghost` | | `DitherText`: what fills the letters and how. |
| `order`, `word`, `showing`, `level`, `duration`, `stagger`, `delay`, `speed` | | `DitherDissolveText`: the order, units and timing. |
| `trigger`, `mode`, `characters`, `text-color`, `rate` | | `DitherScrambleText`: when it plays, how unsettled characters look and the glyphs. |
| `cell`, `matrix`, `animated` | | The dither cell, the pattern and the motion switch. |

### DitherDivider

A rule that fades out towards its ends through dither levels, with an optional label in the middle.

```slint
DitherDivider {
    label: "or";
}

DitherDivider {
    orientation: DitherOrientation.vertical;
}
```

| Property | Default | |
| --- | --- | --- |
| `orientation` | `horizontal` | `horizontal` or `vertical`. A vertical rule fills the height of its container. |
| `label` | | Text in the middle of the rule, which splits around it. Horizontal only. |
| `thickness` | `2` | Thickness in cells. |
| `fade` | `0.6` | Share of each half that fades out, from 0 to 1. |
| `strength` | `0.6` | Densest dither level, at the middle of the rule. |
| `color` | `DitherTheme.ink` | Colour of the rule. |
| `fill` | `gradient` | `gradient` fades through dither levels, the others fade by opacity. |
| `matrix` | `bayer4` | The dither pattern. |
| `cell` | `2px` | Size of one dither cell. |

### DitherSwitch

A toggle switch whose track fills with dither behind the thumb as it turns on, stepping through the Bayer levels, and drains the same way when it turns off.

```slint
DitherSwitch {
    checked: true;
    color: DitherPalette.blue;
    bloom: DitherBloom.low;
    toggled(on) => {
        debug("switch toggled:", on);
    }
}
```

| Property | Default | |
| --- | --- | --- |
| `checked` | `false` | Whether it is on. |
| `fill` | `gradient` | Texture of the shade when on and track when off: `gradient`, `dotted`, `hatched` or `solid`. |
| `color` | `DitherTheme.ink` | Colour of the ink when on. |
| `size` | `md` | `sm`, `md` or `lg`: tracks of 34 by 20, 42 by 24 and 54 by 30 pixels, rounded to whole cells. |
| `bloom` | `off` | `low` or `high` adds a glow in the ink's colour when on. |
| `cell` | `2px` | Size of one dither cell. |
| `enabled` | `true` | Drops to 45% opacity and ignores clicks and keys when `false`. |
| `animated` | `DitherTheme.animated` | A ripple from where you press. |

### DitherSlider

A range slider whose fill is a Bayer ramp that gets denser towards the thumb and ends in a solid edge. Supports both a lit, notched thumb and a direct draggable progress-bar style with segment steps, tick marks, and range boundaries.

```slint
DitherSlider {
    label: "CPU Load";
    value: 64;
    unit: "%";
    variant: DitherSliderVariant.thumb;
    color: DitherPalette.blue;
    bloom: DitherBloom.low;
    changed(val) => {
        debug("slider:", val);
    }
}
```

| Property | Default | |
| --- | --- | --- |
| `value` | `0` | The current value (or lower value in range mode). |
| `value-high` | `100` | The upper value in two-thumb range mode. |
| `is-range` | `false` | When `true`, displays two lit notched thumbs and fills between them. |
| `minimum` | `0` | Lowest value. |
| `maximum` | `100` | Highest value. |
| `step` | `1` | Snapping interval from `minimum`. |
| `variant` | `thumb` | `thumb` for lit notched Bayer thumb; `bar` for seamless draggable meter bar. |
| `fill` | `gradient` | Texture of the track fill: `gradient`, `dotted`, `hatched` or `solid`. |
| `segments` | `0` | When > 1, divides the track into discrete blocks with 1-cell gaps. |
| `ticks` | `false` | When true, marks steps under the track lit in the active range. |
| `label` | `""` | Label text displayed above the track. |
| `hide-label`| `false`| Hides the label title while keeping the value readout. |
| `unit` | `""` | Unit suffix for the value readout (e.g. `"%"`). |
| `value-text`| `""` | Custom readout override string (e.g. `"5 / 10"`). |
| `show-value`| `true` | Shows live numeric readout next to the label. |
| `show-range`| `false`| Shows `minimum` and `maximum` values at the ends. |
| `color` | `DitherTheme.ink` | Colour of the fill and the thumb. |
| `size` | `md` | `sm`, `md` or `lg`: tracks of 6, 8 and 12px; thumbs 20, 24 and 30px tall. |
| `bloom` | `off` | `low` or `high` adds a glow in the slider's colour. |
| `matrix` | `bayer4` | The dither pattern of the fill. |
| `cell` | `2px` | Size of one dither cell. |
| `animated` | `DitherTheme.animated` | Enables smooth 180ms ease-out glide on clicks, steps, and programmatic changes, with 0ms drag latency. |
| `enabled` | `true` | Drops to 45% opacity and ignores clicks and keys when `false`. |

### DitherInput

A text field on a faint dithered tint, with a dithered floor and an edge that lights up on focus.

```slint
DitherInput {
    label: "Email";
    placeholder: "you@example.com";
    clearable: true;
    edited(text) => {
        debug(text);
    }
}
```

| Property | Default | |
| --- | --- | --- |
| `text` | | The current text. |
| `placeholder` | | Shown while the field is empty. |
| `label` | | A label above the field. |
| `description` | | Muted help text below the field. |
| `error` | | An error below the field. Turns it red. |
| `icon` | | An image before the text, such as a magnifier for search. |
| `password` | `false` | Hides the text and adds a show and hide button. |
| `clearable` | `false` | A clear button while there is text. Escape clears too. |
| `fill` | `gradient` | Texture of the floor: `gradient`, `dotted`, `hatched` or `solid`. |
| `color` | `DitherTheme.ink` | Colour of the tint, floor and focus edge. |
| `matrix` | `bayer4` | The dither pattern. |
| `size` | `md` | `sm`, `md` or `lg`: 32, 40 and 44 pixels high. |
| `bloom` | `off` | A soft glow in the colour while focused. |
| `cell` | `2px` | Size of one dither cell. |
| `enabled` | `true` | Drops to 45% opacity and ignores input when `false`. |

`edited(string)` is called whenever the text changes, `accepted(string)` when you press Enter and `cleared()` after the clear button or Escape.

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

### DitherDropdownMenu

A menu that opens under whatever you put inside it, which becomes the trigger. The active item is lit in one of six dithered styles, and the menu sits on a dithered card.

```slint
DitherDropdownMenu {
    items: [
        { text: "Duplicate", shortcut: "Ctrl+D", group: "File actions" },
        { text: "Rename", shortcut: "F2" },
        { text: "Move to trash", shortcut: "Del", destructive: true, divider: true },
    ];
    selected(index) => {
        debug(index);
    }

    DitherButton {
        text: "Actions";
        variant: DitherButtonVariant.outline;
    }
}
```

| Property | Default | |
| --- | --- | --- |
| `items` | | The rows. Each has `text`, `shortcut`, `destructive`, `disabled`, `group` (a heading above the item) and `divider` (a line above the item). |
| `variant` | `pill` | Highlight of the active item: `pill`, `dither`, `glow`, `hatched`, `dotted` or `outline`. |
| `menu` | `card` | The container: `card` with a dithered corner glow, `glow`, `pixel` or `solid`. |
| `color` | `DitherTheme.ink` | Colour of the highlights and the glow. |
| `matrix` | `bayer4` | The dither pattern. |
| `bloom` | `off` | `low` or `high` strengthens the textured highlights. |
| `cell` | `2px` | Size of one dither cell. |
| `menu-width` | `208px` | Width of the menu. |
| `enabled` | `true` | Ignores clicks when `false`. |

`selected(int)` is called with the index of the chosen item. `open()` and `close()` open and close the menu from code, and `is-open` tells you whether it is open.

### DitherAvatar

A generative avatar: a mirrored dither face drawn from a name. The same name always gives the same face and colour. `DitherAvatarGroup` stacks several.

```slint
DitherAvatar {
    name: "Ada Lovelace";
    status: DitherAvatarStatus.online;
}

DitherAvatarGroup {
    names: ["Ada", "Grace", "Alan", "Linus", "Margaret"];
    max: 4;
}
```

| Property | Default | |
| --- | --- | --- |
| `name` | | The string the face is drawn from, usually a name, an email or an id. |
| `size` | `40px` | Width and height. |
| `color` | from `name` | Any colour. Without one, a hue is picked from the name. |
| `fill` | `gradient` | How each block is drawn: `gradient`, `dotted`, `hatched` or `solid`. |
| `shape` | `circle` | `circle` or `square` with rounded corners. |
| `ring` | `false` | A ring in the avatar's colour, for the current user or a selection. |
| `status` | `none` | A presence dot in the corner: `online`, `away`, `busy` or `offline`. |
| `bloom` | `off` | `low` or `high` adds a glow in the avatar's colour. |

`DitherAvatarGroup` takes `names`, `max` (default `4`, the rest become a "+N" chip), `size` (default `32px`) and `outline`, the colour of the gap around each avatar, which should match the surface behind the group.

### DitherLoader

A loading indicator drawn in dither. It is a progress indicator for screen readers.

```slint
DitherLoader {
    variant: DitherLoaderVariant.ring;
    color: DitherPalette.blue;
}
```

| Property | Default | |
| --- | --- | --- |
| `variant` | `ring` | `ring`, `dots`, `pulse` or `bar`. |
| `size` | `md` | `sm`, `md` or `lg`: 24, 40 and 64 pixels high. Dots are twice as wide, the bar three times. |
| `color` | `DitherTheme.ink` | Any colour, or one from `DitherPalette`. |
| `matrix` | `bayer4` | The dither pattern. |
| `cell` | `2px` (`1px` at `sm`) | Size of one dither cell. |
| `speed` | `1` | Speed multiplier. |
| `label` | `Loading` | What a screen reader announces. |
| `animated` | `DitherTheme.animated` | When `false` it holds a still frame. |

### DitherProgressBar

A progress bar built from dither cells. With a `value` the fill grows to it, densest at its leading edge and capped with a solid cell. Without one a dithered comet sweeps the track. `parts` stacks several values with a legend, and `segments` cuts the bar into blocks.

```slint
DitherProgressBar {
    label: "Uploading";
    value: 64;
}

DitherProgressBar {
    label: "Storage";
    unit: " GB";
    parts: [
        { label: "Photos", value: 32 },
        { label: "Videos", value: 24 },
        { label: "Documents", value: 12 },
    ];
}
```

| Property | Default | |
| --- | --- | --- |
| `label` | | What is in progress. Shown above the bar and used as its accessible name. |
| `value` | `-1` | Progress from 0 to `maximum`. A negative value makes the bar indeterminate. |
| `maximum` | `100` | The value that means done, or the capacity when you use `parts`. |
| `value-text` | | Text for the readout, such as "3 of 5". Default is the percentage. |
| `hide-label` | `false` | Hides the label and readout, but keeps them for screen readers. |
| `size` | `md` | `sm`, `md` or `lg`: 2, 4 and 6 cells high. |
| `fill` | `gradient` | `gradient`, `dotted`, `hatched` or `solid`. |
| `color` | `DitherTheme.ink` | Colour of the fill and the track. |
| `bloom` | `off` | `low` or `high` adds a glow. |
| `segments` | `0` | Cuts the bar into this many blocks, a cell apart. Up to 32. |
| `gap` | `DitherTheme.background` | Colour of the cut between segments. Match the surface behind the bar. |
| `cell` | `2px` | Size of one dither cell. |
| `animate` | `DitherTheme.animated` | Grows to the value, eases between values and sweeps the comet. |
| `parts` | | Several values stacked left to right, each with `label`, `value` and an optional `color`. |
| `unit` | | Shown after each part's value in the legend. |

### DitherStatCard

A KPI card: a label, a headline value and a delta that is green when the metric moved the way you want and red when it did not. A `trend` draws the recent values as a dithered area along the bottom, ending in a solid marker. It is a `DitherCard`, so it also takes `color`, `matrix`, `cell` and `radius`.

```slint
DitherStatCard {
    label: "Monthly revenue";
    value: "48,210";
    has-delta: true;
    delta: 12.4;
    delta-label: "vs last month";
    trend: [12, 14, 13, 18, 17, 22, 21, 27, 31];
}
```

| Property | Default | |
| --- | --- | --- |
| `label` | | What the number is. |
| `value` | | The headline number, as text. `DitherEngine.format-number(float)` adds thousands separators. |
| `has-delta` | `false` | Shows the delta row. |
| `delta` | `0` | Change since the last period, in percent. |
| `delta-label` | | What the change compares with, such as "vs last month". |
| `good` | `up` | `DitherStatDirection.up` or `down`: which direction counts as good. |
| `trend` | | Recent values, oldest first. Needs at least two. |
| `trend-fill` | `gradient` | `gradient`, `dotted`, `hatched` or `solid`. |
| `bloom` | `off` | `low` or `high` adds a soft glow under the trend. |

### DitherSparkline

A tiny trend line for tables and cards: a line, a dithered fill that is densest just under it, and a dot on the latest value. The width follows the element. With `interactive` you can scrub across it with the pointer or the arrow keys to read single values.

```slint
DitherSparkline {
    width: 320px;
    label: "Revenue, last 12 days";
    data: [12, 14, 13, 18, 17, 22, 21, 27, 31, 29, 34, 38];
}
```

| Property | Default | |
| --- | --- | --- |
| `data` | | Values in order, oldest first. |
| `label` | | What the trend shows. Starts the accessible summary. |
| `height` | `32px` | Height of the sparkline. |
| `color` | `DitherTheme.ink` | Colour of the line, the fill and the dot. |
| `area` | `true` | Fills the area under the line. |
| `fill` | `gradient` | `gradient`, `dotted`, `hatched` or `solid`. |
| `level` | `0.5` | Gradient only: coverage of the fill just under the line. |
| `bloom` | `off` | `low` or `high` adds a soft glow. |
| `dot` | `true` | A dot on the latest value. |
| `interactive` | `false` | Scrub with the pointer or the arrow keys. |
| `domain` | the data's range | The value range the height covers, as `{ min, max }`. |
| `animate` | `DitherTheme.animated` | The line draws in and the fill fades in. |
| `matrix` | `bayer4` | The dither pattern. |
| `cell` | `2px` | Size of one dither cell. |

### DitherUptimeBar

A status-page row: one dithered bar per day, coloured and textured by status, with the overall uptime. Hover, tap or arrow through the bars for each day's details. It shows the last 30, 60 or 90 days, depending on its width.

```slint
DitherUptimeBar {
    label: "API";
    days: [
        { date: "2026-11-10", uptime: 1 },
        { date: "2026-11-11", uptime: 0.995, note: "Slow responses from the EU region." },
        { date: "2026-11-12", uptime: 1 },
    ];
}
```

| Property | Default | |
| --- | --- | --- |
| `label` | | The service. |
| `days` | | One entry per day, oldest first: `date`, `uptime` from 0 to 1 (negative for no data) and an optional `note`. |
| `fill` | `gradient` | Texture of the up days. Degraded days are hatched, outages solid, days without data dotted. |
| `bloom` | `off` | `low` or `high` adds a glow behind incidents. |
| `up-color`, `degraded-color`, `down-color`, `none-color` | green, orange, red, grey | Colour per status. |
| `matrix` | `bayer4` | The dither pattern. |
| `cell` | `2px` | Size of one dither cell. |

A day is up at 99.9% or more, degraded at 99% or more and an outage below that. Set `status` with `custom-status: true` to decide it yourself.

### DitherGauge

A radial gauge built from dithered blocks: a half arc, a 270° dial or a full ring. Blocks up to the value fill in over a faint track, and the block at the value is solid and steps out by a cell, like a needle.

```slint
DitherGauge {
    label: "CPU";
    value: 72;
    unit: "%";
}

DitherGauge {
    label: "Speed";
    value: 88;
    unit: " km/h";
    maximum: 160;
    variant: DitherGaugeVariant.dial;
}
```

| Property | Default | |
| --- | --- | --- |
| `value` | `0` | The reading, between `minimum` and `maximum`. |
| `label` | | What is measured. Shown under the value and used as the accessible name. |
| `minimum`, `maximum` | `0`, `100` | The scale. |
| `variant` | `half` | `half`, `dial` or `ring`. |
| `size` | `220px` | Width of the gauge. |
| `segments` | 28, 36 or 48 | Number of blocks. The default depends on the variant. |
| `unit` | | Shown after the value. |
| `decimals` | `1` | Most decimals in the value and the range. |
| `value-text` | | Text for the reading, instead of the value and unit. |
| `show-range` | `true`, `false` for `ring` | Shows `minimum` and `maximum` under the ends of the arc. |
| `color` | `DitherTheme.ink` | Colour of the value arc. |
| `track-color` | `color` | Colour of the empty track, drawn faint. |
| `fill` | `gradient` | `gradient`, `dotted`, `hatched` or `solid`. |
| `bloom` | `off` | `low` or `high` adds a glow behind the filled blocks. |
| `animate` | `DitherTheme.animated` | Fades the blocks in one by one. |
| `matrix` | `bayer4` | The dither pattern. |
| `cell` | `3px` | Size of one dither cell. |

### DitherNumber

An animated number for dashboards and landing stats. Each changed digit rolls on a strip to its new value, takes the accent colour while it moves and fades out at its top and bottom in a dither. It rolls in from `from` when it appears, and again whenever `value` changes.

```slint
DitherNumber {
    value: 48210;
    decimals: 0;
    size: DitherNumberSize.xl;
}
```

| Property | Default | |
| --- | --- | --- |
| `value` | `0` | The number to show. |
| `from` | `0` | Where the first roll starts. |
| `decimals` | `3` | Most decimals. Thousands get a separator. |
| `prefix`, `suffix` | | Text before and after the number. |
| `size` | `md` | `sm`, `md`, `lg`, `xl` or `display`. |
| `color` | `DitherTheme.foreground` | Colour of the number. |
| `accent` | `DitherTheme.ink` | Colour a digit takes while it moves. |
| `duration`, `stagger` | `900`, `40` | Milliseconds per digit, and between digits. |
| `animate` | `DitherTheme.animated` | When `false` the number shows its value straight away. |
| `matrix` | `bayer4` | The dither pattern of the faded edges. |
| `cell` | about 1/30 of the font size | Size of one dither cell. |
| `background` | `DitherTheme.background` | Colour behind the number, used for the faded edges. |

### DitherSkeleton

A placeholder for content that is still loading, as a block, lines of text or a circle. A band of denser dither sweeps across it.

```slint
DitherSkeleton {
    shape: DitherSkeletonShape.text;
    lines: 3;
}

DitherSkeleton {
    shape: DitherSkeletonShape.circle;
    size: 48px;
}
```

| Property | Default | |
| --- | --- | --- |
| `shape` | `block` | `block` is a rectangle you size yourself. `text` draws lines and `circle` an avatar. |
| `lines` | `3` | How many lines for `text`. The last one is shorter. |
| `size` | `40px` | Diameter of a `circle`. |
| `color` | grey | Any colour, or one from `DitherPalette`. |
| `fill` | `gradient` | Texture at rest: a dither ramp from the left, a dot lattice, diagonals or a flat tint. |
| `animation` | `sweep` | `sweep` runs a dense band across, `pulse` steps the dither up and down, `none` holds still. |
| `matrix` | `bayer4` | The dither pattern. |
| `cell` | `2px` | Size of one dither cell. |
| `duration` | `1.8` | Seconds for one sweep or pulse, including the pause after it. |
| `delay` | `0` | Seconds to wait before the first sweep. |
| `animated` | `DitherTheme.animated` | When `false` it holds a still frame. |

### DitherEmptyState

A dithered illustration with a title, a description and room for your actions, for lists and searches that have nothing to show.

```slint
DitherEmptyState {
    illustration: DitherIllustration.search;
    title: "Nothing found";
    description: "Try a different word or clear the filters.";

    DitherButton { text: "Clear filters"; }
}
```

| Property | Default | |
| --- | --- | --- |
| `title` | | The heading. Say plainly what is empty. |
| `description` | | One or two sentences: why it is empty and what to do next. |
| `illustration` | `box` | `box` for lists and orders, `search` for no results, `folder` for files and documents, `planet` for anything still to arrive. |
| `color` | `DitherTheme.ink` | Colour of the illustration. |
| `matrix` | `bayer4` | `bayer4` for crisp steps, `bayer8` for finer ones, `blue-noise` for an even grain. |
| `animate` | `DitherTheme.animated` | Bobs the object a cell up and down. |

Anything you put inside is shown under the text, usually one or two buttons.

## What is a dither

A dither draws a smooth tone using only a few colours. Dots of ink are placed at a density that matches the tone, so your eye blends them into a gradient. It started as a workaround for limited screens and printers and now reads as a deliberate texture, a bit like risograph print.

## Credits

- [Mvkweb](https://github.com/Mvkweb), [dither-sdk.vercel.app](https://dither-sdk.vercel.app). This library is a port of dither-sdk by mvk

## License

GPL 3.0
