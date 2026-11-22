use glam::{Vec2, Vec3};

use super::{mix, param, smoothstep};
use crate::field::{Cell, Field};
use crate::noise::snoise;

pub struct Silk;

/// The cloth: a height that rises and falls across the folds, bent by slow waves.
fn height(p: Vec2, t: f32, folds: f32) -> f32 {
    let bend = 0.55 * (p.x * 0.9 + t * 0.09).sin()
        + 0.3 * (p.x * 1.7 - p.y * 0.4 + t * 0.06 + 1.3).sin()
        + 0.35 * snoise(p * 0.35 + Vec2::new(t * 0.015, 0.0));
    let mut across = (p.y + bend) * folds * 0.9;
    across += 0.6 * (across * 0.5 + p.x * 0.3 + t * 0.04).sin();
    (across + 0.4 * (p.x * 0.7 - t * 0.05).sin()).sin()
}

impl Field for Silk {
    // params: folds, angle in degrees as in CSS, sheen, intensity
    fn tone(&self, cell: &Cell) -> f32 {
        let folds = param(cell.params, 0).max(0.5);
        let angle = (param(cell.params, 1) - 90.0).to_radians();
        let sheen = param(cell.params, 2).clamp(0.0, 1.0);
        let intensity = param(cell.params, 3).clamp(0.0, 1.0);

        let t = cell.time;
        let aspect = cell.resolution.x / cell.resolution.y;
        let c = Vec2::new((cell.uv.x - 0.5) * aspect, cell.uv.y - 0.5) * 3.2;
        let (sin, cos) = angle.sin_cos();
        let p = Vec2::new(cos * c.x - sin * c.y, sin * c.x + cos * c.y);

        let e = 3.2 / cell.resolution.y;
        let h = height(p, t, folds);
        let hx = height(p + Vec2::new(e, 0.0), t, folds);
        let hy = height(p + Vec2::new(0.0, e), t, folds);
        let grad = Vec2::new(hx - h, hy - h) / e / folds;
        let normal = Vec3::new(-grad.x * 1.4, -grad.y * 1.4, 1.0).normalize();

        let light = Vec3::new(-0.45, 0.65, 0.6).normalize();
        let diffuse = 0.5 + 0.5 * normal.dot(light);
        let halfway = (light + Vec3::Z).normalize();
        let shine = normal.dot(halfway).max(0.0).powf(mix(6.0, 48.0, sheen));

        let pool = smoothstep(-0.6, 0.9, snoise(c * 0.3 + Vec2::new(t * 0.012, 0.4)) * 0.6 + 0.3 * c.y);
        let tone = smoothstep(0.25, 1.0, diffuse).powi(2) * 0.72 + shine * mix(0.45, 0.85, sheen);
        (tone * mix(0.55, 1.0, pool)).clamp(0.0, 1.0) * intensity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Palette;

    fn tones(params: &[f32], time: f32) -> Vec<f32> {
        let palette = Palette::new(&[]);
        (0..400)
            .map(|i| {
                let uv = Vec2::new((i % 20) as f32 / 20.0, (i / 20) as f32 / 20.0);
                Silk.tone(&Cell { uv, resolution: Vec2::new(200.0, 100.0), time, params, palette: &palette })
            })
            .collect()
    }

    #[test]
    fn the_folds_catch_the_light_and_shift() {
        let now = tones(&[6.0, 110.0, 0.6, 1.0], 7.0);
        assert!(now.iter().all(|t| (0.0..=1.0).contains(t)));
        assert!(now.iter().any(|&t| t < 0.2) && now.iter().any(|&t| t > 0.6));
        assert_ne!(now, tones(&[6.0, 110.0, 0.6, 1.0], 30.0));
    }

    #[test]
    fn the_intensity_caps_the_brightest_tone() {
        assert!(tones(&[6.0, 110.0, 0.6, 0.4], 7.0).iter().all(|&t| t <= 0.4));
    }
}
