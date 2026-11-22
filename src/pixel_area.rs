use slint::{Color, Image, Rgba8Pixel};

use crate::Matrix;
use crate::render::paint;
use crate::style::FillVariant;

const BORDER_ALPHA: f32 = 0.72;
const FEATHER_SHARE: f32 = 0.5;
const OFF_TIER: f32 = 0.4;
const MARKER_ALPHA: f32 = 0.55;
const MIN_SIDE: usize = 8;
const DOTTED_BIAS: f32 = 0.12;
const LIFT_THRESHOLD: f32 = 0.1;
const LIFT_BRIGHTNESS: f32 = 0.22;
const FLOOR_ALPHA: f32 = 0.3;
const DEPTH_ALPHA: f32 = 0.7;
const STAR_SEED: usize = 67;
const STAR_OFFSET: usize = 13;
const STAR_DEPTH: usize = 53;
const STAR_PHASE: usize = 41;
const STAR_SPAN: usize = 14;
const MIN_STARS: usize = 4;
const WINK_SPEED: f32 = 0.35;
const STEADY: f32 = 0.85;
const VISIBLE_LIFT: f32 = 0.55;
const GLINT_THRESHOLD: f32 = 0.9;
const GLINT_SHARE: f32 = 0.6;

pub struct Area {
    pub columns: usize,
    pub rows: usize,
    pub min: f32,
    pub max: Option<f32>,
    pub fill: FillVariant,
    pub color: Color,
    pub reveal: f32,
    pub intensity: f32,
    pub marker: Option<usize>,
    pub stars: bool,
    pub winking: bool,
    pub tick: f32,
}

pub fn side(pixels: f32, cell: f32) -> usize {
    MIN_SIDE.max((pixels / cell.max(1.0)).round() as usize)
}

fn resample(values: &[f32], count: usize) -> Vec<f32> {
    let last = values.len().saturating_sub(1).max(1);
    (0..count)
        .map(|column| {
            let t = column as f32 / count.saturating_sub(1).max(1) as f32 * last as f32;
            let index = (t.floor() as usize).min(values.len().saturating_sub(1));
            let a = values.get(index).copied().unwrap_or(0.0);
            let b = values.get((index + 1).min(values.len().saturating_sub(1))).copied().unwrap_or(a);
            a + (b - a) * (t - index as f32)
        })
        .collect()
}

pub fn tops(values: &[f32], area: &Area) -> Vec<usize> {
    let max = area.max.unwrap_or_else(|| values.iter().copied().fold(area.min + 1e-9, f32::max));
    let span = if max - area.min != 0.0 { max - area.min } else { 1.0 };
    resample(values, area.columns)
        .into_iter()
        .map(|value| {
            let share = ((value - area.min) / span).clamp(0.0, 1.0);
            (1.0 + (1.0 - share) * (area.rows as f32 - 2.0)).round() as usize
        })
        .collect()
}

fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 { 4.0 * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0 }
}

fn column_of(index: usize, count: usize, columns: usize) -> usize {
    (index as f32 / count.saturating_sub(1).max(1) as f32 * (columns - 1) as f32).round() as usize
}

pub struct Star {
    pub index: usize,
    pub depth: f32,
    pub phase: f32,
}

pub fn stars(count: usize, columns: usize) -> Vec<Star> {
    if count == 0 {
        return Vec::new();
    }
    let per = MIN_STARS.max((columns as f32 / STAR_SPAN as f32).round() as usize);
    (0..per)
        .map(|index| {
            let seed = index * STAR_SEED + STAR_OFFSET;
            Star { index: seed % count, depth: ((seed * STAR_DEPTH + 7) % 100) as f32 / 100.0, phase: ((seed * STAR_PHASE) % 360) as f32 }
        })
        .collect()
}

pub fn image(values: &[f32], area: &Area) -> Image {
    let (columns, rows) = (area.columns, area.rows);
    let top = tops(values, area);
    let mut alpha = vec![0.0f32; columns * rows];
    let revealed = (ease_in_out_cubic(area.reveal.clamp(0.0, 1.0)) * columns as f32).ceil() as usize;
    let bias = if area.fill == FillVariant::Dotted { DOTTED_BIAS } else { 0.0 };

    for x in 0..columns.min(revealed) {
        let first = top[x];
        let depth = (rows - first) as f32;
        for y in first..rows {
            let density = (y - first) as f32 / depth;
            if area.fill == FillVariant::Hatched && (x + y) & 3 >= 2 {
                continue;
            }
            let lit = area.fill == FillVariant::Solid || density > Matrix::Bayer4.threshold_at(x as i32, y as i32) - LIFT_THRESHOLD * area.intensity - bias;
            if area.fill == FillVariant::Dotted && !lit {
                continue;
            }
            let strength = (FLOOR_ALPHA + density * DEPTH_ALPHA) * (1.0 + LIFT_BRIGHTNESS * area.intensity);
            alpha[y * columns + x] = if lit { strength } else { strength * OFF_TIER };
        }
        alpha[first * columns + x] = BORDER_ALPHA;
        if depth > 1.0 {
            alpha[(first + 1) * columns + x] = BORDER_ALPHA * FEATHER_SHARE;
        }
    }

    if let Some(index) = area.marker.filter(|_| area.reveal >= 1.0) {
        let marker_x = column_of(index, values.len(), columns);
        let marker_y = top[marker_x];
        for y in marker_y..rows {
            alpha[y * columns + marker_x] = MARKER_ALPHA;
        }
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let (x, y) = (marker_x as i32 + dx, marker_y as i32 + dy);
                if x >= 0 && y >= 0 && (x as usize) < columns && (y as usize) < rows {
                    alpha[y as usize * columns + x as usize] = 1.0;
                }
            }
        }
    }

    if area.stars {
        let mut over = |x: i32, y: i32, amount: f32| {
            if x >= 0 && y >= 0 && (x as usize) < columns && (y as usize) < rows {
                let under = alpha[y as usize * columns + x as usize];
                alpha[y as usize * columns + x as usize] = amount + under * (1.0 - amount);
            }
        };
        for star in stars(values.len(), columns) {
            let x = column_of(star.index, values.len(), columns);
            if x > revealed {
                continue;
            }
            let first = top[x];
            let y = (first as f32 + star.depth * (rows - 1 - first) as f32).round() as i32;
            let wink = if area.winking { ((area.tick + star.phase) * WINK_SPEED).sin().mul_add(0.5, 0.5) } else { STEADY };
            let lift = wink * (0.7 + 0.3 * area.intensity);
            if lift < VISIBLE_LIFT {
                continue;
            }
            over(x as i32, y, lift);
            if wink > GLINT_THRESHOLD {
                let glint = lift * GLINT_SHARE * (wink - GLINT_THRESHOLD) * 10.0;
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    over(x as i32 + dx, y + dy, glint);
                }
            }
        }
    }

    paint(columns as u32, rows as u32, |x, y| {
        let value = alpha[y as usize * columns + x as usize].clamp(0.0, 1.0);
        Rgba8Pixel::new(area.color.red(), area.color.green(), area.color.blue(), (value * 255.0).round() as u8)
    })
}

pub fn readout(label: &str, value: f32) -> String {
    format!("{label}  {}", crate::stat::format_decimals(value, 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area() -> Area {
        Area {
            columns: 40,
            rows: 20,
            min: 0.0,
            max: None,
            fill: FillVariant::Solid,
            color: Color::from_rgb_u8(0, 0, 255),
            reveal: 1.0,
            intensity: 0.0,
            marker: None,
            stars: false,
            winking: false,
            tick: 0.0,
        }
    }

    fn alphas(image: &Image) -> Vec<u8> {
        image.to_rgba8().unwrap().as_slice().iter().map(|pixel| pixel.a).collect()
    }

    #[test]
    fn the_tallest_value_reaches_the_second_row_and_the_lowest_the_floor() {
        let top = tops(&[0.0, 10.0, 0.0], &area());
        assert_eq!(top[20], 1, "the top row stays free so the edge always shows");
        assert_eq!(top[0], 19);
    }

    #[test]
    fn values_are_resampled_to_the_columns() {
        assert_eq!(resample(&[0.0, 10.0], 3), vec![0.0, 5.0, 10.0]);
        assert_eq!(resample(&[4.0], 3), vec![4.0, 4.0, 4.0]);
    }

    #[test]
    fn the_axis_can_be_pinned() {
        let pinned = Area { min: 0.0, max: Some(100.0), ..area() };
        assert_eq!(tops(&[50.0, 50.0], &pinned)[0], 10);
        let clipped = Area { max: Some(5.0), ..area() };
        assert_eq!(tops(&[50.0], &clipped)[0], 1, "a value past the top is clamped");
    }

    #[test]
    fn the_area_has_a_bright_edge_and_fills_below_it() {
        let values = [5.0; 6];
        let cells = alphas(&image(&values, &area()));
        let top = tops(&values, &area())[10];
        let at = |y: usize| cells[y * 40 + 10];
        assert_eq!(at(top - 1), 0, "nothing above the area");
        assert_eq!(at(top), (BORDER_ALPHA * 255.0).round() as u8);
        assert!(at(19) > at(top + 2), "the fill is denser at the floor");
    }

    #[test]
    fn a_gradient_dissolves_upward_and_dotted_keeps_real_gaps() {
        let values = [5.0; 6];
        let count = |fill: FillVariant| alphas(&image(&values, &Area { fill, ..area() })).iter().filter(|&&a| a > 0).count();
        assert!(count(FillVariant::Dotted) < count(FillVariant::Gradient), "dotted cells are empty, gradient cells keep a faint tint");
        assert!(count(FillVariant::Hatched) < count(FillVariant::Solid));
    }

    #[test]
    fn hovering_lifts_the_fill() {
        let values = [5.0; 6];
        let sum = |intensity: f32| alphas(&image(&values, &Area { intensity, fill: FillVariant::Gradient, ..area() })).iter().map(|&a| a as u32).sum::<u32>();
        assert!(sum(1.0) > sum(0.0));
    }

    #[test]
    fn the_entrance_sweeps_in_from_the_left() {
        let values = [5.0; 6];
        let drawn = |reveal: f32| alphas(&image(&values, &Area { reveal, ..area() })).iter().filter(|&&a| a > 0).count();
        assert_eq!(drawn(0.0), 0);
        assert!(drawn(0.5) < drawn(1.0));
        let early = alphas(&image(&values, &Area { reveal: 0.3, ..area() }));
        assert_eq!(early[5 * 40 + 39], 0, "the right end has not arrived");
    }

    #[test]
    fn the_marker_is_a_column_with_a_solid_head() {
        let values = [5.0; 6];
        let cells = alphas(&image(&values, &Area { marker: Some(2), ..area() }));
        let column = column_of(2, 6, 40);
        let top = tops(&values, &area())[column];
        assert_eq!(cells[top * 40 + column], 255);
        assert_eq!(cells[(top + 1) * 40 + column + 1], 255, "a three by three block");
        assert_eq!(cells[10 * 40 + column], (MARKER_ALPHA * 255.0).round() as u8, "a translucent column down to the floor");
    }

    #[test]
    fn stars_are_deterministic_and_spread_over_the_data() {
        let first = stars(30, 70);
        let again = stars(30, 70);
        assert_eq!(first.len(), 5);
        assert!(first.iter().zip(&again).all(|(a, b)| a.index == b.index && a.depth == b.depth));
        assert!(first.iter().all(|star| star.index < 30 && star.depth < 1.0));
        assert!(stars(0, 70).is_empty());
    }

    #[test]
    fn stars_wink_over_time_and_can_be_left_out() {
        let values = [5.0; 12];
        let with = |winking: bool, tick: f32| alphas(&image(&values, &Area { stars: true, winking, tick, ..area() }));
        assert_ne!(with(true, 0.0), with(true, 4.0));
        assert_eq!(with(false, 0.0), with(false, 9.0), "steady stars do not change");
        assert_ne!(with(false, 0.0), alphas(&image(&values, &area())));
    }

    #[test]
    fn the_grid_is_never_tiny() {
        assert_eq!(side(4.0, 2.0), MIN_SIDE);
        assert_eq!(side(300.0, 2.0), 150);
    }
}
