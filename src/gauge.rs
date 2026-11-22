use std::f32::consts::{PI, SQRT_2};

use slint::{Color, Image, Rgba8Pixel};

use crate::Matrix;
use crate::bloom::{self, Bloom};
use crate::render::{paint, pixel};
use crate::style::{FillVariant, texture_inked};

const HEAD_FALLOFF: f32 = 1.0 / 16.0;
const TAIL_LEVEL: f32 = 9.0 / 16.0;
const TRACK_OPACITY: f32 = 0.14;
const GAP: f32 = 2.0;
const STAGGER: f32 = 14.0;
const FADE: f32 = 200.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Variant {
    Half,
    Dial,
    Ring,
}

impl Variant {
    pub const ALL: [Self; 3] = [Self::Half, Self::Dial, Self::Ring];

    /// Start angle (clockwise from 12 o'clock), span in degrees and the default number of blocks.
    fn sweep(self) -> (f32, f32, u32) {
        match self {
            Variant::Half => (-90.0, 180.0, 28),
            Variant::Dial => (-135.0, 270.0, 36),
            Variant::Ring => (0.0, 360.0, 48),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Gauge {
    pub variant: Variant,
    pub size: f32,
    /// Zero picks the default for the variant.
    pub segments: u32,
    pub ratio: f32,
    pub cell: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    pub height: f32,
    pub thickness: f32,
    pub range_width: f32,
    pub count: u32,
    pub filled: f32,
    pub active: u32,
    outer: f32,
    inner: f32,
    step: f32,
    gap: f32,
    start: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ArcStyle {
    pub fill: FillVariant,
    pub matrix: Matrix,
    pub color: Color,
    pub track: Color,
    /// Milliseconds since the gauge appeared; the blocks fade in one by one.
    pub elapsed: f32,
}

pub fn layout(gauge: &Gauge) -> Layout {
    let Gauge { variant, size, segments, ratio, cell } = *gauge;
    let (start, span, default) = variant.sweep();
    let count = if segments == 0 { default } else { segments.max(2) };
    let thickness = (size * 0.12 / cell).round().max(2.0) * cell;
    let outer = size / 2.0 - 1.0 - cell;
    let inner = outer - thickness;
    let middle = outer - thickness / 2.0;
    let height = match variant {
        Variant::Half => size / 2.0,
        Variant::Dial => size / 2.0 + (outer + cell) / SQRT_2,
        Variant::Ring => size,
    };
    let filled = ratio.clamp(0.0, 1.0) * count as f32;
    Layout {
        height,
        thickness,
        range_width: 2.0 * middle * (span * PI / 360.0).sin() + thickness,
        count,
        filled,
        active: (filled - 1e-6).ceil().max(0.0) as u32,
        outer,
        inner,
        step: span / count as f32,
        gap: GAP / middle * 180.0 / PI,
        start,
    }
}

/// Where a point lies on the arc.
enum Spot {
    Off,
    Track,
    Lit(u32),
}

fn spot(layout: &Layout, size: f32, cell: f32, x: f32, y: f32) -> Spot {
    let (dx, dy) = (x - size / 2.0, y - size / 2.0);
    let radius = (dx * dx + dy * dy).sqrt();
    let along = (dx.atan2(-dy).to_degrees() - layout.start).rem_euclid(360.0);
    let index = (along / layout.step) as u32;
    if index >= layout.count {
        return Spot::Off;
    }
    let from = index as f32 * layout.step + layout.gap / 2.0;
    let to = (index + 1) as f32 * layout.step - layout.gap / 2.0;
    if along < from || along > to || radius < layout.inner {
        return Spot::Off;
    }
    let head = index + 1 == layout.active;
    let lit = index < layout.active && (!head || along <= from + (to - from) * (layout.filled - index as f32).clamp(0.0, 1.0));
    let reach = if head && lit { layout.outer + cell } else { layout.outer };
    match radius <= reach {
        false => Spot::Off,
        true if lit => Spot::Lit(index),
        true => Spot::Track,
    }
}

fn over(top: Rgba8Pixel, bottom: Rgba8Pixel) -> Rgba8Pixel {
    let (a, b) = (top.a as f32 / 255.0, bottom.a as f32 / 255.0);
    let alpha = a + b * (1.0 - a);
    if alpha <= 0.0 {
        return Rgba8Pixel::new(0, 0, 0, 0);
    }
    let channel = |t: u8, u: u8| ((t as f32 * a + u as f32 * b * (1.0 - a)) / alpha).round() as u8;
    Rgba8Pixel::new(channel(top.r, bottom.r), channel(top.g, bottom.g), channel(top.b, bottom.b), (alpha * 255.0).round() as u8)
}

fn grid(gauge: &Gauge, layout: &Layout) -> (u32, u32) {
    ((gauge.size / gauge.cell).ceil().max(1.0) as u32, (layout.height / gauge.cell).ceil().max(1.0) as u32)
}

/// The faint track and the blocks up to the value, one pixel per logical pixel so the
/// gaps between blocks stay sharp; the dither runs in cells. The block at the value is
/// solid and steps out by a cell; with the gradient each block behind it is a step lighter.
pub fn arc(gauge: &Gauge, style: &ArcStyle) -> Image {
    let ArcStyle { fill, matrix, color, track, elapsed } = *style;
    let layout = layout(gauge);
    let (columns, rows) = (gauge.size.ceil().max(1.0) as u32, layout.height.ceil().max(1.0) as u32);
    let mut faint = pixel(track);
    faint.a = (faint.a as f32 * TRACK_OPACITY).round() as u8;
    let ink = pixel(color);
    paint(columns, rows, |x, y| {
        let index = match spot(&layout, gauge.size, gauge.cell, x as f32 + 0.5, y as f32 + 0.5) {
            Spot::Off => return Rgba8Pixel::new(0, 0, 0, 0),
            Spot::Track => return faint,
            Spot::Lit(index) => index,
        };
        let head = index + 1 == layout.active;
        let (cx, cy) = ((x as f32 / gauge.cell) as usize, (y as f32 / gauge.cell) as usize);
        let inked = match fill {
            _ if head => true,
            FillVariant::Solid => true,
            FillVariant::Gradient => {
                let level = (1.0 - (layout.active - 1 - index) as f32 * HEAD_FALLOFF).max(TAIL_LEVEL);
                matrix.is_inked(cx as i32, cy as i32, level)
            }
            textured => texture_inked(textured, cx, cy),
        };
        let shown = ((elapsed - index as f32 * STAGGER) / FADE).clamp(0.0, 1.0);
        if !inked || shown <= 0.0 {
            return faint;
        }
        let mut top = ink;
        top.a = (top.a as f32 * shown).round() as u8;
        over(top, faint)
    })
}

/// A glow behind the lit blocks, padded on every side.
pub fn glow(gauge: &Gauge, level: Bloom, color: Color) -> Image {
    let layout = layout(gauge);
    let (columns, rows) = grid(gauge, &layout);
    let pad = bloom::padding(level, gauge.cell) as usize;
    let (width, height) = (columns as usize + 2 * pad, rows as usize + 2 * pad);
    let mask = (0..width * height)
        .map(|index| {
            let x = ((index % width) as f32 + 0.5 - pad as f32) * gauge.cell;
            let y = ((index / width) as f32 + 0.5 - pad as f32) * gauge.cell;
            if matches!(spot(&layout, gauge.size, gauge.cell, x, y), Spot::Lit(_)) { 1.0 } else { 0.0 }
        })
        .collect();
    bloom::glow(mask, width, height, gauge.cell, level, color)
}

/// How long the blocks take to fade in, in milliseconds.
pub fn reveal(layout: &Layout) -> f32 {
    layout.active.saturating_sub(1) as f32 * STAGGER + FADE
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gauge(variant: Variant, ratio: f32) -> Gauge {
        Gauge { variant, size: 220.0, segments: 0, ratio, cell: 2.0 }
    }

    fn style() -> ArcStyle {
        ArcStyle { fill: FillVariant::Gradient, matrix: Matrix::Bayer4, color: Color::from_rgb_u8(255, 0, 0), track: Color::from_rgb_u8(255, 0, 0), elapsed: f32::INFINITY }
    }

    fn pixels(image: &Image) -> Vec<Rgba8Pixel> {
        image.to_rgba8().unwrap().as_slice().to_vec()
    }

    #[test]
    fn each_variant_has_its_own_shape_and_block_count() {
        assert_eq!(layout(&gauge(Variant::Half, 0.5)).height, 110.0);
        assert_eq!(layout(&gauge(Variant::Ring, 0.5)).height, 220.0);
        let dial = layout(&gauge(Variant::Dial, 0.5)).height;
        assert!(dial > 110.0 && dial < 220.0);
        assert_eq!(layout(&gauge(Variant::Half, 0.5)).count, 28);
        assert_eq!(layout(&gauge(Variant::Dial, 0.5)).count, 36);
        assert_eq!(layout(&gauge(Variant::Ring, 0.5)).count, 48);
        assert_eq!(layout(&Gauge { segments: 10, ..gauge(Variant::Half, 0.5) }).count, 10);
    }

    #[test]
    fn the_active_block_is_the_one_holding_the_value() {
        assert_eq!(layout(&gauge(Variant::Half, 0.0)).active, 0);
        assert_eq!(layout(&gauge(Variant::Half, 0.5)).active, 14);
        assert_eq!(layout(&gauge(Variant::Half, 0.51)).active, 15);
        assert_eq!(layout(&gauge(Variant::Half, 1.0)).active, 28);
        assert_eq!(layout(&gauge(Variant::Half, 1.5)).active, 28);
    }

    #[test]
    fn the_thickness_is_a_whole_number_of_cells() {
        let thickness = layout(&gauge(Variant::Half, 0.5)).thickness;
        assert_eq!(thickness % 2.0, 0.0);
        assert!(thickness >= 4.0);
    }

    #[test]
    fn an_empty_gauge_shows_only_the_faint_track() {
        let image = arc(&gauge(Variant::Half, 0.0), &style());
        let alphas: Vec<u8> = pixels(&image).iter().map(|pixel| pixel.a).filter(|a| *a > 0).collect();
        assert!(!alphas.is_empty());
        assert!(alphas.iter().all(|a| *a == (255.0 * TRACK_OPACITY).round() as u8));
    }

    #[test]
    fn the_half_gauge_fills_from_the_left() {
        let image = arc(&gauge(Variant::Half, 0.3), &style());
        let size = image.size();
        assert_eq!((size.width, size.height), (220, 110));
        let pixels = pixels(&image);
        let solid = |from: u32, to: u32| (0..size.height).flat_map(|y| (from..to).map(move |x| (x, y))).filter(|&(x, y)| pixels[(y * size.width + x) as usize].a == 255).count();
        assert!(solid(0, size.width / 3) > 0);
        assert_eq!(solid(size.width * 2 / 3, size.width), 0);
    }

    #[test]
    fn the_head_steps_out_by_a_cell() {
        let ring = layout(&gauge(Variant::Ring, 0.5));
        let at = |index: u32, radius: f32| {
            let angle = (ring.start + (index as f32 + 0.5) * ring.step).to_radians();
            spot(&ring, 220.0, 2.0, 110.0 + radius * angle.sin(), 110.0 - radius * angle.cos())
        };
        let past = ring.outer + 1.0;
        assert!(matches!(at(ring.active - 1, past), Spot::Lit(_)), "the head reaches past the ring");
        assert!(matches!(at(0, past), Spot::Off), "a block behind it does not");
        assert!(matches!(at(0, ring.outer - 1.0), Spot::Lit(0)));
        assert!(matches!(at(ring.active, ring.outer - 1.0), Spot::Track));
    }

    #[test]
    fn blocks_fade_in_one_after_another() {
        let early = arc(&gauge(Variant::Half, 1.0), &ArcStyle { elapsed: 100.0, ..style() });
        let done = arc(&gauge(Variant::Half, 1.0), &style());
        let strong = |image: &Image| pixels(image).iter().filter(|pixel| pixel.a == 255).count();
        assert!(strong(&early) < strong(&done));
        assert_eq!(reveal(&layout(&gauge(Variant::Half, 1.0))), 27.0 * STAGGER + FADE);
    }

    #[test]
    fn the_glow_follows_the_lit_blocks_only() {
        let none = glow(&gauge(Variant::Half, 0.0), Bloom::Low, Color::from_rgb_u8(255, 0, 0));
        assert!(pixels(&none).iter().all(|pixel| pixel.a == 0));
        let some = glow(&gauge(Variant::Half, 0.5), Bloom::Low, Color::from_rgb_u8(255, 0, 0));
        assert!(pixels(&some).iter().any(|pixel| pixel.a > 0));
    }
}
