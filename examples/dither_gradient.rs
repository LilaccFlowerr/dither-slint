slint::slint! {
    import { DitherGradient, DitherMatrix, DitherTheme } from "ui/lib.slint";
    export { DitherEngine, DitherMatrix } from "ui/lib.slint";

    export component Demo inherits Window {
        title: "DitherGradient";
        preferred-width: 640px;
        preferred-height: 420px;
        background: DitherTheme.background;

        VerticalLayout {
            padding: 24px;
            spacing: 16px;

            DitherGradient {
                border-radius: 8px;
            }

            HorizontalLayout {
                spacing: 16px;
                DitherGradient { angle: 90deg; matrix: DitherMatrix.bayer2; border-radius: 8px; }
                DitherGradient { angle: 135deg; matrix: DitherMatrix.bayer8; border-radius: 8px; }
                DitherGradient { angle: 270deg; matrix: DitherMatrix.blue-noise; border-radius: 8px; }
            }

            DitherGradient {
                to: 0.55;
                cell: 5px;
                ink: #5471f5;
                border-radius: 8px;
            }
        }
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let demo = Demo::new()?;
    dither_slint::install!(demo);
    demo.run()
}
