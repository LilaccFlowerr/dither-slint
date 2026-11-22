use glam::{Vec2, Vec3};

use super::{param, smoothstep};
use crate::field::{Cell, Field};
use crate::noise::snoise;

pub struct Contours;

fn height(p: Vec2, t: f32) -> f32 {
    let mut q = p + Vec2::new(4.3 + t * 0.008, 2.9 + t * 0.003);
    q += 0.3 * Vec2::new(snoise(q * 0.45 + Vec2::new(1.7, t * 0.008)), snoise(q * 0.45 + Vec2::new(8.3, -t * 0.006)));
    0.5 + 0.34 * snoise(q * 0.85) + 0.14 * snoise(q * 1.9 + 4.1)
}

/// A contour line about `width` cells wide wherever the height crosses a whole step.
fn line(h: f32, hx: f32, hy: f32, count: f32, width: f32) -> f32 {
    let v = h * count;
    let offset = ((v + 0.5) - (v + 0.5).floor() - 0.5).abs();
    let per_cell = (Vec2::new(hx - h, hy - h).length() * count).max(1e-4);
    let d = offset / per_cell;
    (1.0 - smoothstep(width * 0.5 - 0.5, width * 0.5 + 0.5, d)) * (1.0 - smoothstep(0.08, 0.14, offset))
}

impl Field for Contours {
    // params: lines, scale, index (every how many lines one is heavier, 0 for none), relief
    fn tone(&self, cell: &Cell) -> f32 {
        let lines = param(cell.params, 0).clamp(4.0, 32.0);
        let scale = param(cell.params, 1).max(0.1);
        let index = param(cell.params, 2).round().max(0.0);
        let relief = param(cell.params, 3).clamp(0.0, 1.0);

        let t = cell.time;
        let unit = scale / 200.0;
        let p = (cell.uv - 0.5) * cell.resolution * unit;
        let h = height(p, t);
        let hx = height(p + Vec2::new(unit, 0.0), t);
        let hy = height(p + Vec2::new(0.0, unit), t);

        let mut tone = line(h, hx, hy, lines, 1.2) * 0.5;
        if index > 0.5 {
            let nearest = (h * lines + 0.5).floor();
            if nearest.rem_euclid(index) < 0.5 {
                tone = line(h, hx, hy, lines, 2.0);
            }
        }
        let grad = Vec2::new(hx - h, hy - h) / unit;
        let normal = Vec3::new(-grad.x * 0.9, -grad.y * 0.9, 1.0).normalize();
        let lit = normal.dot(Vec3::new(-0.6, 0.6, 0.6).normalize());
        tone.max(relief * 0.28 * smoothstep(0.6, 0.15, lit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Palette;

    fn tones(params: &[f32], time: f32) -> Vec<f32> {
        let palette = Palette::new(&[]);
        (0..60 * 40)
            .map(|index| {
                let uv = Vec2::new(((index % 60) as f32 + 0.5) / 60.0, ((index / 60) as f32 + 0.5) / 40.0);
                Contours.tone(&Cell { uv, resolution: Vec2::new(240.0, 160.0), time, params, palette: &palette })
            })
            .collect()
    }

    #[test]
    fn lines_are_drawn_in_the_middle_colour_and_index_lines_in_ink() {
        let tones = tones(&[14.0, 1.0, 5.0, 0.0], 6.0);
        assert!(tones.iter().any(|t| (t - 0.5).abs() < 0.05), "regular lines");
        assert!(tones.iter().any(|&t| t > 0.9), "index lines");
        assert!(tones.iter().filter(|&&t| t == 0.0).count() > tones.len() / 2, "mostly paper between the lines");
    }

    #[test]
    fn without_an_index_every_line_is_the_same() {
        assert!(tones(&[14.0, 1.0, 0.0, 0.0], 6.0).iter().all(|&t| t <= 0.5 + 1e-6));
    }

    #[test]
    fn the_hills_drift() {
        assert_ne!(tones(&[14.0, 1.0, 5.0, 0.5], 6.0), tones(&[14.0, 1.0, 5.0, 0.5], 60.0));
    }
}
