pub mod color;
pub mod diffuse;
pub mod export;
pub mod lively;
pub mod matrix;
pub mod render;
pub mod style;
pub mod tabs;

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
        engine.on_picture(|source, columns, rows, method, threshold, ink, paper| {
            use $crate::diffuse::Method;
            let method = match method {
                DitherMethod::Bayer4 => Method::Bayer4,
                DitherMethod::Bayer8 => Method::Bayer8,
                DitherMethod::BlueNoise => Method::BlueNoise,
                DitherMethod::FloydSteinberg => Method::FloydSteinberg,
                DitherMethod::Atkinson => Method::Atkinson,
                DitherMethod::SierraLite => Method::SierraLite,
            };
            let options = $crate::render::PictureOptions { method, threshold, ink, paper };
            $crate::render::picture(&source, columns.max(0) as u32, rows.max(0) as u32, &options)
        });
        engine.on_mix($crate::render::mix);
        engine.on_hover(|kind, columns, rows, t, x, y, level, strength, color| {
            use $crate::lively::Hover;
            let kind = match kind {
                DitherHover::Sweep => Hover::Sweep,
                DitherHover::Spotlight => Hover::Spotlight,
                DitherHover::Scan => Hover::Scan,
                DitherHover::Trace => Hover::Trace,
                DitherHover::Rise => Hover::Rise,
                DitherHover::Noise => Hover::Noise,
                DitherHover::None => Hover::None,
            };
            let layer = $crate::lively::Layer::new(columns, rows, strength, color);
            $crate::lively::hover(kind, &layer, t, (x, y), level)
        });
        engine.on_press(|kind, columns, rows, t, x, y, seed, strength, color| {
            use $crate::lively::Press;
            let kind = match kind {
                DitherPress::Ripple => Press::Ripple,
                DitherPress::Burst => Press::Burst,
                DitherPress::Flash => Press::Flash,
                DitherPress::Pulse => Press::Pulse,
                DitherPress::Glitch => Press::Glitch,
                DitherPress::None => Press::None,
            };
            let layer = $crate::lively::Layer::new(columns, rows, strength, color);
            $crate::lively::press(kind, &layer, t, (x, y), seed)
        });
        engine.on_idle(|kind, columns, rows, t, strength, color, cell| {
            use $crate::lively::Idle;
            let kind = match kind {
                DitherIdle::None => Idle::None,
                DitherIdle::Shimmer => Idle::Shimmer,
                DitherIdle::Breathe => Idle::Breathe,
                DitherIdle::Twinkle => Idle::Twinkle,
            };
            let layer = $crate::lively::Layer::new(columns, rows, strength, color);
            $crate::lively::idle(kind, &layer, t, cell)
        });
        engine.on_tabs_indicator(|columns, rows, cell, variant, kind, color, fx, fy, fw, fh, tx, ty, tw, th, progress| {
            let variant = match variant {
                DitherTabsVariant::Pill => $crate::tabs::TabsVariant::Pill,
                DitherTabsVariant::Underline => $crate::tabs::TabsVariant::Underline,
            };
            let from = $crate::tabs::BoxRect { x: fx, y: fy, w: fw, h: fh };
            let to = $crate::tabs::BoxRect { x: tx, y: ty, w: tw, h: th };
            $crate::tabs::tabs_indicator(
                columns.max(0) as u32,
                rows.max(0) as u32,
                cell,
                variant,
                matrix(kind),
                color,
                from,
                to,
                progress,
            )
        });
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
