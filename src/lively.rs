use slint::{Color, Image, Rgba8Pixel};

use crate::Matrix;
use crate::render::{paint, pixel};

fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn draw(columns: u32, rows: u32, color: Color, tone: impl Fn(f32, f32) -> f32) -> Image {
    let (columns, rows) = (columns.max(1), rows.max(1));
    let (ink, clear) = (pixel(color), Rgba8Pixel::new(0, 0, 0, 0));
    paint(columns, rows, |x, y| {
        let inked = Matrix::Bayer4.is_inked(x as i32, y as i32, tone(x as f32, y as f32));
        if inked { ink } else { clear }
    })
}

pub fn sweep(columns: u32, rows: u32, t: f32, strength: f32, color: Color) -> Image {
    let t = t.clamp(0.0, 1.0);
    let width = (columns as f32 * 0.18).max(6.0);
    let front = -width + t * (columns as f32 + rows as f32 + width * 2.0);
    let fade = (std::f32::consts::PI * t).sin();
    draw(columns, rows, color, |x, y| {
        let d = (x + y * 0.6 - front).abs() / width;
        if d < 1.0 { (1.0 - d * d) * strength * fade } else { 0.0 }
    })
}

pub fn ripple(columns: u32, rows: u32, cx: f32, cy: f32, t: f32, strength: f32, color: Color) -> Image {
    let t = t.clamp(0.0, 1.0);
    let far = cx.max(columns as f32 - cx).hypot(cy.max(rows as f32 - cy));
    let radius = far * ease_out(t);
    let band = 2.0 + 4.0 * (1.0 - t);
    let peak = strength * (1.0 - t);
    draw(columns, rows, color, |x, y| {
        let d = ((x - cx).hypot(y - cy) - radius).abs() / band;
        if d < 1.0 { (1.0 - d) * peak } else { 0.0 }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inked(image: &Image) -> usize {
        image.to_rgba8().unwrap().as_slice().iter().filter(|p| p.a > 0).count()
    }

    #[test]
    fn sweep_is_empty_at_both_ends() {
        let white = Color::from_rgb_u8(255, 255, 255);
        assert_eq!(inked(&sweep(40, 18, 0.0, 0.85, white)), 0);
        assert_eq!(inked(&sweep(40, 18, 1.0, 0.85, white)), 0);
        assert!(inked(&sweep(40, 18, 0.5, 0.85, white)) > 0);
    }

    #[test]
    fn sweep_moves_left_to_right() {
        let white = Color::from_rgb_u8(255, 255, 255);
        let centre = |t: f32| {
            let buffer = sweep(60, 10, t, 1.0, white).to_rgba8().unwrap();
            let xs: Vec<usize> = buffer.as_slice().iter().enumerate().filter(|(_, p)| p.a > 0).map(|(i, _)| i % 60).collect();
            xs.iter().sum::<usize>() as f32 / xs.len() as f32
        };
        assert!(centre(0.3) < centre(0.6));
    }

    #[test]
    fn ripple_fades_out() {
        let white = Color::from_rgb_u8(255, 255, 255);
        assert!(inked(&ripple(40, 18, 20.0, 9.0, 0.2, 0.85, white)) > 0);
        assert_eq!(inked(&ripple(40, 18, 20.0, 9.0, 1.0, 0.85, white)), 0);
    }
}
