use std::f32::consts::TAU;

use slint::{Color, Image, Rgba8Pixel, SharedPixelBuffer};

use crate::Matrix;
use crate::render::pixel;
use crate::stat::format_decimals;
use crate::style::{FillVariant, texture_inked};

const POP: f32 = 2.0;
const DIMMED: f32 = 0.18;
const FLOOR: f32 = 3.0 / 16.0;
const PIE_FLOOR: f32 = 6.0 / 16.0;
const MIN_SIDE: f32 = 8.0;
const FALLOFF: f32 = 1.4;
const THINNEST: f32 = 0.1;
const THICKEST: f32 = 0.9;
const EDGE_REACH: f32 = 1.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Donut,
    Pie,
}

#[derive(Clone, Copy, Debug)]
pub struct Slice {
    pub value: f32,
    pub color: Color,
    pub fill: FillVariant,
}

#[derive(Clone, Copy, Debug)]
pub struct Spec {
    pub size: f32,
    pub cell: f32,
    pub mode: Mode,
    pub thickness: f32,
    pub gap: f32,
    pub matrix: Matrix,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Arc {
    pub share: f32,
    pub from: f32,
    pub to: f32,
}

struct Cell {
    x: i32,
    y: i32,
    slice: usize,
    angle: f32,
}

pub struct Layout {
    pub side: u32,
    pub outer: f32,
    pub inner: f32,
    pub total: f32,
    pub arcs: Vec<Arc>,
}

pub fn arcs(slices: &[Slice]) -> (Vec<Arc>, f32) {
    let values: Vec<f32> = slices.iter().map(|slice| if slice.value.is_finite() { slice.value.max(0.0) } else { 0.0 }).collect();
    let total: f32 = values.iter().sum();
    let mut angle = 0.0;
    let arcs = values
        .iter()
        .map(|&value| {
            let share = if total > 0.0 { value / total } else { 0.0 };
            let from = angle;
            angle += share * TAU;
            Arc { share, from, to: angle }
        })
        .collect();
    (arcs, total)
}

pub fn layout(spec: &Spec, slices: &[Slice]) -> Layout {
    let (arcs, total) = arcs(slices);
    let side = MIN_SIDE.max((spec.size / spec.cell.max(1.0)).floor());
    let outer = side / 2.0 - POP;
    let ring = spec.thickness.clamp(THINNEST, THICKEST);
    let inner = if spec.mode == Mode::Pie { 0.0 } else { (outer * (1.0 - ring)).round() };
    Layout { side: side as u32, outer, inner, total, arcs }
}

fn angle_of(dx: f32, dy: f32) -> f32 {
    let raw = dx.atan2(-dy);
    if raw < 0.0 { raw + TAU } else { raw }
}

fn near_edge(dx: f32, dy: f32, angle: f32, half: f32) -> bool {
    let (sx, sy) = (angle.sin(), -angle.cos());
    let along = dx * sx + dy * sy;
    along > -half && (dx * sy - dy * sx).abs() < half
}

struct Hit {
    index: Option<usize>,
    edge: bool,
}

fn slice_at(dx: f32, dy: f32, arcs: &[Arc], gap: f32) -> Hit {
    let angle = angle_of(dx, dy);
    let shown: Vec<(usize, &Arc)> = arcs.iter().enumerate().filter(|(_, arc)| arc.share > 0.0).collect();
    let Some(&(owner, _)) = shown.iter().find(|(_, arc)| angle >= arc.from && angle < arc.to) else {
        return Hit { index: None, edge: false };
    };
    if shown.len() < 2 {
        return Hit { index: Some(owner), edge: false };
    }
    let half = gap / 2.0;
    let near = |reach: f32| shown.iter().any(|(_, arc)| near_edge(dx, dy, arc.from, reach));
    if gap > 0.0 && near(half) {
        return Hit { index: None, edge: false };
    }
    Hit { index: Some(owner), edge: near(half + EDGE_REACH) }
}

fn inked(fill: FillVariant, matrix: Matrix, x: i32, y: i32, depth: f32, outline: bool, floor: f32) -> bool {
    if fill == FillVariant::Solid || outline {
        return true;
    }
    match fill {
        FillVariant::Gradient => {
            let level = floor + (1.0 - floor) * (1.0 - depth).powf(FALLOFF);
            matrix.is_inked(x, y, level)
        }
        textured => texture_inked(textured, x as usize, y as usize),
    }
}

fn cells(spec: &Spec, slices: &[Slice], layout: &Layout) -> Vec<Cell> {
    let Layout { side, outer, inner, ref arcs, .. } = *layout;
    let side = side as i32;
    let half = side as f32 / 2.0;
    let floor = if spec.mode == Mode::Pie { PIE_FLOOR } else { FLOOR };
    let mut found = Vec::new();
    for y in 0..side {
        for x in 0..side {
            let (dx, dy) = (x as f32 + 0.5 - half, y as f32 + 0.5 - half);
            let radius = dx.hypot(dy);
            if radius >= outer || radius < inner {
                continue;
            }
            let hit = slice_at(dx, dy, arcs, spec.gap);
            let Some(index) = hit.index else { continue };
            let depth = (outer - radius) / (outer - inner).max(1.0);
            let outline = outer - radius < 1.0 || (inner > 0.0 && radius - inner < 1.0) || hit.edge;
            if inked(slices[index].fill, spec.matrix, x, y, depth, outline, floor) {
                found.push(Cell { x, y, slice: index, angle: angle_of(dx, dy) });
            }
        }
    }
    found
}

pub fn hit(spec: &Spec, slices: &[Slice], x: f32, y: f32) -> Option<usize> {
    let layout = layout(spec, slices);
    let half = layout.side as f32 / 2.0;
    let (dx, dy) = (x / spec.cell.max(1.0) - half, y / spec.cell.max(1.0) - half);
    let radius = dx.hypot(dy);
    if radius > layout.outer + POP || radius < layout.inner - 1.0 {
        return None;
    }
    slice_at(dx, dy, &layout.arcs, 0.0).index
}

pub fn image(spec: &Spec, slices: &[Slice], sweep: f32, focus: Option<usize>) -> Image {
    let layout = layout(spec, slices);
    let side = layout.side as i32;
    let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(layout.side, layout.side);
    let pixels = buffer.make_mut_slice();
    let offset = |index: usize| {
        let arc = layout.arcs[index];
        let middle = (arc.from + arc.to) / 2.0;
        ((middle.sin() * POP).round() as i32, (-middle.cos() * POP).round() as i32)
    };
    let found = cells(spec, slices, &layout);
    let mut draw = |only_focus: bool| {
        for cell in found.iter().filter(|cell| cell.angle <= sweep && (Some(cell.slice) == focus) == only_focus) {
            let (shift_x, shift_y) = if Some(cell.slice) == focus { offset(cell.slice) } else { (0, 0) };
            let (x, y) = (cell.x + shift_x, cell.y + shift_y);
            if x < 0 || y < 0 || x >= side || y >= side {
                continue;
            }
            let mut ink = pixel(slices[cell.slice].color);
            if focus.is_some() && Some(cell.slice) != focus {
                ink.a = (ink.a as f32 * DIMMED).round() as u8;
            }
            pixels[(y * side + x) as usize] = ink;
        }
    };
    draw(false);
    draw(true);
    Image::from_rgba8(buffer)
}

pub fn format_value(value: f32, unit: &str) -> String {
    format!("{}{unit}", format_decimals(value, 0))
}

pub fn format_share(share: f32) -> String {
    format!("{}%", format_decimals(share * 100.0, 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slice(value: f32, fill: FillVariant) -> Slice {
        Slice { value, color: Color::from_rgb_u8(255, 0, 0), fill }
    }

    fn spec(mode: Mode) -> Spec {
        Spec { size: 100.0, cell: 2.0, mode, thickness: 0.36, gap: 1.0, matrix: Matrix::Bayer4 }
    }

    fn alpha(image: &Image) -> (Vec<u8>, usize) {
        let buffer = image.to_rgba8().unwrap();
        (buffer.as_slice().iter().map(|pixel| pixel.a).collect(), buffer.width() as usize)
    }

    #[test]
    fn slices_share_the_circle_clockwise_from_twelve() {
        let (arcs, total) = arcs(&[slice(1.0, FillVariant::Solid), slice(3.0, FillVariant::Solid)]);
        assert_eq!(total, 4.0);
        assert_eq!((arcs[0].from, arcs[0].to), (0.0, TAU / 4.0));
        assert!((arcs[1].to - TAU).abs() < 1e-4);
        assert_eq!(arcs[1].share, 0.75);
    }

    #[test]
    fn negative_and_missing_values_count_as_nothing() {
        let (arcs, total) = arcs(&[slice(-5.0, FillVariant::Solid), slice(f32::NAN, FillVariant::Solid), slice(2.0, FillVariant::Solid)]);
        assert_eq!(total, 2.0);
        assert_eq!((arcs[0].share, arcs[1].share, arcs[2].share), (0.0, 0.0, 1.0));
    }

    #[test]
    fn a_point_belongs_to_the_slice_at_its_angle() {
        let (arcs, _) = arcs(&[slice(1.0, FillVariant::Solid), slice(1.0, FillVariant::Solid)]);
        assert_eq!(slice_at(10.0, -2.0, &arcs, 0.0).index, Some(0), "right of twelve o'clock is the first half");
        assert_eq!(slice_at(-10.0, -2.0, &arcs, 0.0).index, Some(1));
    }

    #[test]
    fn a_gap_keeps_the_seam_between_slices_empty() {
        let (arcs, _) = arcs(&[slice(1.0, FillVariant::Solid), slice(1.0, FillVariant::Solid)]);
        assert_eq!(slice_at(0.2, -10.0, &arcs, 2.0).index, None);
        assert_eq!(slice_at(0.2, -10.0, &arcs, 0.0).index, Some(0));
    }

    #[test]
    fn a_donut_has_a_hole_and_a_pie_does_not() {
        let slices = [slice(1.0, FillVariant::Solid)];
        let (donut, width) = alpha(&image(&spec(Mode::Donut), &slices, TAU, None));
        let (pie, _) = alpha(&image(&spec(Mode::Pie), &slices, TAU, None));
        let middle = width / 2 * width + width / 2;
        assert_eq!(donut[middle], 0);
        assert!(pie[middle] > 0);
    }

    #[test]
    fn the_rim_is_solid_even_in_a_gradient() {
        let (cells, width) = alpha(&image(&spec(Mode::Donut), &[slice(1.0, FillVariant::Gradient)], TAU, None));
        let layout = layout(&spec(Mode::Donut), &[slice(1.0, FillVariant::Gradient)]);
        let row = width / 2;
        let rim = (layout.outer.floor() as usize).min(row) - 1;
        assert!(cells[row * width + row + rim] > 0 || cells[row * width + row + rim + 1] > 0);
    }

    #[test]
    fn the_sweep_draws_the_ring_clockwise() {
        let slices = [slice(1.0, FillVariant::Solid)];
        let (early, width) = alpha(&image(&spec(Mode::Donut), &slices, TAU / 4.0, None));
        let layout = layout(&spec(Mode::Donut), &slices);
        let row = (width / 2 - 1) * width + width / 2;
        let (right, left) = (row + layout.outer as usize - 2, row - layout.outer as usize + 2);
        assert!(early[right] > 0, "a quarter turn has reached three o'clock");
        assert_eq!(early[left], 0, "and not yet nine o'clock");
    }

    #[test]
    fn the_spotlit_slice_stays_lit_and_the_others_dim() {
        let slices = [slice(1.0, FillVariant::Solid), slice(1.0, FillVariant::Solid)];
        let (lit, _) = alpha(&image(&spec(Mode::Donut), &slices, TAU, Some(0)));
        assert!(lit.contains(&255));
        assert!(lit.iter().any(|&alpha| alpha > 0 && alpha < 255), "the other slice is dimmed");
    }

    #[test]
    fn a_pointer_finds_the_slice_under_it() {
        let slices = [slice(1.0, FillVariant::Solid), slice(1.0, FillVariant::Solid)];
        let spec = spec(Mode::Donut);
        let layout = layout(&spec, &slices);
        let ring = (layout.outer + layout.inner) / 2.0 * spec.cell;
        let centre = layout.side as f32 / 2.0 * spec.cell;
        assert_eq!(hit(&spec, &slices, centre + ring, centre), Some(0));
        assert_eq!(hit(&spec, &slices, centre - ring, centre), Some(1));
        assert_eq!(hit(&spec, &slices, centre, centre), None, "the hole is not a slice");
    }

    #[test]
    fn values_and_shares_are_written_for_people() {
        assert_eq!(format_value(12345.0, " GB"), "12,345 GB");
        assert_eq!(format_share(0.1234), "12.3%");
        assert_eq!(format_share(1.0), "100%");
    }
}
