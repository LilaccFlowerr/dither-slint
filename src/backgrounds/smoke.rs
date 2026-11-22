use glam::Vec2;

use super::{param, smoothstep};
use crate::field::{Cell, Field};
use crate::noise::snoise;

pub struct Smoke;

fn density(p: Vec2, t: f32, wind: f32) -> f32 {
    let q = Vec2::new(p.x, p.y * 0.82) - Vec2::new(wind * t * 0.04, t * 0.045);
    let warp = Vec2::new(
        snoise(q * 0.45 + Vec2::new(0.0, t * 0.012)),
        snoise(q * 0.45 + Vec2::new(5.2, 1.3) - Vec2::new(t * 0.01, 0.0)),
    );
    let w = q + warp * 0.6;
    0.5 + 0.36 * snoise(w * 0.8) + 0.14 * snoise(w * 1.9 + 3.0)
}

impl Field for Smoke {
    // params: scale, height, wind, density
    fn tone(&self, cell: &Cell) -> f32 {
        let [scale, height, wind, amount] = std::array::from_fn(|index| param(cell.params, index));
        let scale = scale.max(0.1);
        let height = height.clamp(0.0, 1.0);
        let amount = amount.clamp(0.0, 1.5);

        let t = cell.time;
        let aspect = cell.resolution.x / cell.resolution.y;
        let p = Vec2::new((cell.uv.x - 0.5) * aspect, cell.uv.y) * 2.2 * scale;

        let d = density(p, t, wind);
        let lit = density(p + Vec2::new(-0.2, 0.3), t, wind);
        let light = (0.75 + (d - lit) * 1.6).clamp(0.35, 1.0);

        let rise = cell.uv.y / height.max(0.05);
        let mut body = smoothstep(0.3, 0.85, d - rise * 0.5 + 0.25);
        body *= 1.0 - smoothstep(0.6, 1.2, rise - (d - 0.5) * 0.9);
        (body * light * amount * 0.9).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Palette;

    fn rows(params: &[f32], time: f32) -> Vec<f32> {
        let palette = Palette::new(&[]);
        (0..20)
            .map(|row| {
                (0..20)
                    .map(|column| {
                        let uv = Vec2::new(column as f32 / 20.0, row as f32 / 20.0);
                        Smoke.tone(&Cell { uv, resolution: Vec2::new(200.0, 100.0), time, params, palette: &palette })
                    })
                    .sum::<f32>()
            })
            .collect()
    }

    #[test]
    fn the_smoke_is_thick_low_down_and_thins_out_upward() {
        let rows = rows(&[1.0, 0.8, 0.2, 1.0], 12.0);
        assert!(rows[2] > rows[18]);
        assert_ne!(rows, self::rows(&[1.0, 0.8, 0.2, 1.0], 40.0));
    }

    #[test]
    fn no_density_means_no_smoke() {
        assert!(rows(&[1.0, 0.8, 0.2, 0.0], 12.0).iter().all(|row| *row == 0.0));
    }
}
