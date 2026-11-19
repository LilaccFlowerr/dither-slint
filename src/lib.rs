mod blue_noise;
mod diffuse;
mod export;
mod matrix;
mod render;

pub use diffuse::{Method, Options, PaletteColor, dither_image};
pub use export::{Source, SvgOptions, svg_data_url, to_svg};
pub use matrix::{Matrix, ThresholdMap};
pub use render::render;

use std::collections::HashMap;
use std::path::PathBuf;

pub fn library_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui")
}

pub fn library_paths() -> HashMap<String, PathBuf> {
    HashMap::from([("dither".to_string(), library_path())])
}
