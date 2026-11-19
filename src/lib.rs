mod blue_noise;
mod diffuse;
mod export;
mod matrix;
mod render;
mod style;

pub use diffuse::{Method, Options, PaletteColor, dither_image};
pub use export::{Source, SvgOptions, svg_data_url, to_svg};
pub use matrix::{Matrix, ThresholdMap};
pub use render::render;
pub use style::{
    Bloom, BloomLevel, FillEdge, FillTile, FillVariant, RampOptions, RampTile, TextureVariant,
    SERIES_COLORS, bloom_level, dither_ramp, fill_tile, series_color,
};

use std::collections::HashMap;
use std::path::PathBuf;

pub fn library_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui")
}

pub fn library_paths() -> HashMap<String, PathBuf> {
    HashMap::from([("dither".to_string(), library_path())])
}
