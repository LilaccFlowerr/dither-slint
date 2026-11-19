pub mod color;
pub mod diffuse;
pub mod export;
pub mod matrix;
pub mod motion;
pub mod pattern;
pub mod render;
pub mod style;

mod blue_noise;

pub use matrix::Matrix;
#[doc(hidden)]
pub use slint;

#[macro_export]
macro_rules! install {
    ($component:expr) => {{
        use $crate::slint::ComponentHandle as _;
        use $crate::style::{FillEdge, FillVariant};

        fn matrix(matrix: DitherMatrix) -> $crate::Matrix {
            match matrix {
                DitherMatrix::Bayer2 => $crate::Matrix::Bayer2,
                DitherMatrix::Bayer4 => $crate::Matrix::Bayer4,
                DitherMatrix::Bayer8 => $crate::Matrix::Bayer8,
                DitherMatrix::BlueNoise => $crate::Matrix::BlueNoise,
            }
        }

        let engine = $component.global::<DitherEngine>();
        engine.on_linear_gradient(|columns, rows, angle, from, to, kind, ink, paper| {
            let options = $crate::render::GradientOptions { angle, from, to, matrix: matrix(kind), ink, paper };
            $crate::render::linear_gradient(columns.max(0) as u32, rows.max(0) as u32, &options)
        });
        engine.on_fill(|columns, rows, variant, edge, length, from, to, kind, color, frame| {
            let variant = match variant {
                DitherFill::Gradient => FillVariant::Gradient,
                DitherFill::Dotted => FillVariant::Dotted,
                DitherFill::Hatched => FillVariant::Hatched,
                DitherFill::Solid => FillVariant::Solid,
            };
            let edge = match edge {
                DitherEdge::Top => FillEdge::Top,
                DitherEdge::Bottom => FillEdge::Bottom,
                DitherEdge::Left => FillEdge::Left,
                DitherEdge::Right => FillEdge::Right,
            };
            let options = $crate::render::FillOptions { variant, edge, length, from, to, matrix: matrix(kind), color, frame };
            $crate::render::fill(columns.max(0) as u32, rows.max(0) as u32, &options)
        });
        engine.on_mix($crate::render::mix);
    }};
}

use std::collections::HashMap;
use std::path::PathBuf;

pub fn library_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui")
}

pub fn library_paths() -> HashMap<String, PathBuf> {
    HashMap::from([("dither".to_string(), library_path())])
}
