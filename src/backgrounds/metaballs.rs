use glam::Vec2;

use super::{mix, param, smoothstep};
use crate::field::{Cell, Field};

pub struct Metaballs;

/// Sum of soft falloffs: 1 is the edge of a blob, more is inside.
fn sum(p: Vec2, aspect: f32, unit: f32, count: usize, size: f32, t: f32) -> f32 {
    (0..count)
        .map(|i| {
            let i = i as f32;
            let seed = i * 1.618;
            let centre = Vec2::new(
                aspect * (0.5 + 0.3 * (t * (0.05 + 0.017 * i) + seed * 2.3).sin()),
                0.5 + 0.34 * (t * (0.07 + 0.013 * i) + seed * 3.1).sin(),
            );
            let wobble = (seed * 7.3).fract();
            let radius = unit * size * (0.14 + 0.06 * wobble);
            let d = p - centre;
            radius * radius / d.length_squared().max(1e-5)
        })
        .sum()
}

impl Field for Metaballs {
    // params: rings (0 for solid blobs, 1 for contour lines), count, size
    fn tone(&self, cell: &Cell) -> f32 {
        let [rings, count, size] = std::array::from_fn(|index| param(cell.params, index));
        let rings = rings > 0.5;
        let count = count.round().clamp(2.0, 8.0) as usize;
        let size = size.max(0.1);

        let aspect = cell.resolution.x / cell.resolution.y;
        let p = Vec2::new(cell.uv.x * aspect, cell.uv.y);
        // In a tall box the blobs are sized by the width, so they still fit.
        let unit = aspect.min(1.0);
        let t = cell.time;
        let f = sum(p, aspect, unit, count, size, t);

        if rings {
            let step = 1.0 / cell.resolution.y;
            let h = f.ln();
            let hx = sum(p + Vec2::new(step, 0.0), aspect, unit, count, size, t).ln();
            let hy = sum(p + Vec2::new(0.0, step), aspect, unit, count, size, t).ln();
            let lines = 3.2;
            let v = h * lines + t * 0.12;
            let offset = ((v + 0.5).fract() - 0.5).abs();
            let per_cell = (Vec2::new(hx - h, hy - h).length() * lines).max(1e-4);
            let line = 1.0 - smoothstep(0.3, 1.3, offset / per_cell);
            let inside = smoothstep(0.85, 1.15, f);
            let fade = smoothstep(0.08, 0.6, f);
            return (line * mix(0.45, 1.0, inside) * fade).max(inside * 0.14);
        }

        let inside = smoothstep(0.92, 1.08, f);
        let core = mix(0.55, 0.95, smoothstep(1.0, 3.5, f));
        let halo = 0.24 * smoothstep(0.5, 1.0, f);
        mix(halo, core, inside)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Palette;

    fn tones(params: &[f32], time: f32) -> Vec<f32> {
        let palette = Palette::new(&[]);
        (0..120 * 80)
            .map(|index| {
                let uv = Vec2::new(((index % 120) as f32 + 0.5) / 120.0, ((index / 120) as f32 + 0.5) / 80.0);
                Metaballs.tone(&Cell { uv, resolution: Vec2::new(120.0, 80.0), time, params, palette: &palette })
            })
            .collect()
    }

    #[test]
    fn solid_blobs_have_dense_cores_and_empty_space() {
        let tones = tones(&[0.0, 5.0, 1.0], 3.0);
        assert!(tones.iter().any(|&t| t > 0.5));
        assert!(tones.iter().any(|&t| t < 0.05));
    }

    #[test]
    fn rings_draw_lines_instead_of_filled_blobs() {
        let solid = tones(&[0.0, 5.0, 1.0], 3.0);
        let rings = tones(&[1.0, 5.0, 1.0], 3.0);
        assert_ne!(solid, rings);
        assert!(rings.iter().any(|&t| t > 0.5));
    }

    #[test]
    fn more_blobs_cover_more_of_the_box() {
        let covered = |count: f32| tones(&[0.0, count, 1.0], 3.0).iter().filter(|&&t| t > 0.5).count();
        assert!(covered(8.0) > covered(2.0));
    }

    #[test]
    fn the_blobs_drift() {
        assert_ne!(tones(&[0.0, 5.0, 1.0], 0.0), tones(&[0.0, 5.0, 1.0], 20.0));
    }
}
