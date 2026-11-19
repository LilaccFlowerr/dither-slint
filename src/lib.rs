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
        $component.global::<DitherEngine>().on_linear_gradient(
            |columns, rows, angle, from, to, matrix, ink, paper| {
                let matrix = match matrix {
                    DitherMatrix::Bayer2 => $crate::Matrix::Bayer2,
                    DitherMatrix::Bayer4 => $crate::Matrix::Bayer4,
                    DitherMatrix::Bayer8 => $crate::Matrix::Bayer8,
                    DitherMatrix::BlueNoise => $crate::Matrix::BlueNoise,
                };
                let options = $crate::render::GradientOptions { angle, from, to, matrix, ink, paper };
                $crate::render::linear_gradient(columns.max(0) as u32, rows.max(0) as u32, &options)
            },
        );
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
