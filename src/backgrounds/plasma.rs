use std::f32::consts::FRAC_PI_4;

use glam::Vec2;

use super::{param, smoothstep};
use crate::field::{Cell, Field};
use crate::noise::fbm;

pub struct Plasma;

impl Field for Plasma {
    // params: scale, warp, bands, intensity
    fn tone(&self, cell: &Cell) -> f32 {
        let [scale, warp, bands, intensity] = std::array::from_fn(|index| param(cell.params, index));
        let scale = scale.max(0.05);
        let warp = warp.max(0.0);
        let bands = bands.max(0.1);
        let intensity = intensity.clamp(0.0, 1.0);

        let t = cell.time;
        let aspect = cell.resolution.x / cell.resolution.y;
        let mut p = Vec2::new((cell.uv.x - 0.5) * aspect, cell.uv.y - 0.5) * 4.2 * scale;

        let bend = Vec2::new(
            fbm(p * 0.35 + Vec2::new(t * 0.05, 0.0)),
            fbm(p * 0.35 + Vec2::new(3.7, 1.9) - Vec2::new(0.0, t * 0.04)),
        ) - 0.5;
        p += bend * 3.0 * warp;

        let orbit = Vec2::new((t * 0.13).sin() * 1.6, (t * 0.11).cos() * 1.2);
        let v = (p.x * 0.9 + t * 0.31).sin()
            + (p.y * 1.1 - t * 0.23).sin()
            + ((p.x * 0.6 + p.y * 0.8) * 1.2 + t * 0.17).sin()
            + (p.distance(orbit) * 1.4 - t * 0.29).sin();

        let tone = 0.5 + 0.5 * (v * FRAC_PI_4 * bands).sin();
        smoothstep(0.05, 1.0, tone).powf(1.7) * intensity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Palette;

    fn tones(time: f32) -> Vec<f32> {
        let palette = Palette::new(&[]);
        let params = [1.0, 0.3, 1.5, 1.0];
        (0..400)
            .map(|i| {
                let uv = Vec2::new((i % 20) as f32 / 20.0, (i / 20) as f32 / 20.0);
                let cell = Cell { uv, resolution: Vec2::new(200.0, 100.0), time, params: &params, palette: &palette };
                Plasma.tone(&cell)
            })
            .collect()
    }

    #[test]
    fn plasma_spans_paper_to_ink_and_flows() {
        let now = tones(2.0);
        assert!(now.iter().all(|t| (0.0..=1.0).contains(t)));
        assert!(now.iter().any(|&t| t < 0.1) && now.iter().any(|&t| t > 0.5));
        assert_ne!(now, tones(8.0));
    }
}
