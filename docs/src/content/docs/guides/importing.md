---
title: Importing
description: Which names a .slint file needs to import, and how to keep the list short.
---

Slint has no import of everything at once. Every name you write in a `.slint` file has to be imported first, from the library that holds it. That is true for the components that ship with Slint too: you import `Button` from `std-widgets.slint` before you use it.

For dither-slint everything comes from one file:

```slint
import { DitherButton, DitherFill } from "@dither/lib.slint";
```

## What to import

| You write | Import |
| --- | --- |
| `DitherButton { }` | The component. |
| `DitherFill.dotted` | The enum, as soon as you write one of its values. |
| `DitherTheme.ink`, `DitherPalette.green` | The global. |
| `{ label: "Revenue", values: [1, 3, 2] }` | Nothing. A struct written as a literal takes its type from the property. |

A missing import gives an error like this:

```text
error: Cannot access id 'DitherScrambleMode'
```

That means the name is used but not imported.

## Copy the line from the component page

Every component page starts its Properties section with an **Import** line that lists the component and every enum its properties use. Copy it, then delete the names you do not need:

```slint
import { DitherScrambleText, DitherScrambleMode, DitherScrambleTrigger } from "@dither/lib.slint";
```

The globals `DitherTheme`, `DitherPalette` and `DitherMotion` are not in that line, so add them when you set colours.

## Keep the list in one file

In a larger app, give the library one file of your own that passes on what you use. Create `ui/dither.slint`:

```slint
export {
    DitherEngine,
    DitherButton,
    DitherCard,
    DitherFill,
    DitherPalette,
    DitherTheme,
} from "@dither/lib.slint";
```

Every other file then imports from it, with a path relative to the file:

```slint
import { DitherButton, DitherCard } from "dither.slint";
```

The `@dither/lib.slint` path is now in one place, and a new component is one extra name in `dither.slint`. In a file in another folder the path is `"../dither.slint"`.

The window you hand to `install!` still has to export the engine. With the bundle file, that is:

```slint
export { DitherEngine } from "dither.slint";
```
