use slint::{Color, Image, Rgba8Pixel};

use crate::Matrix;
use crate::render::{cubic_bezier, paint, pixel};
use crate::stat::format_decimals;

const EASE: [f32; 4] = [0.23, 1.0, 0.32, 1.0];
const EDGE_FADE: f32 = 0.2;
const TINT_START: f32 = 0.45;
const TINT_FADE: f32 = 0.8;

/// A change from one value to another, formatted with `decimals`.
#[derive(Clone, Copy, Debug)]
pub struct Roll {
    pub from: f32,
    pub to: f32,
    pub decimals: usize,
    pub duration: f32,
    pub stagger: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Glyph {
    /// A digit on its strip. Row `r` shows digit `r % 10`; the strip holds rows 0 to 29.
    /// `tint` is how much of the accent colour it has, 1 while it moves.
    Digit { row: f32, tint: f32 },
    Static(char),
}

fn digits(text: &str) -> Vec<u32> {
    text.chars().filter_map(|char| char.to_digit(10)).collect()
}

fn number(digits: &[u32]) -> f64 {
    digits.iter().fold(0.0, |total, digit| total * 10.0 + *digit as f64)
}

/// Where the strip starts and ends to go from digit `at` to `value`. Going up it rolls
/// forward from the top copy, going down backward from the bottom one, and a spin adds a lap.
fn rows(at: u32, value: u32, up: bool, spin: bool) -> (f32, f32) {
    let (at, value, extra) = (at as f32, value as f32, if spin { 10.0 } else { 0.0 });
    if up {
        (at, value + if value > at { 0.0 } else { 10.0 } + extra)
    } else {
        (at + 20.0, value + 20.0 - if value < at { 0.0 } else { 10.0 } - extra)
    }
}

/// The glyphs of `to` at `elapsed` milliseconds into the change. Digits are aligned
/// from the right with those of `from`; digits `from` does not have start at 0.
pub fn glyphs(roll: &Roll, elapsed: f32) -> Vec<Glyph> {
    let Roll { from, to, decimals, duration, stagger } = *roll;
    let (target, shown) = (format_decimals(to, decimals), format_decimals(from, decimals));
    let shown = digits(&shown);
    let target_digits = digits(&target);
    let delta = (number(&target_digits) - number(&shown)).abs();
    let moving = to != from;
    let mut place = target_digits.len();
    let mut slot = 0;
    target
        .chars()
        .map(|char| {
            let Some(value) = char.to_digit(10) else {
                return Glyph::Static(char);
            };
            place -= 1;
            let at = shown.len().checked_sub(1 + place).map_or(0, |index| shown[index]);
            let delay = slot as f32 * stagger;
            slot += 1;
            let (start, end) = rows(at, value, to > from, delta >= 10f64.powi(place as i32 + 1));
            let rest = value as f32 + 10.0;
            if !moving || (end - start).abs() < 0.01 {
                return Glyph::Digit { row: rest, tint: 0.0 };
            }
            let progress = ((elapsed - delay) / duration).clamp(0.0, 1.0);
            let row = if progress >= 1.0 { rest } else { start + (end - start) * cubic_bezier(EASE, progress) };
            let fade = ((elapsed - delay - TINT_START * duration) / (TINT_FADE * duration)).clamp(0.0, 1.0);
            Glyph::Digit { row, tint: 1.0 - cubic_bezier(EASE, fade) }
        })
        .collect()
}

/// How long a change takes, colour fade included, in milliseconds.
pub fn length(roll: &Roll) -> f32 {
    let slots = digits(&format_decimals(roll.to, roll.decimals)).len().saturating_sub(1);
    slots as f32 * roll.stagger + (TINT_START + TINT_FADE) * roll.duration
}

/// Cells in `color` that cover the top and bottom of a digit window, thinning
/// towards the middle, so the rolling digits fade out at the edges.
pub fn edge(columns: u32, rows: u32, matrix: Matrix, color: Color) -> Image {
    let (columns, rows) = (columns.max(1), rows.max(1));
    let fade = (rows as f32 * EDGE_FADE).round().max(1.0);
    let ink = pixel(color);
    paint(columns, rows, |x, y| {
        let from_edge = y.min(rows - 1 - y) as f32;
        let shown = if from_edge >= fade { 1.0 } else { (from_edge + 0.5) / fade };
        if matrix.is_inked(x as i32, y as i32, shown) { Rgba8Pixel::new(0, 0, 0, 0) } else { ink }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roll(from: f32, to: f32) -> Roll {
        Roll { from, to, decimals: 0, duration: 900.0, stagger: 40.0 }
    }

    fn rows_of(glyphs: &[Glyph]) -> Vec<f32> {
        glyphs.iter().filter_map(|glyph| if let Glyph::Digit { row, .. } = glyph { Some(*row) } else { None }).collect()
    }

    #[test]
    fn separators_stay_put_and_digits_get_a_strip() {
        let glyphs = glyphs(&roll(0.0, 1234.0), f32::INFINITY);
        assert_eq!(glyphs.len(), 5);
        assert_eq!(glyphs[1], Glyph::Static(','));
        assert_eq!(rows_of(&glyphs), [11.0, 12.0, 13.0, 14.0]);
    }

    #[test]
    fn an_unchanged_value_rests_without_tint() {
        let glyphs = glyphs(&roll(42.0, 42.0), 0.0);
        assert_eq!(glyphs, [Glyph::Digit { row: 14.0, tint: 0.0 }, Glyph::Digit { row: 12.0, tint: 0.0 }]);
    }

    #[test]
    fn going_up_rolls_forward_and_going_down_backward() {
        assert_eq!(rows(3, 7, true, false), (3.0, 7.0));
        assert_eq!(rows(7, 3, true, false), (7.0, 13.0));
        assert_eq!(rows(7, 3, false, false), (27.0, 23.0));
        assert_eq!(rows(3, 7, false, false), (23.0, 17.0));
    }

    #[test]
    fn a_big_jump_spins_an_extra_lap_and_stays_on_the_strip() {
        assert_eq!(rows(9, 9, true, true), (9.0, 29.0));
        assert_eq!(rows(0, 0, false, true), (20.0, 0.0));
        let glyphs = glyphs(&roll(100.0, 200.0), 300.0);
        let rows = rows_of(&glyphs);
        assert!(rows[1] > 10.0, "the tens spin a lap although the digit stays 0");
    }

    #[test]
    fn digits_start_one_after_another_from_the_left() {
        let rows = rows_of(&glyphs(&roll(0.0, 99.0), 30.0));
        assert!(rows[0] > 0.0, "the first digit has started");
        assert_eq!(rows[1], 0.0, "the second waits for its stagger");
    }

    #[test]
    fn a_moving_digit_is_tinted_and_fades_back() {
        let tint = |elapsed: f32| match glyphs(&roll(1.0, 5.0), elapsed)[0] {
            Glyph::Digit { tint, .. } => tint,
            Glyph::Static(_) => unreachable!(),
        };
        assert_eq!(tint(0.0), 1.0);
        assert!(tint(900.0) < 1.0);
        assert_eq!(tint(f32::INFINITY), 0.0);
    }

    #[test]
    fn the_change_lasts_until_the_last_digit_has_faded() {
        assert_eq!(length(&roll(0.0, 123.0)), 2.0 * 40.0 + 1.25 * 900.0);
    }

    #[test]
    fn the_edge_covers_the_ends_and_leaves_the_middle_open() {
        let image = edge(8, 20, Matrix::Bayer4, Color::from_rgb_u8(10, 10, 10));
        let alpha: Vec<u8> = image.to_rgba8().unwrap().as_slice().iter().map(|pixel| pixel.a).collect();
        let covered = |row: usize| alpha[row * 8..row * 8 + 8].iter().filter(|a| **a > 0).count();
        assert!(covered(0) > covered(3));
        assert_eq!(covered(10), 0);
    }
}
