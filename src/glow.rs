use slint::{Color, Image, Rgba8Pixel};

use crate::Matrix;
use crate::render::{paint, pixel};
use crate::style::{FillVariant, texture_inked};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Glow {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Top,
    Bottom,
}

impl Glow {
    pub const ALL: [Self; 6] =
        [Self::TopLeft, Self::TopRight, Self::BottomLeft, Self::BottomRight, Self::Top, Self::Bottom];

    pub(crate) fn origin(self) -> (f32, f32, f32) {
        match self {
            Glow::TopLeft => (0.0, 0.0, 1.0),
            Glow::TopRight => (1.0, 0.0, 1.0),
            Glow::BottomLeft => (0.0, 1.0, 1.0),
            Glow::BottomRight => (1.0, 1.0, 1.0),
            Glow::Top => (0.5, 0.0, 1.8),
            Glow::Bottom => (0.5, 1.0, 1.8),
        }
    }
}

#[derive(Clone, Copy)]
pub struct GlowOptions {
    pub glow: Glow,
    pub reach: f32,
    pub strength: f32,
    pub boost: f32,
    pub rim: bool,
    pub radius: f32,
    pub fill: FillVariant,
    pub matrix: Matrix,
    pub color: Color,
}

const RINGS: usize = 9;
const TEXTURE_OPACITY: f32 = 0.2;

struct Ring {
    radius: f32,
    level: f32,
}

struct Stack {
    rings: [Ring; RINGS],
    stretch: f32,
}

impl Stack {
    fn new(reach: f32, strength: f32, stretch: f32) -> Self {
        let rings = std::array::from_fn(|i| {
            let t = (i + 1) as f32 / RINGS as f32;
            Ring { radius: reach * (1.0 - i as f32 / RINGS as f32 * 0.92), level: strength * t.powf(2.2) }
        });
        Self { rings, stretch }
    }

    fn covers(&self, index: usize, dx: f32, dy: f32) -> bool {
        let radius = self.rings[index].radius;
        (dx / (radius * self.stretch)).powi(2) + (dy / radius).powi(2) <= 1.0
    }

    fn level(&self, active: impl Fn(usize) -> bool, dx: f32, dy: f32) -> f32 {
        let innermost = (0..RINGS).rev().find(|&i| active(i) && self.covers(i, dx, dy));
        innermost.map_or(0.0, |i| self.rings[i].level)
    }

    fn layers(&self, active: impl Fn(usize) -> bool, dx: f32, dy: f32) -> usize {
        (0..RINGS).filter(|&i| active(i) && self.covers(i, dx, dy)).count()
    }
}

fn on_the_rim(x: f32, y: f32, columns: f32, rows: f32, radius: f32) -> bool {
    let (half_x, half_y) = (columns / 2.0, rows / 2.0);
    let radius = radius.min(half_x).min(half_y);
    let qx = (x - half_x).abs() - (half_x - radius);
    let qy = (y - half_y).abs() - (half_y - radius);
    let distance = qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - radius;
    distance <= 0.0 && distance > -1.0
}

pub fn card_glow(columns: u32, rows: u32, options: &GlowOptions) -> Image {
    let GlowOptions { glow, reach, strength, boost, rim, radius, fill, matrix, color } = *options;
    let (columns, rows) = (columns.max(1), rows.max(1));
    let (width, height) = (columns as f32, rows as f32);
    let (cx, cy, stretch) = glow.origin();

    let base = Stack::new(reach, strength, stretch);
    let raised = Stack::new(reach * 1.3, (strength * 1.4).min(1.0), stretch);
    let rim_stack = Stack::new(reach * 1.9, 1.0, stretch);
    let always = |_: usize| true;
    let switched_on = |i: usize| boost * RINGS as f32 > (RINGS - 1 - i) as f32;
    let (ink, clear) = (pixel(color), Rgba8Pixel::new(0, 0, 0, 0));

    paint(columns, rows, |x, y| {
        let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
        let (dx, dy) = (px - cx * width, py - cy * height);
        let cell = (x as i32, y as i32);

        let lit_rim = rim && on_the_rim(px, py, width, height, radius);
        if lit_rim && matrix.is_inked(cell.0, cell.1, rim_stack.level(always, dx, dy)) {
            return ink;
        }

        if fill == FillVariant::Gradient {
            let tone = base.level(always, dx, dy).max(raised.level(switched_on, dx, dy));
            return if matrix.is_inked(cell.0, cell.1, tone) { ink } else { clear };
        }

        let layers = base.layers(always, dx, dy) + raised.layers(switched_on, dx, dy);
        if layers == 0 || !texture_inked(fill, x as usize, y as usize) {
            return clear;
        }
        let alpha = 1.0 - (1.0 - TEXTURE_OPACITY).powi(layers as i32);
        Rgba8Pixel { a: (ink.a as f32 * alpha).round() as u8, ..ink }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(glow: Glow, boost: f32, rim: bool, fill: FillVariant) -> GlowOptions {
        GlowOptions {
            glow,
            reach: 30.0,
            strength: 0.8,
            boost,
            rim,
            radius: 4.0,
            fill,
            matrix: Matrix::Bayer4,
            color: Color::from_rgb_u8(255, 255, 255),
        }
    }

    fn lit(options: &GlowOptions) -> Vec<bool> {
        card_glow(80, 40, options).to_rgba8().unwrap().as_slice().iter().map(|p| p.a > 0).collect()
    }

    fn count(lit: &[bool], inside: impl Fn(usize, usize) -> bool) -> usize {
        lit.iter().enumerate().filter(|(i, on)| **on && inside(i % 80, i / 80)).count()
    }

    #[test]
    fn the_glow_is_dense_at_its_corner_and_gone_far_away() {
        let lit = lit(&options(Glow::TopLeft, 0.0, false, FillVariant::Gradient));
        assert!(count(&lit, |x, y| x < 10 && y < 10) > 20);
        assert_eq!(count(&lit, |x, y| x > 50 || y > 35), 0);
    }

    #[test]
    fn hovering_grows_the_glow() {
        let rest = lit(&options(Glow::TopLeft, 0.0, false, FillVariant::Gradient));
        let hover = lit(&options(Glow::TopLeft, 1.0, false, FillVariant::Gradient));
        assert!(count(&hover, |_, _| true) > count(&rest, |_, _| true));
    }

    #[test]
    fn the_rim_lights_the_border_near_the_glow() {
        let without = lit(&options(Glow::TopLeft, 0.0, false, FillVariant::Gradient));
        let with = lit(&options(Glow::TopLeft, 0.0, true, FillVariant::Gradient));
        assert!(count(&with, |x, y| y == 0 && x > 12) > count(&without, |x, y| y == 0 && x > 12));
        assert_eq!(count(&with, |x, y| x > 60 && y > 5 && y < 35 && x < 79), 0);
    }

    #[test]
    fn textured_fills_stack_up_towards_the_origin() {
        let image = card_glow(80, 40, &options(Glow::TopLeft, 0.0, false, FillVariant::Solid)).to_rgba8().unwrap();
        let alpha = |x: usize, y: usize| image.as_slice()[y * 80 + x].a;
        assert!(alpha(1, 1) > alpha(25, 1));
        assert_eq!(alpha(79, 39), 0);
    }
}
