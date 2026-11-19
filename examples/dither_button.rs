slint::slint! {
    #[edition = "1.19"]
    import { DitherButton, DitherButtonVariant, DitherButtonSize, DitherFill, DitherBloom, DitherPalette, DitherTheme } from "ui/lib.slint";
    export { DitherEngine } from "ui/lib.slint";

    export component Demo inherits Window {
        title: "DitherButton";
        background: DitherTheme.background;
        in-out property <bool> busy;

        VerticalLayout {
            padding: 32px;
            spacing: 20px;
            alignment: center;

            HorizontalLayout {
                spacing: 12px;
                alignment: center;
                DitherButton { text: "Deploy"; bloom: DitherBloom.low; }
                DitherButton { text: "View logs"; variant: DitherButtonVariant.outline; fill: DitherFill.dotted; }
                DitherButton { text: "Save"; loading: root.busy; clicked => { root.busy = true; } }
            }

            for fill in [DitherFill.gradient, DitherFill.dotted, DitherFill.hatched, DitherFill.solid]: HorizontalLayout {
                spacing: 12px;
                alignment: center;
                DitherButton { text: "Ink"; fill: fill; }
                DitherButton { text: "Green"; fill: fill; color: DitherPalette.green; }
                DitherButton { text: "Orange"; fill: fill; color: DitherPalette.orange; }
                DitherButton { text: "Pink"; fill: fill; color: DitherPalette.pink; }
                DitherButton { text: "Ink"; fill: fill; variant: DitherButtonVariant.outline; }
                DitherButton { text: "Green"; fill: fill; variant: DitherButtonVariant.outline; color: DitherPalette.green; }
            }

            HorizontalLayout {
                spacing: 12px;
                alignment: center;
                DitherButton { text: "Small"; size: DitherButtonSize.sm; }
                DitherButton { text: "Medium"; }
                DitherButton { text: "Large"; size: DitherButtonSize.lg; bloom: DitherBloom.high; }
                DitherButton { text: "Disabled"; enabled: false; }
            }
        }
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let demo = Demo::new()?;
    dither_slint::install!(demo);
    demo.run()
}
