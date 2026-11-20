use slint::{Color, Image, Rgba8Pixel};

use crate::Matrix;
use crate::render::{paint, pixel};

const TAU: f32 = std::f32::consts::TAU;
const TRACK: f32 = 0.13;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Variant {
    Ring,
    Dots,
    Pulse,
    Bar,
}

impl Variant {
    pub const ALL: [Self; 4] = [Self::Ring, Self::Dots, Self::Pulse, Self::Bar];

    pub fn still_time(self) -> f32 {
        match self {
            Self::Ring => 0.4,
            Self::Dots => 0.2,
            Self::Pulse => 0.5,
            Self::Bar => 0.9,
        }
    }
}

fn fract(value: f32) -> f32 {
    value - value.floor()
}

fn ease_in_out(value: f32) -> f32 {
    if value < 0.5 { 4.0 * value.powi(3) } else { 1.0 - (-2.0 * value + 2.0).powi(3) / 2.0 }
}

pub fn grid(variant: Variant, height: f32, cell: f32) -> (u32, u32) {
    let rows = ((height / cell).round() as u32).max(6);
    match variant {
        Variant::Bar => (rows * 3, ((rows as f32 / 5.0).round() as u32).max(2)),
        Variant::Dots => (rows * 2, (rows as f32 * 0.62).round() as u32),
        Variant::Ring | Variant::Pulse => (rows, rows),
    }
}

fn ring(columns: u32, rows: u32, x: u32, y: u32, t: f32) -> f32 {
    let dx = (x as f32 + 0.5 - columns as f32 / 2.0) / (columns as f32 / 2.0);
    let dy = (y as f32 + 0.5 - rows as f32 / 2.0) / (rows as f32 / 2.0);
    let radius = dx.hypot(dy);
    if !(0.58..=1.0).contains(&radius) {
        return 0.0;
    }
    let angle = fract(dx.atan2(-dy) / TAU);
    let tail = 0.5 + 0.3 * (0.5 - 0.5 * (t * 2.1).cos());
    let behind = fract(t * 0.85 - angle);
    if behind > tail {
        return TRACK;
    }
    TRACK.max((1.0 - behind / tail).powf(1.4) * 1.1)
}

fn dots(columns: u32, rows: u32, x: u32, y: u32, t: f32) -> f32 {
    let pitch = columns as f32 / 3.0;
    let index = ((x as f32 / pitch).floor() as i32).min(2);
    let dx = (x as f32 + 0.5 - (index as f32 + 0.5) * pitch) / (pitch / 2.0);
    let dy = (y as f32 + 0.5 - rows as f32 / 2.0) / (rows as f32 / 2.0);
    if dx.hypot(dy) > 0.86 {
        return 0.0;
    }
    let wave = 0.5 + 0.5 * (t * TAU * 0.8 - index as f32 * 1.1).sin();
    TRACK.max(wave.powf(1.6) * 1.05)
}

fn pulse(columns: u32, rows: u32, x: u32, y: u32, t: f32) -> f32 {
    let dx = (x as f32 + 0.5 - columns as f32 / 2.0) / (columns as f32 / 2.0);
    let dy = (y as f32 + 0.5 - rows as f32 / 2.0) / (rows as f32 / 2.0);
    let radius = dx.hypot(dy);
    if radius > 1.0 {
        return 0.0;
    }
    let beat = 0.5 - 0.5 * (t * TAU * 0.6).cos();
    if radius < 0.18 + 0.08 * beat {
        return 1.0;
    }
    let band = 1.6 / columns as f32;
    (0..3)
        .filter_map(|ring| {
            let front = 0.25 + 0.75 * fract(t * 0.6 + ring as f32 / 3.0);
            ((radius - front).abs() < band).then(|| (1.0 - front).powf(0.6) * 1.3)
        })
        .fold(0.0, f32::max)
}

fn bar(columns: u32, x: u32, t: f32) -> f32 {
    let u = (x as f32 + 0.5) / columns as f32;
    let head = 1.45 * ease_in_out(fract(t * 0.55));
    let behind = head - u;
    if !(0.0..=0.45).contains(&behind) {
        return TRACK;
    }
    TRACK.max((1.0 - behind / 0.45).powf(1.1))
}

pub fn loader(variant: Variant, height: f32, cell: f32, time: f32, matrix: Matrix, color: Color) -> Image {
    let (columns, rows) = grid(variant, height, cell.max(1.0));
    let (ink, clear) = (pixel(color), Rgba8Pixel::new(0, 0, 0, 0));
    paint(columns, rows, |x, y| {
        let tone = match variant {
            Variant::Ring => ring(columns, rows, x, y, time),
            Variant::Dots => dots(columns, rows, x, y, time),
            Variant::Pulse => pulse(columns, rows, x, y, time),
            Variant::Bar => bar(columns, x, time),
        };
        if matrix.is_inked(x as i32, y as i32, tone) { ink } else { clear }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inked(variant: Variant, time: f32) -> Vec<bool> {
        let image = loader(variant, 40.0, 2.0, time, Matrix::Bayer4, Color::from_rgb_u8(255, 0, 0));
        image.to_rgba8().unwrap().as_slice().iter().map(|p| p.a > 0).collect()
    }

    #[test]
    fn the_grid_follows_the_size_and_the_shape() {
        assert_eq!(grid(Variant::Ring, 40.0, 2.0), (20, 20));
        assert_eq!(grid(Variant::Dots, 40.0, 2.0), (40, 12));
        assert_eq!(grid(Variant::Bar, 40.0, 2.0), (60, 4));
        assert_eq!(grid(Variant::Ring, 24.0, 8.0).0, 6);
    }

    #[test]
    fn every_variant_draws_something_and_keeps_to_its_shape() {
        for variant in Variant::ALL {
            assert!(inked(variant, 0.4).iter().any(|&on| on), "{variant:?}");
        }
        let ring = inked(Variant::Ring, 0.4);
        assert!(!ring[0] && !ring[19] && !ring[ring.len() - 1]);
    }

    #[test]
    fn the_animation_changes_over_time_and_repeats() {
        for variant in Variant::ALL {
            assert_ne!(inked(variant, 0.0), inked(variant, 0.7), "{variant:?}");
        }
        assert_eq!(inked(Variant::Bar, 0.3), inked(Variant::Bar, 0.3 + 1.0 / 0.55));
    }
}
