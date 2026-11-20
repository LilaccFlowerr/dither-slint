pub mod backgrounds;
pub mod color;
pub mod diffuse;
pub mod export;
pub mod field;
pub mod glow;
pub mod lively;
pub mod matrix;
pub mod noise;
pub mod render;
pub mod slider;
pub mod style;
pub mod switch;
pub mod tabs;

mod blue_noise;

pub use glam;
pub use matrix::Matrix;
#[doc(hidden)]
pub use slint;

#[macro_export]
macro_rules! install {
    ($component:expr) => {{
        use $crate::lively::{Hover, Idle, Layer, Press};
        use $crate::slint::{ComponentHandle as _, Model as _};
        use $crate::style::{FillEdge, FillVariant};
        use $crate::{Matrix, diffuse::Method, glow::Glow, render};

        fn cells(count: i32) -> u32 {
            count.max(0) as u32
        }

        let engine = $component.global::<DitherEngine>();

        engine.on_linear_gradient(|columns, rows, angle, from, to, matrix, palette, levels| {
            let matrix = Matrix::ALL[matrix as usize];
            let palette = palette.iter().collect();
            let options = render::GradientOptions { angle, from, to, matrix, palette, levels: cells(levels) };
            render::linear_gradient(cells(columns), cells(rows), &options)
        });
        engine.on_fill(|columns, rows, variant, edge, length, from, to, matrix, color, frame| {
            let options = render::FillOptions {
                variant: FillVariant::ALL[variant as usize],
                edge: FillEdge::ALL[edge as usize],
                length,
                from,
                to,
                matrix: Matrix::ALL[matrix as usize],
                color,
                frame,
            };
            render::fill(cells(columns), cells(rows), &options)
        });
        engine.on_rule(|columns, rows, vertical, fade_start, fade_end, fade, strength, variant, matrix, color| {
            let options = render::RuleOptions {
                vertical,
                fade_start,
                fade_end,
                fade,
                strength,
                variant: FillVariant::ALL[variant as usize],
                matrix: Matrix::ALL[matrix as usize],
                color,
            };
            render::rule(cells(columns), cells(rows), &options)
        });
        engine.on_slider_fill(|columns, rows, end, hot, matrix, color| {
            let options = render::SliderOptions {
                start: 0,
                end,
                both: false,
                hot,
                matrix: Matrix::ALL[matrix as usize],
                color,
            };
            render::slider_fill(cells(columns), cells(rows), &options)
        });
        engine.on_slider_range_fill(|columns, rows, start, end, both, hot, matrix, color| {
            let options = render::SliderOptions {
                start,
                end,
                both,
                hot,
                matrix: Matrix::ALL[matrix as usize],
                color,
            };
            render::slider_fill(cells(columns), cells(rows), &options)
        });
        engine.on_picture(|source, columns, rows, method, threshold, palette| {
            let method = Method::ALL[method as usize];
            let options = render::PictureOptions { method, threshold, palette: palette.iter().collect() };
            render::picture(&source, cells(columns), cells(rows), &options)
        });
        engine.on_card_glow(|columns, rows, glow, reach, strength, boost, rim, radius, fill, matrix, color| {
            let options = $crate::glow::GlowOptions {
                glow: Glow::ALL[glow as usize],
                reach,
                strength,
                boost,
                rim,
                radius,
                fill: FillVariant::ALL[fill as usize],
                matrix: Matrix::ALL[matrix as usize],
                color,
            };
            $crate::glow::card_glow(cells(columns), cells(rows), &options)
        });
        engine.on_card_sparkles(|columns, rows, cell, tick, glow, reach, color| {
            let layer = Layer::new(columns, rows, 1.0, color, Matrix::Bayer4);
            $crate::lively::glow_sparkles(&layer, cell, tick, Glow::ALL[glow as usize], reach)
        });
        engine.on_shader(|field, columns, rows, time, params, palette, matrix, levels| {
            let Some(field) = $crate::field::lookup(&field) else {
                return $crate::slint::Image::default();
            };
            let (params, palette): (Vec<f32>, Vec<_>) = (params.iter().collect(), palette.iter().collect());
            $crate::field::render(&$crate::field::Shader {
                field: field.as_ref(),
                columns: cells(columns),
                rows: cells(rows),
                time,
                params: &params,
                palette: &palette,
                matrix: Matrix::ALL[matrix as usize],
                levels: cells(levels),
            })
        });
        engine.on_mix(render::mix);
        engine.on_hover(|kind, columns, rows, t, x, y, level, strength, color, matrix| {
            let layer = Layer::new(columns, rows, strength, color, Matrix::ALL[matrix as usize]);
            $crate::lively::hover(Hover::ALL[kind as usize], &layer, t, (x, y), level)
        });
        engine.on_press(|kind, columns, rows, t, x, y, seed, strength, color, matrix| {
            let layer = Layer::new(columns, rows, strength, color, Matrix::ALL[matrix as usize]);
            $crate::lively::press(Press::ALL[kind as usize], &layer, t, (x, y), seed)
        });
        engine.on_idle(|kind, columns, rows, t, strength, color, cell, matrix| {
            let layer = Layer::new(columns, rows, strength, color, Matrix::ALL[matrix as usize]);
            $crate::lively::idle(Idle::ALL[kind as usize], &layer, t, cell)
        });
        engine.on_tabs_indicator(|columns, rows, cell, variant, kind, color, fx, fy, fw, fh, tx, ty, tw, th, progress| {
            let variant = match variant {
                DitherTabsVariant::Pill => $crate::tabs::TabsVariant::Pill,
                DitherTabsVariant::Underline => $crate::tabs::TabsVariant::Underline,
            };
            let from = $crate::tabs::BoxRect { x: fx, y: fy, w: fw, h: fh };
            let to = $crate::tabs::BoxRect { x: tx, y: ty, w: tw, h: th };
            $crate::tabs::tabs_indicator(
                cells(columns),
                cells(rows),
                cell,
                variant,
                Matrix::ALL[kind as usize],
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diffuse::Method;
    use crate::glow::Glow;
    use crate::lively::{Hover, Idle, Press};
    use crate::style::{FillEdge, FillVariant};

    const ENGINE: &str = include_str!("../ui/engine.slint");

    fn in_slint(name: &str) -> Vec<String> {
        let start = ENGINE.find(&format!("export enum {name} {{")).unwrap();
        let body = &ENGINE[start..];
        let body = &body[body.find('{').unwrap() + 1..body.find('}').unwrap()];
        body.split(',').map(|variant| variant.trim().replace('-', "").to_lowercase()).collect()
    }

    fn in_rust<T: std::fmt::Debug>(all: &[T]) -> Vec<String> {
        all.iter().map(|variant| format!("{variant:?}").to_lowercase()).collect()
    }

    #[test]
    fn rust_enums_follow_the_order_of_the_slint_ones() {
        assert_eq!(in_rust(&Matrix::ALL), in_slint("DitherMatrix"));
        assert_eq!(in_rust(&FillVariant::ALL), in_slint("DitherFill"));
        assert_eq!(in_rust(&FillEdge::ALL), in_slint("DitherEdge"));
        assert_eq!(in_rust(&Glow::ALL), in_slint("DitherGlow"));
        assert_eq!(in_rust(&Hover::ALL), in_slint("DitherHover"));
        assert_eq!(in_rust(&Press::ALL), in_slint("DitherPress"));
        assert_eq!(in_rust(&Idle::ALL), in_slint("DitherIdle"));
        assert_eq!(in_rust(&Method::ALL), in_slint("DitherMethod"));
    }
}
