use slint::{Color, Image, Rgba8Pixel};

use crate::Matrix;
use crate::render::{cubic_bezier, paint, pixel};
use crate::style::{FillVariant, texture_inked};

const BAND: [(f32, f32); 4] = [(120.0, 4.0 / 16.0), (82.0, 6.0 / 16.0), (48.0, 8.0 / 16.0), (20.0, 11.0 / 16.0)];
const REACH: f32 = 120.0;
const PULSE: [f32; 3] = [4.0 / 16.0, 6.0 / 16.0, 9.0 / 16.0];
const SWEEP_SHARE: f32 = 0.8;
const OPACITY: f32 = 0.6;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Animation {
    Sweep,
    Pulse,
    None,
}

impl Animation {
    pub const ALL: [Self; 3] = [Self::Sweep, Self::Pulse, Self::None];
}

fn ease(progress: f32) -> f32 {
    cubic_bezier([0.65, 0.0, 0.35, 1.0], progress)
}

fn rest(fill: FillVariant, matrix: Matrix, x: u32, y: u32) -> bool {
    match fill {
        FillVariant::Gradient => {
            let level = if x >= 40 { 0.125 } else { 0.375 + (0.125 - 0.375) * ((x as f32 + 0.5) / 40.0) };
            matrix.is_inked(x as i32, y as i32, level.max(0.125))
        }
        FillVariant::Dotted | FillVariant::Hatched => texture_inked(fill, x as usize, y as usize),
        FillVariant::Solid => false,
    }
}

fn sweep(matrix: Matrix, x: u32, y: u32, width_px: f32, cell: f32, phase: f32) -> bool {
    let travelled = ease((phase / SWEEP_SHARE).min(1.0));
    let centre = -REACH + travelled * (width_px + 2.0 * REACH);
    let at = (x as f32 + 0.5) * cell;
    let level = BAND.iter().filter(|(half, _)| (at - centre).abs() <= *half).map(|&(_, level)| level).fold(0.0, f32::max);
    level > 0.0 && matrix.is_inked(x as i32, y as i32, level)
}

fn pulse(matrix: Matrix, x: u32, y: u32, phase: f32) -> bool {
    let percent = phase * 100.0;
    let level = PULSE
        .iter()
        .enumerate()
        .filter(|(index, _)| {
            let on = 10.0 + *index as f32 * 12.0;
            percent >= on && percent <= 100.0 - on
        })
        .map(|(_, &level)| level)
        .fold(0.0, f32::max);
    level > 0.0 && matrix.is_inked(x as i32, y as i32, level)
}

pub struct SkeletonOptions {
    pub cell: f32,
    pub fill: FillVariant,
    pub animation: Animation,
    pub phase: f32,
    pub matrix: Matrix,
    pub color: Color,
}

pub fn skeleton(columns: u32, rows: u32, options: &SkeletonOptions) -> Image {
    let SkeletonOptions { cell, fill, animation, phase, matrix, color } = *options;
    let (columns, rows) = (columns.max(1), rows.max(1));
    let (ink, phase) = (pixel(color), phase.rem_euclid(1.0));
    let width_px = columns as f32 * cell;
    paint(columns, rows, |x, y| {
        let at_rest = rest(fill, matrix, x, y);
        let moving = match animation {
            Animation::Sweep => sweep(matrix, x, y, width_px, cell, phase),
            Animation::Pulse => pulse(matrix, x, y, phase),
            Animation::None => false,
        };
        let strength = match animation {
            Animation::Pulse => 1.0,
            _ => OPACITY,
        };
        let opacity = match (at_rest, moving) {
            (true, true) => 1.0 - (1.0 - OPACITY) * (1.0 - strength),
            (true, false) => OPACITY,
            (false, true) => strength,
            (false, false) => return Rgba8Pixel::new(0, 0, 0, 0),
        };
        Rgba8Pixel::new(ink.r, ink.g, ink.b, (ink.a as f32 * opacity).round() as u8)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(animation: Animation, phase: f32) -> Vec<u8> {
        let options = SkeletonOptions { cell: 2.0, fill: FillVariant::Gradient, animation, phase, matrix: Matrix::Bayer4, color: Color::from_rgb_u8(128, 128, 128) };
        let image = skeleton(100, 6, &options);
        image.to_rgba8().unwrap().as_slice().iter().map(|p| p.a).collect()
    }

    #[test]
    fn at_rest_the_ramp_is_denser_on_the_left() {
        let alpha = frame(Animation::None, 0.0);
        let inked = |from: usize, to: usize| (0..6).flat_map(|row| (from..to).map(move |x| row * 100 + x)).filter(|&index| alpha[index] > 0).count();
        assert!(inked(0, 20) > inked(60, 80));
    }

    #[test]
    fn the_sweep_moves_across_and_rests_at_the_end_of_each_cycle() {
        assert_ne!(frame(Animation::Sweep, 0.1), frame(Animation::Sweep, 0.4));
        assert_eq!(frame(Animation::Sweep, 0.9), frame(Animation::Sweep, 0.95));
        assert_eq!(frame(Animation::Sweep, 0.9), frame(Animation::None, 0.0));
    }

    #[test]
    fn the_pulse_steps_up_and_back_down() {
        let count = |phase: f32| frame(Animation::Pulse, phase).iter().filter(|a| **a > 0).count();
        assert!(count(0.5) > count(0.15));
        assert_eq!(frame(Animation::Pulse, 0.02), frame(Animation::None, 0.0));
    }
}
