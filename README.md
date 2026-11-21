# dither-slint

Thanks to MVKweb for the idea and allowing me to make such a thing

Dithered UI components for [Slint](https://slint.dev), based on the dither work of [Mvkweb](https://github.com/Mvkweb).

> Work in progress. More components are on the way.

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

## Components

- **Basics:** `DitherGradient`, `DitherButton`, `DitherBadge`, `DitherCard`, `DitherDivider`
- **Controls:** `DitherSwitch`, `DitherSlider`, `DitherInput`, `DitherTabs`, `DitherDropdownMenu`
- **Identity and feedback:** `DitherAvatar`, `DitherLoader`, `DitherSkeleton`, `DitherEmptyState`

There is also `DitherImage` for dithering a photo, four animated backgrounds (`DitherNoiseField`, `DitherPlasma`, `DitherRipples` and `DitherWaves`) with `DitherShader` to write your own, and `DitherFx` for hover, press and idle effects. The docs in `docs/` show every one of them live. Build the engine for the browser with `./engine-web/build.sh`, then run `npm install` and `npm run dev` in `docs/`.

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
