use std::f32::consts::TAU;

use slint::{Color, Image, Rgba8Pixel};

use crate::Matrix;
use crate::render::{paint, pixel};
use crate::style::{FillVariant, texture_inked};

pub const LABEL_ROOM: f32 = 54.0;
const DIMMED: f32 = 0.18;
const LABEL_PUSH: f32 = 12.0;
const GRADIENT_FLOOR: f32 = 1.0 / 16.0;
const GRADIENT_RANGE: f32 = 11.0 / 16.0;
const FALLOFF: f32 = 1.4;
const NICE: [f32; 5] = [1.0, 2.0, 2.5, 5.0, 10.0];
const PICK_REACH: f32 = 0.6;
const DASH_ON: f32 = 1.0;
const DASH_OFF: f32 = 3.0;
const CIRCLE_STEPS: usize = 96;
const SPOKE_LABEL_SIDE: f32 = 0.3;
const LABEL_LINE: f32 = 0.34;
const LABEL_LEAN: f32 = 0.5;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Grid {
    Polygon,
    Circle,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

pub struct Series {
    pub values: Vec<f32>,
    pub color: Color,
    pub fill: FillVariant,
}

pub struct Spec {
    pub side: f32,
    pub cell: f32,
    pub max: Option<f32>,
    pub rings: usize,
    pub grid: Grid,
    pub matrix: Matrix,
}

pub struct Label {
    pub at: Vec2,
    pub align: i32,
    pub line: f32,
}

pub struct Layout {
    pub radius: f32,
    pub top: f32,
    pub rings: Vec<String>,
    pub spokes: Vec<Vec2>,
    pub labels: Vec<Label>,
    pub vertices: Vec<Vec<Vec2>>,
}

pub fn polar(angle: f32, radius: f32) -> Vec2 {
    Vec2 { x: angle.sin() * radius, y: -angle.cos() * radius }
}

pub fn angle_of(point: Vec2) -> f32 {
    let raw = point.x.atan2(-point.y);
    if raw < 0.0 { raw + TAU } else { raw }
}

pub fn nice_max(value: f32) -> f32 {
    if value <= 0.0 {
        return 1.0;
    }
    let magnitude = 10f32.powf(value.log10().floor());
    NICE.iter().find(|&&candidate| candidate * magnitude >= value).copied().unwrap_or(10.0) * magnitude
}

fn edge_radius(angle: f32, a: Vec2, b: Vec2) -> f32 {
    let direction = polar(angle, 1.0);
    let (ex, ey) = (b.x - a.x, b.y - a.y);
    let denominator = direction.x * ey - direction.y * ex;
    if denominator.abs() < 1e-9 {
        return a.x.hypot(a.y);
    }
    (a.x * ey - a.y * ex) / denominator
}

pub fn radius(side: f32) -> f32 {
    (side / 2.0 - LABEL_ROOM).max(0.0)
}

pub fn top(series: &[Series], max: Option<f32>) -> f32 {
    max.unwrap_or_else(|| nice_max(series.iter().flat_map(|line| line.values.iter().copied()).fold(0.0, f32::max)))
}

fn vertices(series: &[Series], axes: usize, top: f32, radius: f32) -> Vec<Vec<Vec2>> {
    let step = TAU / axes.max(1) as f32;
    series
        .iter()
        .map(|line| {
            (0..axes)
                .map(|axis| {
                    let raw = line.values.get(axis).copied().unwrap_or(0.0);
                    let value = if raw.is_finite() { raw.max(0.0) } else { 0.0 };
                    polar(axis as f32 * step, value.min(top) / top * radius)
                })
                .collect()
        })
        .collect()
}

fn ring_points(spec: &Spec, axes: usize, radius: f32) -> Vec<Vec2> {
    match spec.grid {
        Grid::Polygon => (0..axes).map(|axis| polar(axis as f32 * TAU / axes as f32, radius)).collect(),
        Grid::Circle => (0..CIRCLE_STEPS).map(|step| polar(step as f32 * TAU / CIRCLE_STEPS as f32, radius)).collect(),
    }
}

fn dashed(points: &[Vec2]) -> String {
    let mut closed: Vec<Vec2> = points.to_vec();
    closed.push(points[0]);
    let (mut commands, mut travelled, mut pen_down) = (String::new(), 0.0f32, false);
    for pair in closed.windows(2) {
        let (from, to) = (pair[0], pair[1]);
        let length = (to.x - from.x).hypot(to.y - from.y);
        let steps = length.ceil().max(1.0) as usize;
        for step in 0..=steps {
            let t = step as f32 / steps as f32;
            let point = Vec2 { x: from.x + (to.x - from.x) * t, y: from.y + (to.y - from.y) * t };
            if step > 0 {
                travelled += length / steps as f32;
            }
            if travelled % (DASH_ON + DASH_OFF) < DASH_ON {
                commands.push_str(&format!("{} {:.1} {:.1} ", if pen_down { 'L' } else { 'M' }, point.x, point.y));
                pen_down = true;
            } else {
                pen_down = false;
            }
        }
    }
    commands.trim_end().to_string()
}

fn solid(points: &[Vec2]) -> String {
    let mut commands = String::new();
    for (index, point) in points.iter().enumerate() {
        commands.push_str(&format!("{} {:.1} {:.1} ", if index == 0 { 'M' } else { 'L' }, point.x, point.y));
    }
    commands.push('Z');
    commands
}

pub fn layout(axes: &[String], series: &[Series], spec: &Spec) -> Layout {
    let radius = radius(spec.side);
    let top = top(series, spec.max);
    let count = axes.len().max(1);
    let rings = (0..spec.rings.max(1))
        .map(|ring| {
            let points = ring_points(spec, count, radius * (ring + 1) as f32 / spec.rings.max(1) as f32);
            if ring + 1 == spec.rings.max(1) { solid(&points) } else { dashed(&points) }
        })
        .collect();
    let step = TAU / count as f32;
    let labels = axes
        .iter()
        .enumerate()
        .map(|(axis, _)| {
            let angle = axis as f32 * step;
            let sin = angle.sin();
            Label {
                at: polar(angle, radius + LABEL_PUSH),
                align: if sin > SPOKE_LABEL_SIDE { 1 } else if sin < -SPOKE_LABEL_SIDE { -1 } else { 0 },
                line: LABEL_LINE - angle.cos() * LABEL_LEAN,
            }
        })
        .collect();
    Layout { radius, top, rings, spokes: (0..count).map(|axis| polar(axis as f32 * step, radius)).collect(), labels, vertices: vertices(series, axes.len(), top, radius) }
}

pub fn axis_at(count: usize, point: Vec2, side: f32) -> Option<usize> {
    let centre = side / 2.0;
    let offset = Vec2 { x: point.x - centre, y: point.y - centre };
    if offset.x.hypot(offset.y) > radius(side) + LABEL_ROOM * PICK_REACH {
        return None;
    }
    Some((angle_of(offset) / (TAU / count as f32)).round() as usize % count)
}

struct Cover {
    shape: usize,
    outline: bool,
}

fn inked(fill: FillVariant, matrix: Matrix, x: usize, y: usize, band: usize, rings: usize, outline: bool) -> bool {
    if fill == FillVariant::Solid || outline {
        return true;
    }
    match fill {
        FillVariant::Gradient => {
            let level = GRADIENT_FLOOR + GRADIENT_RANGE * (band as f32 / rings as f32).powf(FALLOFF);
            matrix.is_inked(x as i32, y as i32, level)
        }
        textured => texture_inked(textured, x, y),
    }
}

pub fn image(axes: usize, series: &[Series], spec: &Spec, scale: f32, focus: Option<usize>) -> Image {
    let radius = radius(spec.side);
    let top = top(series, spec.max);
    let shapes = vertices(series, axes, top, radius);
    let columns = (spec.side / spec.cell.max(1.0)).floor().max(1.0) as u32;
    let centre = spec.side / 2.0;
    let count = axes.max(1);
    let step = TAU / count as f32;
    let rings = spec.rings.max(1);
    let clear = Rgba8Pixel::new(0, 0, 0, 0);

    paint(columns, columns, |x, y| {
        let (x, y) = (x as usize, y as usize);
        let point = Vec2 { x: (x as f32 + 0.5) * spec.cell - centre, y: (y as f32 + 0.5) * spec.cell - centre };
        let distance = point.x.hypot(point.y);
        if distance > radius * scale + spec.cell {
            return clear;
        }
        let angle = angle_of(point);
        let segment = (angle / step).floor() as usize % count;
        let band = ((distance / (radius * scale.max(1e-6))) * rings as f32).ceil().max(1.0) as usize;
        let covering: Vec<Cover> = shapes
            .iter()
            .enumerate()
            .filter_map(|(index, shape)| {
                let (a, b) = (shape[segment], shape[(segment + 1) % count]);
                let reach = edge_radius(angle, a, b) * scale;
                (distance < reach).then(|| {
                    let (ex, ey) = ((b.x - a.x) * scale, (b.y - a.y) * scale);
                    let length = ex.hypot(ey);
                    let across = if length < 1e-6 { reach - distance } else { (ex * (point.y - a.y * scale) - ey * (point.x - a.x * scale)).abs() / length };
                    Cover { shape: index, outline: across < spec.cell }
                })
            })
            .collect();
        let owner = covering.iter().rev().find(|cover| cover.outline).or_else(|| if covering.len() < 2 { covering.first() } else { covering.get((x + y * 2) % covering.len()) });
        let Some(owner) = owner else { return clear };
        if !inked(series[owner.shape].fill, spec.matrix, x, y, band, rings, owner.outline) {
            return clear;
        }
        let mut ink = pixel(series[owner.shape].color);
        if focus.is_some_and(|index| index != owner.shape) {
            ink.a = (ink.a as f32 * DIMMED).round() as u8;
        }
        ink
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(values: &[f32], fill: FillVariant) -> Series {
        Series { values: values.to_vec(), color: Color::from_rgb_u8(255, 0, 0), fill }
    }

    fn spec(side: f32) -> Spec {
        Spec { side, cell: 3.0, max: None, rings: 4, grid: Grid::Polygon, matrix: Matrix::Bayer4 }
    }

    fn cells(image: &Image) -> (Vec<u8>, usize) {
        let buffer = image.to_rgba8().unwrap();
        (buffer.as_slice().iter().map(|pixel| pixel.a).collect(), buffer.width() as usize)
    }

    #[test]
    fn axes_run_clockwise_from_twelve() {
        let top = polar(0.0, 10.0);
        assert!((top.x).abs() < 1e-4 && (top.y + 10.0).abs() < 1e-4);
        let right = polar(TAU / 4.0, 10.0);
        assert!((right.x - 10.0).abs() < 1e-4 && right.y.abs() < 1e-4);
        assert!((angle_of(right) - TAU / 4.0).abs() < 1e-4);
        assert!(angle_of(Vec2 { x: -1.0, y: 0.0 }) > TAU / 2.0);
    }

    #[test]
    fn the_outer_ring_is_a_round_number_above_the_largest_value() {
        assert_eq!(nice_max(0.0), 1.0);
        assert_eq!(nice_max(73.0), 100.0);
        assert_eq!(nice_max(4.2), 5.0);
        assert_eq!(nice_max(22.0), 25.0);
        assert_eq!(top(&[line(&[3.0, 18.0], FillVariant::Solid)], None), 20.0);
        assert_eq!(top(&[line(&[3.0, 18.0], FillVariant::Solid)], Some(50.0)), 50.0);
    }

    #[test]
    fn vertices_sit_at_their_value_along_their_axis() {
        let shapes = vertices(&[line(&[10.0, 5.0, 0.0, 99.0], FillVariant::Solid)], 4, 10.0, 100.0);
        assert!((shapes[0][0].y + 100.0).abs() < 1e-3, "a full value reaches the outer ring");
        assert!((shapes[0][1].x - 50.0).abs() < 1e-3);
        assert!(shapes[0][2].x.hypot(shapes[0][2].y) < 1e-3);
        assert!(shapes[0][3].x.hypot(shapes[0][3].y) <= 100.001, "values past the top are clamped");
    }

    #[test]
    fn missing_and_negative_values_count_as_nothing() {
        let shapes = vertices(&[line(&[-5.0, f32::NAN], FillVariant::Solid)], 3, 10.0, 100.0);
        assert!(shapes[0].iter().all(|vertex| vertex.x.hypot(vertex.y) < 1e-3));
    }

    #[test]
    fn the_grid_has_a_ring_per_step_with_the_outer_one_solid() {
        let axes: Vec<String> = ["a", "b", "c", "d", "e"].iter().map(|axis| axis.to_string()).collect();
        let layout = layout(&axes, &[line(&[1.0; 5], FillVariant::Solid)], &spec(300.0));
        assert_eq!(layout.rings.len(), 4);
        assert!(layout.rings[3].ends_with('Z'), "the outer ring is a closed line");
        assert!(layout.rings[0].matches('M').count() > 5, "inner rings are dotted");
        assert_eq!(layout.spokes.len(), 5);
        assert_eq!(layout.labels.len(), 5);
        assert_eq!(layout.labels[0].align, 0, "the label at twelve o'clock is centred");
        assert_eq!(layout.labels[1].align, 1, "labels on the right start at the plot");
    }

    #[test]
    fn a_series_fills_its_polygon_and_leaves_the_corners_clear() {
        let spec = spec(300.0);
        let (alpha, width) = cells(&image(4, &[line(&[10.0, 10.0, 10.0, 10.0], FillVariant::Solid)], &spec, 1.0, None));
        let middle = width / 2 * width + width / 2;
        assert!(alpha[middle] > 0);
        assert_eq!(alpha[0], 0, "far outside the plot");
        assert_eq!(alpha[width * width - 1], 0);
    }

    #[test]
    fn the_series_grows_out_from_the_centre() {
        let spec = spec(300.0);
        let count = |scale: f32| cells(&image(4, &[line(&[10.0; 4], FillVariant::Solid)], &spec, scale, None)).0.iter().filter(|&&a| a > 0).count();
        assert!(count(0.0) < 12);
        assert!(count(0.5) < count(1.0));
    }

    #[test]
    fn a_gradient_steps_up_in_density_with_the_rings() {
        let spec = spec(300.0);
        let (alpha, width) = cells(&image(4, &[line(&[10.0; 4], FillVariant::Gradient)], &spec, 1.0, None));
        let row = width / 2;
        let inked = |from: usize, to: usize| (from..to).filter(|&x| alpha[row * width + width / 2 + x] > 0).count();
        assert!(inked(1, 8) < inked(20, 27), "denser towards the outer edge");
    }

    #[test]
    fn the_outline_stays_solid_and_a_later_series_covers_an_earlier_one() {
        let spec = spec(300.0);
        let big = Series { values: vec![10.0; 4], color: Color::from_rgb_u8(255, 0, 0), fill: FillVariant::Solid };
        let small = Series { values: vec![5.0; 4], color: Color::from_rgb_u8(0, 255, 0), fill: FillVariant::Solid };
        let image = image(4, &[big, small], &spec, 1.0, None).to_rgba8().unwrap();
        let width = image.width() as usize;
        let row = width / 2;
        let colours = |from: usize, to: usize| {
            let line = &image.as_slice()[row * width + width / 2 + from..row * width + width / 2 + to];
            (line.iter().any(|pixel| pixel.r == 255 && pixel.g == 0), line.iter().any(|pixel| pixel.g == 255))
        };
        let (_, green_inside) = colours(1, 12);
        assert!(green_inside, "the later series shows where both cover");
        let (red_outside, green_outside) = colours(18, 28);
        assert!(red_outside && !green_outside, "only the larger series reaches the outer cells");
    }

    #[test]
    fn a_spotlit_series_leaves_the_others_dim() {
        let spec = spec(300.0);
        let first = Series { values: vec![10.0; 4], color: Color::from_rgb_u8(255, 0, 0), fill: FillVariant::Solid };
        let second = Series { values: vec![4.0; 4], color: Color::from_rgb_u8(0, 255, 0), fill: FillVariant::Solid };
        let (alpha, _) = cells(&image(4, &[first, second], &spec, 1.0, Some(1)));
        assert!(alpha.contains(&255));
        assert!(alpha.iter().any(|&a| a > 0 && a < 255));
    }

    #[test]
    fn the_pointer_picks_the_nearest_axis() {
        let side = 300.0;
        assert_eq!(axis_at(4, Vec2 { x: 150.0, y: 40.0 }, side), Some(0));
        assert_eq!(axis_at(4, Vec2 { x: 260.0, y: 150.0 }, side), Some(1));
        assert_eq!(axis_at(4, Vec2 { x: 150.0, y: 260.0 }, side), Some(2));
        assert_eq!(axis_at(4, Vec2 { x: 5.0, y: 5.0 }, side), None, "far from the plot");
    }
}
