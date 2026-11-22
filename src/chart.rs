use std::collections::BTreeSet;

use slint::{Color, Image, Rgba8Pixel};

use crate::Matrix;
use crate::matrix::quantize;
use crate::render::{paint, pixel};
use crate::stat::format_compact;
use crate::style::{FillVariant, texture_inked};
use crate::trend::Point;

const NICE_STEPS: [f32; 5] = [1.0, 2.0, 2.5, 5.0, 10.0];
const TICK_COUNT: usize = 4;
const AXIS_FONT_WIDTH: f32 = 6.1;
const LABEL_GUTTER: f32 = 10.0;
const AXES_TOP: f32 = 8.0;
const BARE_TOP: f32 = 4.0;
const AXES_BOTTOM: f32 = 22.0;
const BARE_BOTTOM: f32 = 1.0;
const POINT_EDGE: f32 = 6.0;
const LABEL_SPACING: f32 = 72.0;
const CLOSE_TO_LAST: f32 = 0.7;
const MAX_TANGENT_SQUARED: f32 = 9.0;
const DASH_ON: f32 = 4.0;
const DASH_OFF: f32 = 5.0;
const DASH_STEP: f32 = 1.0;
const PERCENT_TICKS: [f32; 5] = [0.0, 25.0, 50.0, 75.0, 100.0];
const LEVEL_STEPS: u32 = 17;
const MIN_DEPTH_CELLS: f32 = 2.0;
const DENSITY_BANDS: [(f32, f32); 5] = [(0.05, 1.0), (0.13, 0.8), (0.25, 0.65), (0.4, 0.5), (0.6, 0.35)];
const DEEPEST_SHARE: f32 = 0.2;
const MAX_BAR: f32 = 36.0;
const BAR_GAP: f32 = 2.0;
const BAR_CAP: f32 = 2.0;
const BAR_FADE: f32 = 0.7;
const BAR_FLOOR: f32 = 0.125;
const STACKED_SHARE: f32 = 0.6;
const GROUPED_SHARE: f32 = 0.7;
const MIN_BAR: f32 = 2.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Scale {
    pub min: f32,
    pub max: f32,
    pub ticks: Vec<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plot {
    pub band: bool,
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    pub from: Point,
    pub first: Point,
    pub second: Point,
    pub to: Point,
}

pub struct Series {
    pub values: Vec<f32>,
    pub low: Vec<f32>,
    pub high: Vec<f32>,
    pub dashed: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stack {
    None,
    Stacked,
    Percent,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Edge {
    pub low: f32,
    pub high: f32,
}

pub struct Chart {
    pub band: bool,
    pub stack: Stack,
    pub count: usize,
    pub width: f32,
    pub height: f32,
    pub axes: bool,
    pub zero: bool,
    pub smooth: bool,
}

pub struct Geometry {
    pub scale: Scale,
    pub plot: Plot,
    pub tick_labels: Vec<String>,
}

pub struct Label {
    pub x: f32,
    pub index: usize,
    pub align: i32,
}

pub fn nice_scale(values: &[f32], zero: bool, count: usize) -> Scale {
    let finite: Vec<f32> = values.iter().copied().filter(|value| value.is_finite()).collect();
    let floor = if zero { 0.0 } else { f32::INFINITY };
    let ceiling = if zero { 0.0 } else { f32::NEG_INFINITY };
    let low = finite.iter().copied().fold(floor, f32::min);
    let high = finite.iter().copied().fold(ceiling, f32::max);
    let (low, high) = if finite.is_empty() { (0.0, 1.0) } else { (low, high) };
    let span = if high - low != 0.0 { high - low } else if high != 0.0 { high.abs() } else { 1.0 };
    let raw = span / count as f32;
    let magnitude = 10f32.powf(raw.log10().floor());
    let step = NICE_STEPS.iter().find(|&&candidate| candidate * magnitude >= raw).copied().unwrap_or(10.0) * magnitude;
    let min = (low / step).floor() * step;
    let max = (high / step).ceil() * step;
    let mut ticks = Vec::new();
    let mut tick = min;
    while tick <= max + step / 2.0 {
        ticks.push((tick * 1e6).round() / 1e6);
        tick += step;
    }
    Scale { min, max, ticks }
}

pub fn format_value(value: f32, unit: &str) -> String {
    format!("{}{unit}", format_compact(value))
}

pub fn geometry(chart: &Chart, values: &[f32], unit: &str) -> Geometry {
    let (scale, tick_labels) = if chart.stack == Stack::Percent {
        let scale = Scale { min: 0.0, max: 100.0, ticks: PERCENT_TICKS.to_vec() };
        let labels: Vec<String> = PERCENT_TICKS.iter().map(|tick| format!("{tick}%")).collect();
        (scale, labels)
    } else {
        let scale = nice_scale(values, chart.zero, TICK_COUNT);
        let labels = scale.ticks.iter().map(|&tick| format_value(tick, unit)).collect();
        (scale, labels)
    };
    let longest = tick_labels.iter().map(|label| label.chars().count()).max().unwrap_or(1).max(1);
    let plot = Plot {
        band: chart.band,
        left: if chart.axes { (longest as f32 * AXIS_FONT_WIDTH).ceil() + LABEL_GUTTER } else { 0.0 },
        right: chart.width - if chart.axes && !chart.band { POINT_EDGE } else { 0.0 },
        top: if chart.axes { AXES_TOP } else { BARE_TOP },
        bottom: chart.height - if chart.axes { AXES_BOTTOM } else { BARE_BOTTOM },
    };
    Geometry { scale, plot, tick_labels }
}

pub fn edges(series: &[Series], stack: Stack) -> Vec<Vec<Edge>> {
    let count = series.first().map_or(0, |line| line.values.len());
    let totals: Vec<f32> = (0..count).map(|index| series.iter().map(|line| line.values.get(index).copied().unwrap_or(0.0).max(0.0)).sum()).collect();
    let mut running = vec![0.0; count];
    series
        .iter()
        .map(|line| {
            line.values
                .iter()
                .enumerate()
                .map(|(index, &raw)| {
                    if stack == Stack::None {
                        return Edge { low: 0.0, high: raw };
                    }
                    let value = raw.max(0.0);
                    let scaled = if stack == Stack::Percent { value / if totals[index] != 0.0 { totals[index] } else { 1.0 } * 100.0 } else { value };
                    let low = running[index];
                    running[index] = low + scaled;
                    Edge { low, high: low + scaled }
                })
                .collect()
        })
        .collect()
}

pub fn scale_values(series: &[Series], edges: &[Vec<Edge>], stack: Stack) -> Vec<f32> {
    if stack == Stack::None {
        series.iter().flat_map(|line| line.values.iter().chain(&line.low).chain(&line.high).copied()).collect()
    } else {
        edges.iter().flatten().map(|edge| edge.high).collect()
    }
}

pub fn readout(series: &[Series], edges: &[Vec<Edge>], index: usize, unit: &str, stack: Stack) -> Vec<String> {
    series
        .iter()
        .zip(edges)
        .map(|(line, edges)| {
            let value = format_value(line.values.get(index).copied().unwrap_or(0.0), unit);
            match (stack, edges.get(index)) {
                (Stack::Percent, Some(edge)) => format!("{value} · {}%", (edge.high - edge.low).round()),
                _ => value,
            }
        })
        .collect()
}

pub fn x_at(plot: &Plot, count: usize, index: usize) -> f32 {
    let width = (plot.right - plot.left).max(1.0);
    if plot.band {
        return plot.left + width / count.max(1) as f32 * (index as f32 + 0.5);
    }
    if count <= 1 {
        return plot.left + width / 2.0;
    }
    plot.left + width / (count - 1) as f32 * index as f32
}

pub fn y_at(plot: &Plot, scale: &Scale, value: f32) -> f32 {
    let span = if scale.max - scale.min != 0.0 { scale.max - scale.min } else { 1.0 };
    plot.bottom - (value - scale.min) / span * (plot.bottom - plot.top)
}

pub fn x_label_indices(count: usize, every: usize) -> BTreeSet<usize> {
    let step = every.max(1);
    let mut shown: BTreeSet<usize> = (0..count).step_by(step).collect();
    if count > 1 {
        let last = count - 1;
        if !shown.contains(&last) {
            let previous = last - last % step;
            if previous > 0 && ((last - previous) as f32) < step as f32 * CLOSE_TO_LAST {
                shown.remove(&previous);
            }
            shown.insert(last);
        }
    }
    shown
}

pub fn labels(plot: &Plot, count: usize) -> Vec<Label> {
    let width = (plot.right - plot.left).max(1.0);
    let every = count.div_ceil(((width / LABEL_SPACING).floor() as usize).max(1));
    x_label_indices(count, every)
        .into_iter()
        .map(|index| {
            let align = if count == 1 || plot.band { 0 } else if index == 0 { -1 } else if index == count - 1 { 1 } else { 0 };
            Label { x: x_at(plot, count, index), index, align }
        })
        .collect()
}

fn linear_segments(points: &[Point]) -> Vec<Segment> {
    points
        .windows(2)
        .map(|pair| {
            let (from, to) = (pair[0], pair[1]);
            let at = |share: f32| Point { x: from.x + (to.x - from.x) * share, y: from.y + (to.y - from.y) * share };
            Segment { from, first: at(1.0 / 3.0), second: at(2.0 / 3.0), to }
        })
        .collect()
}

fn monotone_segments(points: &[Point]) -> Vec<Segment> {
    let count = points.len();
    if count < 3 {
        return linear_segments(points);
    }
    let slopes: Vec<f32> = points.windows(2).map(|pair| (pair[1].y - pair[0].y) / if pair[1].x != pair[0].x { pair[1].x - pair[0].x } else { 1.0 }).collect();
    let mut tangents: Vec<f32> = (0..count)
        .map(|index| match (index.checked_sub(1).map(|before| slopes[before]), slopes.get(index)) {
            (None, after) => after.copied().unwrap_or(0.0),
            (Some(before), None) => before,
            (Some(before), Some(&after)) => if before * after <= 0.0 { 0.0 } else { (before + after) / 2.0 },
        })
        .collect();
    for index in 0..count - 1 {
        let slope = slopes[index];
        if slope == 0.0 {
            tangents[index] = 0.0;
            tangents[index + 1] = 0.0;
            continue;
        }
        let (a, b) = (tangents[index] / slope, tangents[index + 1] / slope);
        let length = a * a + b * b;
        if length > MAX_TANGENT_SQUARED {
            let scale = 3.0 / length.sqrt();
            tangents[index] = scale * a * slope;
            tangents[index + 1] = scale * b * slope;
        }
    }
    points
        .windows(2)
        .enumerate()
        .map(|(index, pair)| {
            let (from, to) = (pair[0], pair[1]);
            let third = (to.x - from.x) / 3.0;
            Segment {
                from,
                first: Point { x: from.x + third, y: from.y + tangents[index] * third },
                second: Point { x: to.x - third, y: to.y - tangents[index + 1] * third },
                to,
            }
        })
        .collect()
}

pub fn segments(points: &[Point], smooth: bool) -> Vec<Segment> {
    if smooth { monotone_segments(points) } else { linear_segments(points) }
}

pub fn path(points: &[Point], segments: &[Segment]) -> String {
    let Some(first) = points.first() else { return String::new() };
    if segments.is_empty() {
        return format!("M {:.1} {:.1}", first.x, first.y);
    }
    let mut commands = format!("M {:.1} {:.1}", first.x, first.y);
    for segment in segments {
        commands.push_str(&format!(
            " C {:.1} {:.1} {:.1} {:.1} {:.1} {:.1}",
            segment.first.x, segment.first.y, segment.second.x, segment.second.y, segment.to.x, segment.to.y
        ));
    }
    commands
}

pub fn dashed_path(segments: &[Segment]) -> String {
    let mut commands = String::new();
    let mut travelled = 0.0;
    let mut last: Option<Point> = None;
    let mut pen_down = false;
    for segment in segments {
        let length = ((segment.to.x - segment.from.x).powi(2) + (segment.to.y - segment.from.y).powi(2)).sqrt();
        let steps = (length / DASH_STEP).ceil().max(1.0) as usize;
        for step in 0..=steps {
            let t = step as f32 / steps as f32;
            let point = Point {
                x: bezier(segment.from.x, segment.first.x, segment.second.x, segment.to.x, t),
                y: bezier(segment.from.y, segment.first.y, segment.second.y, segment.to.y, t),
            };
            if let Some(previous) = last {
                travelled += ((point.x - previous.x).powi(2) + (point.y - previous.y).powi(2)).sqrt();
            }
            if travelled % (DASH_ON + DASH_OFF) < DASH_ON {
                let command = if pen_down { 'L' } else { 'M' };
                commands.push_str(&format!("{command} {:.1} {:.1} ", point.x, point.y));
                pen_down = true;
            } else {
                pen_down = false;
            }
            last = Some(point);
        }
    }
    commands.trim_end().to_string()
}

fn bezier(from: f32, first: f32, second: f32, to: f32, t: f32) -> f32 {
    let u = 1.0 - t;
    u * u * u * from + 3.0 * u * u * t * first + 3.0 * u * t * t * second + t * t * t * to
}

fn curve_y(segments: &[Segment], x: f32) -> Option<f32> {
    let segment = segments.iter().find(|segment| x >= segment.from.x && x <= segment.to.x)?;
    let run = segment.to.x - segment.from.x;
    let t = if run > 0.0 { (x - segment.from.x) / run } else { 1.0 };
    Some(bezier(segment.from.y, segment.first.y, segment.second.y, segment.to.y, t))
}

pub fn points(plot: &Plot, scale: &Scale, values: &[f32], count: usize) -> Vec<Point> {
    values.iter().enumerate().map(|(index, &value)| Point { x: x_at(plot, count, index), y: y_at(plot, scale, value) }).collect()
}

pub struct Band<'a> {
    pub chart: &'a Chart,
    pub series: &'a Series,
    pub geometry: &'a Geometry,
    pub cell: f32,
    pub level: f32,
    pub matrix: Matrix,
    pub color: Color,
}

pub fn band(options: &Band) -> Image {
    let Band { chart, series, geometry, cell, level, matrix, color } = *options;
    let cell = cell.max(1.0);
    let (columns, rows) = ((chart.width / cell).ceil().max(1.0) as u32, (chart.height / cell).ceil().max(1.0) as u32);
    let (high, low) = (
        points(&geometry.plot, &geometry.scale, &series.high, chart.count),
        points(&geometry.plot, &geometry.scale, &series.low, chart.count),
    );
    let (high, low) = (segments(&high, chart.smooth), segments(&low, chart.smooth));
    let (ink, clear) = (pixel(color), Rgba8Pixel::new(0, 0, 0, 0));
    paint(columns, rows, |x, y| {
        let (px, py) = ((x as f32 + 0.5) * cell, (y as f32 + 0.5) * cell);
        let (Some(top), Some(bottom)) = (curve_y(&high, px), curve_y(&low, px)) else { return clear };
        let inside = py >= top.min(bottom) && py <= top.max(bottom);
        if inside && matrix.is_inked(x as i32, y as i32, level) { ink } else { clear }
    })
}

pub struct Area<'a> {
    pub chart: &'a Chart,
    pub series: &'a [Series],
    pub geometry: &'a Geometry,
    pub colors: &'a [Color],
    pub cell: f32,
    pub level: f32,
    pub matrix: Matrix,
    pub fill: FillVariant,
}

fn density(depth: f32, plot_height: f32, cell: f32) -> f32 {
    DENSITY_BANDS
        .iter()
        .find(|&&(reach, _)| depth < (reach * plot_height).max(cell * MIN_DEPTH_CELLS))
        .map_or(DEEPEST_SHARE, |&(_, share)| share)
}

pub fn area(options: &Area) -> Image {
    let Area { chart, series, geometry, colors, cell, level, matrix, fill } = *options;
    let cell = cell.max(1.0);
    let (columns, rows) = ((chart.width / cell).ceil().max(1.0) as u32, (chart.height / cell).ceil().max(1.0) as u32);
    let (scale, plot) = (&geometry.scale, &geometry.plot);
    let edges = edges(series, chart.stack);
    let baseline = y_at(plot, scale, scale.min.max(0.0));
    let shapes: Vec<(Vec<Segment>, Option<Vec<Segment>>)> = edges
        .iter()
        .map(|row| {
            let top = points(plot, scale, &row.iter().map(|edge| edge.high).collect::<Vec<_>>(), chart.count);
            let bottom = (chart.stack != Stack::None).then(|| segments(&points(plot, scale, &row.iter().map(|edge| edge.low).collect::<Vec<_>>(), chart.count), chart.smooth));
            (segments(&top, chart.smooth), bottom)
        })
        .collect();
    let plot_height = plot.bottom - plot.top;
    let clear = Rgba8Pixel::new(0, 0, 0, 0);
    paint(columns, rows, |x, y| {
        let (px, py) = ((x as f32 + 0.5) * cell, (y as f32 + 0.5) * cell);
        for (index, (top, bottom)) in shapes.iter().enumerate().rev() {
            let Some(top_y) = curve_y(top, px) else { continue };
            let bottom_y = match bottom {
                Some(bottom) => curve_y(bottom, px).unwrap_or(baseline),
                None => baseline,
            };
            if py < top_y.min(bottom_y) || py > top_y.max(bottom_y) {
                continue;
            }
            let inked = match fill {
                FillVariant::Gradient => {
                    let tone = quantize(level * density(py - top_y.min(bottom_y), plot_height, cell), LEVEL_STEPS);
                    matrix.is_inked(x as i32, y as i32, tone)
                }
                FillVariant::Solid => true,
                textured => texture_inked(textured, x as usize, y as usize),
            };
            return if inked { pixel(colors[index % colors.len().max(1)]) } else { clear };
        }
        clear
    })
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Bar {
    pub series: usize,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub fn bars(chart: &Chart, geometry: &Geometry, series: &[Series]) -> Vec<Vec<Bar>> {
    let (plot, scale) = (&geometry.plot, &geometry.scale);
    let edges = edges(series, chart.stack);
    let band = (plot.right - plot.left) / chart.count.max(1) as f32;
    let zero = y_at(plot, scale, 0.0);
    (0..chart.count)
        .map(|index| {
            let centre = x_at(plot, chart.count, index);
            let row: Vec<f32> = if chart.stack == Stack::Percent {
                edges.iter().map(|line| line.get(index).map_or(0.0, |edge| edge.high - edge.low)).collect()
            } else {
                series.iter().map(|line| line.values.get(index).copied().unwrap_or(0.0)).collect()
            };
            if chart.stack != Stack::None {
                let width = MIN_BAR.max((MAX_BAR.min(band * STACKED_SHARE)).round());
                let left = (centre - width / 2.0).round();
                let mut top = zero;
                let mut running = 0.0;
                return row
                    .iter()
                    .enumerate()
                    .filter(|&(_, &value)| value > 0.0)
                    .filter_map(|(line, &value)| {
                        running += value;
                        let next = y_at(plot, scale, running).round();
                        let bottom = if top == zero { zero.round() } else { top - BAR_GAP };
                        let height = (bottom - next).max(0.0);
                        top = next;
                        (height > 0.0).then_some(Bar { series: line, x: left, y: next, w: width, h: height })
                    })
                    .collect();
            }
            let count = row.len() as f32;
            let width = MIN_BAR.max((MAX_BAR.min((band * GROUPED_SHARE - BAR_GAP * (count - 1.0)) / count)).floor());
            let group = width * count + BAR_GAP * (count - 1.0);
            let left = (centre - group / 2.0).round();
            row.iter()
                .enumerate()
                .map(|(line, &value)| {
                    let (end, base) = (y_at(plot, scale, value).round(), zero.round());
                    Bar { series: line, x: left + line as f32 * (width + BAR_GAP), y: base.min(end), w: width, h: (base - end).abs() }
                })
                .collect()
        })
        .collect()
}

pub fn bar_tops(chart: &Chart, geometry: &Geometry, columns: &[Vec<Bar>], series: usize) -> Vec<Point> {
    let baseline = y_at(&geometry.plot, &geometry.scale, 0.0);
    columns
        .iter()
        .enumerate()
        .map(|(index, column)| Point { x: x_at(&geometry.plot, chart.count, index), y: column.iter().find(|bar| bar.series == series).map_or(baseline, |bar| bar.y) })
        .collect()
}

pub fn totals(chart: &Chart, series: &[Series]) -> Vec<f32> {
    (0..chart.count)
        .map(|index| {
            let values = series.iter().map(|line| line.values.get(index).copied().unwrap_or(0.0));
            match chart.stack {
                Stack::None => values.fold(0.0, f32::max),
                Stack::Stacked | Stack::Percent => values.map(|value| value.max(0.0)).sum(),
            }
        })
        .collect()
}

pub fn max_index(totals: &[f32]) -> usize {
    totals.iter().enumerate().fold((0, f32::NEG_INFINITY), |best, (index, &total)| if total > best.1 { (index, total) } else { best }).0
}

pub struct BarFill<'a> {
    pub chart: &'a Chart,
    pub geometry: &'a Geometry,
    pub bars: &'a [Vec<Bar>],
    pub colors: &'a [Color],
    pub cell: f32,
    pub level: f32,
    pub matrix: Matrix,
    pub fill: FillVariant,
    pub solid: Option<usize>,
}

pub fn bars_image(options: &BarFill) -> Image {
    let BarFill { chart, geometry, bars, colors, cell, level, matrix, fill, solid } = *options;
    let cell = cell.max(1.0);
    let (columns, rows) = ((chart.width / cell).ceil().max(1.0) as u32, (chart.height / cell).ceil().max(1.0) as u32);
    let fade = (geometry.plot.bottom - geometry.plot.top) * BAR_FADE;
    let clear = Rgba8Pixel::new(0, 0, 0, 0);
    paint(columns, rows, |x, y| {
        let (px, py) = ((x as f32 + 0.5) * cell, (y as f32 + 0.5) * cell);
        if py > geometry.plot.bottom + 1.0 {
            return clear;
        }
        for (index, column) in bars.iter().enumerate() {
            let Some(bar) = column.iter().find(|bar| px >= bar.x && px < bar.x + bar.w && py >= bar.y && py < bar.y + bar.h) else { continue };
            let ink = pixel(colors[bar.series % colors.len().max(1)]);
            let depth = py - bar.y;
            let inked = depth < BAR_CAP
                || solid == Some(index)
                || match fill {
                    FillVariant::Gradient => {
                        let tone = BAR_FLOOR + (level - BAR_FLOOR) * (1.0 - depth / fade).clamp(0.0, 1.0);
                        matrix.is_inked(x as i32, y as i32, quantize(tone, LEVEL_STEPS))
                    }
                    FillVariant::Solid => true,
                    textured => texture_inked(textured, x as usize, y as usize),
                };
            return if inked { ink } else { clear };
        }
        clear
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chart(count: usize) -> Chart {
        Chart { band: false, stack: Stack::None, count, width: 400.0, height: 200.0, axes: true, zero: false, smooth: true }
    }

    #[test]
    fn the_scale_covers_the_values_with_round_ticks() {
        let scale = nice_scale(&[3.0, 47.0, 82.0], true, 4);
        assert_eq!(scale.min, 0.0);
        assert_eq!(scale.max, 100.0);
        assert_eq!(scale.ticks, vec![0.0, 25.0, 50.0, 75.0, 100.0]);
    }

    #[test]
    fn without_zero_the_scale_fits_the_data() {
        let scale = nice_scale(&[102.0, 118.0], false, 4);
        assert!(scale.min > 0.0 && scale.min <= 102.0);
        assert!(scale.max >= 118.0);
    }

    #[test]
    fn a_flat_or_empty_series_still_gets_a_scale() {
        assert!(nice_scale(&[5.0, 5.0], false, 4).ticks.len() >= 2);
        assert_eq!(nice_scale(&[], true, 4).min, 0.0);
    }

    #[test]
    fn axes_leave_room_for_the_labels() {
        let with = geometry(&chart(5), &[0.0, 1200.0], "");
        let bare = geometry(&Chart { axes: false, ..chart(5) }, &[0.0, 1200.0], "");
        assert!(with.plot.left > bare.plot.left && bare.plot.left == 0.0);
        assert!(with.plot.bottom < bare.plot.bottom);
    }

    #[test]
    fn points_sit_on_the_plot_edges_and_between() {
        let geometry = geometry(&chart(3), &[0.0, 10.0], "");
        let plot = geometry.plot;
        assert_eq!(x_at(&plot, 3, 0), plot.left);
        assert_eq!(x_at(&plot, 3, 2), plot.right);
        assert_eq!(x_at(&plot, 1, 0), plot.left + (plot.right - plot.left) / 2.0);
        assert!(y_at(&plot, &geometry.scale, geometry.scale.max) < y_at(&plot, &geometry.scale, geometry.scale.min));
    }

    #[test]
    fn the_curve_never_overshoots_its_points() {
        let points = [Point { x: 0.0, y: 10.0 }, Point { x: 10.0, y: 0.0 }, Point { x: 20.0, y: 0.0 }, Point { x: 30.0, y: 10.0 }];
        for segment in segments(&points, true) {
            let (low, high) = (segment.from.y.min(segment.to.y), segment.from.y.max(segment.to.y));
            for control in [segment.first.y, segment.second.y] {
                assert!(control >= low - 0.001 && control <= high + 0.001);
            }
        }
    }

    #[test]
    fn a_straight_line_is_a_run_of_straight_segments() {
        let points = [Point { x: 0.0, y: 0.0 }, Point { x: 10.0, y: 10.0 }];
        let line = segments(&points, false);
        assert_eq!(line.len(), 1);
        assert_eq!(curve_y(&line, 5.0), Some(5.0));
        assert!(path(&points, &line).starts_with("M 0.0 0.0 C"));
    }

    #[test]
    fn labels_include_the_ends_and_thin_out_in_between() {
        let shown = x_label_indices(12, 3);
        assert!(shown.contains(&0) && shown.contains(&11));
        assert!(!shown.contains(&1));
        assert!(!shown.contains(&9), "a label close to the last one makes way for it");
    }

    #[test]
    fn a_dashed_line_is_made_of_separate_strokes() {
        let points = [Point { x: 0.0, y: 0.0 }, Point { x: 100.0, y: 0.0 }];
        let dashes = dashed_path(&segments(&points, false));
        let strokes = dashes.matches('M').count();
        assert!(strokes > 5, "{dashes}");
        assert!(dashes.len() < path(&points, &segments(&points, false)).len() * 30);
    }

    #[test]
    fn the_first_and_last_labels_hug_the_edges() {
        let plot = geometry(&chart(6), &[0.0, 10.0], "").plot;
        let labels = labels(&plot, 6);
        assert_eq!(labels.first().map(|label| label.align), Some(-1));
        assert_eq!(labels.last().map(|label| label.align), Some(1));
    }

    #[test]
    fn a_band_fills_the_space_between_two_lines() {
        let chart = chart(3);
        let series = Series { values: vec![5.0, 5.0, 5.0], low: vec![2.0, 2.0, 2.0], high: vec![8.0, 8.0, 8.0], dashed: false };
        let geometry = geometry(&chart, &[2.0, 8.0], "");
        let options = Band { chart: &chart, series: &series, geometry: &geometry, cell: 2.0, level: 1.0, matrix: Matrix::Bayer4, color: Color::from_rgb_u8(255, 0, 0) };
        let image = band(&options).to_rgba8().unwrap();
        let (width, height) = (image.width() as usize, image.height() as usize);
        let inked = |x: usize, y: usize| image.as_slice()[y * width + x].a > 0;
        let (middle_x, outside_y) = (width / 2, 1);
        let middle_y = ((geometry.plot.top + geometry.plot.bottom) / 2.0 / 2.0) as usize;
        assert!(inked(middle_x, middle_y.min(height - 1)));
        assert!(!inked(middle_x, outside_y));
    }

    fn lines(rows: &[&[f32]]) -> Vec<Series> {
        rows.iter().map(|values| Series { values: values.to_vec(), low: Vec::new(), high: Vec::new(), dashed: false }).collect()
    }

    #[test]
    fn overlapping_series_each_start_at_the_baseline() {
        let edges = edges(&lines(&[&[1.0, 2.0], &[3.0, 4.0]]), Stack::None);
        assert_eq!(edges[1][0], Edge { low: 0.0, high: 3.0 });
    }

    #[test]
    fn stacked_series_sit_on_the_ones_before() {
        let edges = edges(&lines(&[&[1.0, 2.0], &[3.0, 4.0]]), Stack::Stacked);
        assert_eq!(edges[0][1], Edge { low: 0.0, high: 2.0 });
        assert_eq!(edges[1][1], Edge { low: 2.0, high: 6.0 });
    }

    #[test]
    fn percent_series_fill_the_whole_height() {
        let edges = edges(&lines(&[&[1.0, 0.0], &[3.0, 0.0]]), Stack::Percent);
        assert_eq!(edges[1][0], Edge { low: 25.0, high: 100.0 });
        assert_eq!(edges[1][1].high, 0.0, "an empty column does not divide by zero");
    }

    #[test]
    fn a_percent_chart_has_a_fixed_scale_in_percent() {
        let chart = Chart { stack: Stack::Percent, ..chart(3) };
        let geometry = geometry(&chart, &[5.0], "k");
        assert_eq!((geometry.scale.min, geometry.scale.max), (0.0, 100.0));
        assert_eq!(geometry.tick_labels.last().map(String::as_str), Some("100%"));
    }

    #[test]
    fn the_percent_readout_adds_the_share() {
        let series = lines(&[&[1.0], &[3.0]]);
        let edges = edges(&series, Stack::Percent);
        assert_eq!(readout(&series, &edges, 0, "", Stack::Percent), vec!["1 · 25%", "3 · 75%"]);
        assert_eq!(readout(&series, &edges, 0, "k", Stack::None), vec!["1k", "3k"]);
    }

    fn area_pixels(stack: Stack, rows: &[&[f32]], fill: FillVariant) -> (Vec<Rgba8Pixel>, usize, Geometry) {
        let chart = Chart { stack, zero: true, ..chart(3) };
        let series = lines(rows);
        let edges = edges(&series, stack);
        let geometry = geometry(&chart, &scale_values(&series, &edges, stack), "");
        let colors = [Color::from_rgb_u8(255, 0, 0), Color::from_rgb_u8(0, 255, 0)];
        let options = Area { chart: &chart, series: &series, geometry: &geometry, colors: &colors, cell: 2.0, level: 1.0, matrix: Matrix::Bayer4, fill };
        let image = area(&options).to_rgba8().unwrap();
        (image.as_slice().to_vec(), image.width() as usize, geometry)
    }

    #[test]
    fn the_area_is_inked_under_the_line_and_clear_above_it() {
        let (pixels, width, geometry) = area_pixels(Stack::None, &[&[5.0, 5.0, 5.0]], FillVariant::Solid);
        let column = width / 2;
        let line_y = y_at(&geometry.plot, &geometry.scale, 5.0);
        let at = |y: f32| pixels[(y / 2.0) as usize * width + column];
        assert!(at(line_y + 6.0).a > 0);
        assert_eq!(at(line_y - 6.0).a, 0);
    }

    #[test]
    fn the_fill_thins_out_away_from_the_line() {
        let (pixels, width, geometry) = area_pixels(Stack::None, &[&[10.0, 10.0, 10.0]], FillVariant::Gradient);
        let line_y = y_at(&geometry.plot, &geometry.scale, 10.0);
        let row = |y: f32| {
            let start = (y / 2.0) as usize * width;
            pixels[start..start + width].iter().filter(|pixel| pixel.a > 0).count()
        };
        assert!(row(line_y + 3.0) > row(geometry.plot.bottom - 6.0));
    }

    #[test]
    fn a_later_series_covers_the_one_behind_it() {
        let (pixels, width, geometry) = area_pixels(Stack::None, &[&[8.0, 8.0, 8.0], &[4.0, 4.0, 4.0]], FillVariant::Solid);
        let y = y_at(&geometry.plot, &geometry.scale, 2.0);
        let pixel = pixels[(y / 2.0) as usize * width + width / 2];
        assert_eq!((pixel.r, pixel.g), (0, 255), "the front series wins where both are present");
    }

    #[test]
    fn stacked_areas_do_not_overlap() {
        let (pixels, width, geometry) = area_pixels(Stack::Stacked, &[&[4.0, 4.0, 4.0], &[4.0, 4.0, 4.0]], FillVariant::Solid);
        let upper = pixels[(y_at(&geometry.plot, &geometry.scale, 6.0) / 2.0) as usize * width + width / 2];
        let lower = pixels[(y_at(&geometry.plot, &geometry.scale, 2.0) / 2.0) as usize * width + width / 2];
        assert_eq!(upper.g, 255);
        assert_eq!(lower.r, 255);
    }

    fn bar_chart(stack: Stack, count: usize) -> (Chart, Series) {
        let chart = Chart { band: true, stack, zero: true, ..chart(count) };
        (chart, Series { values: vec![4.0, 8.0, 2.0][..count].to_vec(), low: Vec::new(), high: Vec::new(), dashed: false })
    }

    fn bar_geometry(chart: &Chart, series: &[Series]) -> Geometry {
        let edges = edges(series, chart.stack);
        geometry(chart, &scale_values(series, &edges, chart.stack), "")
    }

    #[test]
    fn categories_own_a_slot_each_in_a_band_layout() {
        let (chart, line) = bar_chart(Stack::None, 3);
        let geometry = bar_geometry(&chart, std::slice::from_ref(&line));
        let slot = (geometry.plot.right - geometry.plot.left) / 3.0;
        assert_eq!(x_at(&geometry.plot, 3, 0), geometry.plot.left + slot / 2.0);
        assert!(labels(&geometry.plot, 3).iter().all(|label| label.align == 0));
        assert_eq!(geometry.plot.right, chart.width, "bars need no edge gap");
    }

    #[test]
    fn a_bar_is_as_tall_as_its_value_and_never_wider_than_the_cap() {
        let (chart, line) = bar_chart(Stack::None, 3);
        let series = [line];
        let geometry = bar_geometry(&chart, &series);
        let columns = bars(&chart, &geometry, &series);
        let (short, tall) = (columns[0][0], columns[1][0]);
        assert!(tall.h > short.h * 1.9 && tall.h < short.h * 2.1);
        assert!(columns.iter().flatten().all(|bar| bar.w <= MAX_BAR));
        assert_eq!(tall.y + tall.h, y_at(&geometry.plot, &geometry.scale, 0.0).round());
    }

    #[test]
    fn grouped_bars_sit_side_by_side_and_stacked_ones_on_top() {
        let first = Series { values: vec![4.0, 4.0], low: Vec::new(), high: Vec::new(), dashed: false };
        let second = Series { values: vec![6.0, 6.0], low: Vec::new(), high: Vec::new(), dashed: false };
        let series = [first, second];
        let grouped = Chart { band: true, zero: true, ..chart(2) };
        let columns = bars(&grouped, &bar_geometry(&grouped, &series), &series);
        assert!(columns[0][1].x > columns[0][0].x + columns[0][0].w - 0.1);
        assert_eq!(columns[0][0].y + columns[0][0].h, columns[0][1].y + columns[0][1].h);

        let stacked = Chart { band: true, stack: Stack::Stacked, zero: true, ..chart(2) };
        let columns = bars(&stacked, &bar_geometry(&stacked, &series), &series);
        assert_eq!(columns[0][0].x, columns[0][1].x);
        assert!(columns[0][1].y + columns[0][1].h <= columns[0][0].y, "the second segment sits above the first, with a gap");
    }

    #[test]
    fn totals_and_the_tallest_bar_follow_the_layout() {
        let first = Series { values: vec![1.0, 5.0], low: Vec::new(), high: Vec::new(), dashed: false };
        let second = Series { values: vec![4.0, 2.0], low: Vec::new(), high: Vec::new(), dashed: false };
        let series = [first, second];
        assert_eq!(totals(&Chart { band: true, ..chart(2) }, &series), vec![4.0, 5.0]);
        let stacked = Chart { band: true, stack: Stack::Stacked, ..chart(2) };
        assert_eq!(totals(&stacked, &series), vec![5.0, 7.0]);
        assert_eq!(max_index(&[5.0, 7.0, 6.0]), 1);
    }

    #[test]
    fn a_bar_has_a_solid_cap_and_thins_out_below_it() {
        let (chart, line) = bar_chart(Stack::None, 3);
        let series = [line];
        let geometry = bar_geometry(&chart, &series);
        let columns = bars(&chart, &geometry, &series);
        let colors = [Color::from_rgb_u8(255, 0, 0)];
        let options = BarFill { chart: &chart, geometry: &geometry, bars: &columns, colors: &colors, cell: 2.0, level: 0.75, matrix: Matrix::Bayer4, fill: FillVariant::Gradient, solid: None };
        let image = bars_image(&options).to_rgba8().unwrap();
        let width = image.width() as usize;
        let bar = columns[1][0];
        let row = |y: f32| {
            let start = (y / 2.0) as usize * width + (bar.x / 2.0) as usize;
            image.as_slice()[start..start + (bar.w / 2.0) as usize].iter().filter(|pixel| pixel.a > 0).count()
        };
        assert_eq!(row(bar.y + 1.0), (bar.w / 2.0) as usize, "the cap is solid");
        assert!(row(bar.y + 6.0) > row(bar.y + bar.h - 4.0));
    }

    #[test]
    fn the_solid_column_fills_in_completely() {
        let (chart, line) = bar_chart(Stack::None, 3);
        let series = [line];
        let geometry = bar_geometry(&chart, &series);
        let columns = bars(&chart, &geometry, &series);
        let colors = [Color::from_rgb_u8(255, 0, 0)];
        let options = BarFill { chart: &chart, geometry: &geometry, bars: &columns, colors: &colors, cell: 2.0, level: 0.75, matrix: Matrix::Bayer4, fill: FillVariant::Gradient, solid: Some(1) };
        let image = bars_image(&options).to_rgba8().unwrap();
        let bar = columns[1][0];
        let width = image.width() as usize;
        let inside = (0..(bar.h / 2.0) as usize - 1).flat_map(|dy| (0..(bar.w / 2.0) as usize).map(move |dx| (dx, dy)));
        assert!(inside.clone().all(|(dx, dy)| image.as_slice()[((bar.y / 2.0) as usize + dy) * width + (bar.x / 2.0) as usize + dx].a > 0));
    }

    #[test]
    fn values_are_written_compactly_with_their_unit() {
        assert_eq!(format_value(1200.0, ""), "1.2K");
        assert_eq!(format_value(42.0, "%"), "42%");
    }
}
