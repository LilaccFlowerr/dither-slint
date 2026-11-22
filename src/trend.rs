use slint::{Color, Image, Rgba8Pixel};

use crate::Matrix;
use crate::bloom::Bloom;
use crate::render::{paint, pixel};
use crate::stat::format_compact;
use crate::style::{FillVariant, texture_inked};

const BAND_LEVELS: [f32; 4] = [10.0 / 16.0, 6.0 / 16.0, 3.0 / 16.0, 1.0 / 16.0];
const BAND_DEPTH: f32 = 7.0;
const NEAR_SHARE: f32 = 0.3;
const SOLID_OPACITY: f32 = 0.35;
const STROKE: f32 = 4.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Margin {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

/// The box a series is drawn in, in pixels, and the value range its height covers.
/// Without a domain the range of the series itself is used.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub width: f32,
    pub height: f32,
    pub margin: Margin,
    pub domain: Option<(f32, f32)>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shading {
    Bands,
    Near,
}

impl Shading {
    pub const ALL: [Self; 2] = [Self::Bands, Self::Near];
}

#[derive(Clone, Copy, Debug)]
pub struct AreaOptions {
    pub fill: FillVariant,
    pub shading: Shading,
    pub level: f32,
    pub matrix: Matrix,
    pub color: Color,
}

#[derive(Clone, Copy, Debug)]
pub struct BloomOptions {
    pub area: bool,
    pub bloom: Bloom,
    pub color: Color,
}

pub fn points(values: &[f32], frame: &Frame) -> Vec<Point> {
    let Frame { width, height, margin, domain } = *frame;
    let (floor, ceiling) = domain.unwrap_or_else(|| {
        let low = values.iter().copied().fold(f32::INFINITY, f32::min);
        let high = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        (low, high)
    });
    let span = if ceiling > floor { ceiling - floor } else { 1.0 };
    let last = values.len().saturating_sub(1).max(1) as f32;
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let x = if values.len() == 1 { width / 2.0 } else { margin.left + index as f32 / last * (width - margin.left - margin.right) };
            let y = margin.top + (1.0 - (value - floor) / span) * (height - margin.top - margin.bottom);
            Point { x, y }
        })
        .collect()
}

fn line_at(points: &[Point], x: f32) -> Option<f32> {
    let (first, last) = (points.first()?, points.last()?);
    if points.len() < 2 || x < first.x || x > last.x {
        return None;
    }
    let index = points.partition_point(|point| point.x <= x).clamp(1, points.len() - 1);
    let (from, to) = (points[index - 1], points[index]);
    let run = to.x - from.x;
    Some(if run > 0.0 { from.y + (to.y - from.y) * (x - from.x) / run } else { to.y })
}

pub fn path(points: &[Point]) -> String {
    let mut commands = String::new();
    for (index, point) in points.iter().enumerate() {
        let command = if index == 0 { 'M' } else { 'L' };
        commands.push_str(&format!("{command} {:.1} {:.1} ", point.x, point.y));
    }
    commands.trim_end().to_string()
}

pub fn summary(label: &str, values: &[f32]) -> String {
    let (Some(first), Some(last)) = (values.first(), values.last()) else {
        return format!("{label}: no data");
    };
    let low = values.iter().copied().fold(f32::INFINITY, f32::min);
    let high = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    format!("{label}: {} to {}, low {}, high {}", format_compact(*first), format_compact(*last), format_compact(low), format_compact(high))
}

fn grid(frame: &Frame, cell: f32) -> (u32, u32) {
    ((frame.width / cell).ceil().max(1.0) as u32, (frame.height / cell).ceil().max(1.0) as u32)
}

/// The area under the line. The gradient thins out in bands below the line
/// (`Bands`) or is densest just under it (`Near`).
pub fn area(values: &[f32], frame: &Frame, cell: f32, options: &AreaOptions) -> Image {
    let AreaOptions { fill, shading, level, matrix, color } = *options;
    let line = points(values, frame);
    let ink = pixel(color);
    let (columns, rows) = grid(frame, cell);
    let near = (cell * 2.0).max(frame.height * NEAR_SHARE);
    let (far_level, near_level) = ((level * 8.0).round() / 16.0, (level * 16.0).round() / 16.0);
    paint(columns, rows, |x, y| {
        let clear = Rgba8Pixel::new(0, 0, 0, 0);
        let Some(top) = line_at(&line, (x as f32 + 0.5) * cell) else {
            return clear;
        };
        let below = (y as f32 + 0.5) * cell - top;
        if below < 0.0 {
            return clear;
        }
        let inked = match fill {
            FillVariant::Gradient => {
                let level = match shading {
                    Shading::Bands => BAND_LEVELS[((below / BAND_DEPTH) as usize).min(BAND_LEVELS.len() - 1)],
                    Shading::Near if below < near => near_level,
                    Shading::Near => far_level,
                };
                matrix.is_inked(x as i32, y as i32, level)
            }
            textured => texture_inked(textured, x as usize, y as usize),
        };
        match (inked, fill) {
            (false, _) => clear,
            (true, FillVariant::Solid) => Rgba8Pixel::new(ink.r, ink.g, ink.b, (ink.a as f32 * SOLID_OPACITY).round() as u8),
            (true, _) => ink,
        }
    })
}

fn distance_to_segment(point: Point, from: Point, to: Point) -> f32 {
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    let length = dx * dx + dy * dy;
    let t = if length > 0.0 { (((point.x - from.x) * dx + (point.y - from.y) * dy) / length).clamp(0.0, 1.0) } else { 0.0 };
    ((point.x - (from.x + t * dx)).powi(2) + (point.y - (from.y + t * dy)).powi(2)).sqrt()
}

/// A blurred, translucent copy of the area and a 4px line, or of the line
/// alone, padded on every side so it can spill past the box.
pub fn bloom(values: &[f32], frame: &Frame, cell: f32, options: &BloomOptions) -> Image {
    let BloomOptions { area, bloom, color } = *options;
    let line = points(values, frame);
    let pad = crate::bloom::padding(bloom, cell) as usize;
    let (columns, rows) = grid(frame, cell);
    let (width, height) = (columns as usize + 2 * pad, rows as usize + 2 * pad);
    let mut mask = vec![0.0_f32; width * height];
    for y in 0..height {
        for x in 0..width {
            let at = Point { x: (x as f32 + 0.5 - pad as f32) * cell, y: (y as f32 + 0.5 - pad as f32) * cell };
            let under = area && at.y <= frame.height && line_at(&line, at.x).is_some_and(|top| at.y >= top);
            let on_line = line.windows(2).any(|pair| distance_to_segment(at, pair[0], pair[1]) <= STROKE / 2.0);
            mask[y * width + x] = if under || on_line { 1.0 } else { 0.0 };
        }
    }
    crate::bloom::glow(mask, width, height, cell, bloom, color)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame() -> Frame {
        Frame { width: 100.0, height: 56.0, margin: Margin { left: 0.0, right: 0.0, top: 6.0, bottom: 16.8 }, domain: None }
    }

    fn options(fill: FillVariant, shading: Shading) -> AreaOptions {
        AreaOptions { fill, shading, level: 0.5, matrix: Matrix::Bayer4, color: Color::from_rgb_u8(200, 100, 50) }
    }

    fn alpha(image: &Image) -> Vec<u8> {
        image.to_rgba8().unwrap().as_slice().iter().map(|pixel| pixel.a).collect()
    }

    #[test]
    fn no_values_give_no_points() {
        assert!(points(&[], &frame()).is_empty());
    }

    #[test]
    fn a_single_value_sits_in_the_middle() {
        let line = points(&[3.0], &frame());
        assert_eq!(line.len(), 1);
        assert_eq!(line[0].x, 50.0);
    }

    #[test]
    fn points_run_from_left_to_right_with_high_values_on_top() {
        let line = points(&[1.0, 5.0, 3.0], &frame());
        assert_eq!((line[0].x, line[2].x), (0.0, 100.0));
        assert!(line[1].y < line[2].y && line[2].y < line[0].y);
    }

    #[test]
    fn a_flat_series_stays_inside_the_box() {
        let line = points(&[4.0, 4.0, 4.0], &frame());
        assert!(line.iter().all(|point| point.y.is_finite() && (0.0..=56.0).contains(&point.y)));
    }

    #[test]
    fn the_margin_keeps_the_line_off_the_edges() {
        let margin = Margin { left: 4.0, right: 4.0, top: 4.0, bottom: 4.0 };
        let line = points(&[0.0, 10.0], &Frame { margin, ..frame() });
        assert_eq!((line[0].x, line[1].x), (4.0, 96.0));
        assert!((line[0].y - 52.0).abs() < 1e-4 && (line[1].y - 4.0).abs() < 1e-4);
    }

    #[test]
    fn a_domain_decides_where_values_land() {
        let narrow = points(&[5.0, 5.0], &Frame { domain: Some((0.0, 10.0)), ..frame() });
        let wide = points(&[5.0, 5.0], &Frame { domain: Some((0.0, 100.0)), ..frame() });
        assert!(narrow[0].y < wide[0].y, "the same value is lower in a taller range");
    }

    #[test]
    fn the_line_is_interpolated_between_points() {
        let line = points(&[0.0, 10.0], &frame());
        assert!((line_at(&line, 50.0).unwrap() - (line[0].y + line[1].y) / 2.0).abs() < 1e-4);
        assert_eq!(line_at(&line, 101.0), None);
        assert_eq!(line_at(&line, -1.0), None);
    }

    #[test]
    fn the_path_starts_with_a_move() {
        let commands = path(&points(&[0.0, 10.0], &frame()));
        assert!(commands.starts_with("M 0.0 "));
        assert!(commands.contains("L 100.0 6.0"));
    }

    #[test]
    fn the_summary_names_both_ends_and_the_extremes() {
        assert_eq!(summary("Revenue", &[1200.0, 800.0, 3400.0]), "Revenue: 1.2K to 3.4K, low 800, high 3.4K");
        assert_eq!(summary("Revenue", &[]), "Revenue: no data");
    }

    #[test]
    fn nothing_is_drawn_above_the_line() {
        let image = area(&[0.0, 0.0, 10.0, 10.0], &frame(), 2.0, &options(FillVariant::Solid, Shading::Bands));
        let alpha = alpha(&image);
        assert_eq!(alpha[0], 0, "the top left corner lies above the line");
        assert!(*alpha.last().unwrap() > 0, "the bottom right corner lies inside the area");
    }

    #[test]
    fn the_bands_thin_out_below_the_line() {
        let flat = Frame { margin: Margin::default(), domain: Some((0.0, 10.0)), ..frame() };
        let image = area(&[5.0, 5.0], &flat, 1.0, &options(FillVariant::Gradient, Shading::Bands));
        let alpha = alpha(&image);
        let inked = |from: usize, to: usize| (from..to).flat_map(|row| (0..100).map(move |x| row * 100 + x)).filter(|&index| alpha[index] > 0).count();
        assert!(inked(28, 35) > inked(49, 56));
    }

    #[test]
    fn the_near_shading_is_densest_just_under_the_line() {
        let flat = Frame { margin: Margin::default(), domain: Some((0.0, 10.0)), ..frame() };
        let image = area(&[5.0, 5.0], &flat, 1.0, &options(FillVariant::Gradient, Shading::Near));
        let alpha = alpha(&image);
        let inked = |from: usize, to: usize| (from..to).flat_map(|row| (0..100).map(move |x| row * 100 + x)).filter(|&index| alpha[index] > 0).count();
        assert!(inked(28, 38) > inked(44, 54));
    }

    #[test]
    fn solid_is_translucent() {
        let image = area(&[1.0, 2.0], &frame(), 2.0, &options(FillVariant::Solid, Shading::Bands));
        assert!(alpha(&image).iter().all(|a| *a < 128));
    }

    #[test]
    fn a_single_value_draws_no_area() {
        assert!(alpha(&area(&[1.0], &frame(), 2.0, &options(FillVariant::Gradient, Shading::Near))).iter().all(|a| *a == 0));
    }

    #[test]
    fn the_bloom_spills_past_the_box() {
        let options = BloomOptions { area: true, bloom: Bloom::Low, color: Color::from_rgb_u8(200, 100, 50) };
        let image = bloom(&[0.0, 10.0], &frame(), 2.0, &options);
        let size = image.size();
        let pad = crate::bloom::padding(Bloom::Low, 2.0);
        assert_eq!((size.width, size.height), (50 + 2 * pad, 28 + 2 * pad));
        let alpha = alpha(&image);
        assert!(alpha[(size.width * (size.height / 2) + pad - 2) as usize] > 0, "the glow reaches left of the area");
        assert!(alpha.iter().all(|a| *a <= 77), "it never gets stronger than the low strength");
    }

    #[test]
    fn the_bloom_fades_out_before_the_edge_of_its_image() {
        let options = BloomOptions { area: true, bloom: Bloom::High, color: Color::from_rgb_u8(255, 255, 255) };
        let image = bloom(&[0.0, 10.0, 5.0], &frame(), 2.0, &options);
        let size = image.size();
        let alpha = alpha(&image);
        let bottom = &alpha[((size.height - 1) * size.width) as usize..];
        assert!(bottom.iter().all(|a| *a <= 2), "the glow is gone by the bottom row");
    }

    #[test]
    fn the_line_bloom_hugs_the_line() {
        let options = BloomOptions { area: false, bloom: Bloom::Low, color: Color::from_rgb_u8(255, 255, 255) };
        let image = bloom(&[0.0, 0.0], &Frame { margin: Margin::default(), domain: Some((-1.0, 1.0)), ..frame() }, 2.0, &options);
        let size = image.size();
        let alpha = alpha(&image);
        let column = size.width as usize / 2;
        let at = |row: usize| alpha[row * size.width as usize + column];
        let middle = size.height as usize / 2;
        assert!(at(middle) > at(2), "the glow is strongest on the line and fades away from it");
    }
}
