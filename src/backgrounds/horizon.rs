use glam::Vec2;

use super::{mix, param, smoothstep};
use crate::field::{Cell, Field};
use crate::noise::hash21;

pub struct Horizon;

fn noise(p: Vec2) -> f32 {
    let (i, f) = (p.floor(), p - p.floor());
    let u = f * f * (3.0 - 2.0 * f);
    let top = mix(hash21(i), hash21(i + Vec2::X), u.x);
    let bottom = mix(hash21(i + Vec2::Y), hash21(i + Vec2::ONE), u.x);
    mix(top, bottom, u.y)
}

fn fbm(mut p: Vec2) -> f32 {
    let (mut sum, mut amplitude) = (0.0, 0.5);
    for _ in 0..3 {
        sum += amplitude * noise(p);
        p = p * 2.03 + Vec2::new(17.1, 9.2);
        amplitude *= 0.5;
    }
    sum / 0.875
}

impl Field for Horizon {
    // params: horizon, curve, glow, aurora, stars
    fn tone(&self, cell: &Cell) -> f32 {
        let height = param(cell.params, 0).clamp(0.0, 0.95);
        let curve = param(cell.params, 1).clamp(0.0, 1.0);
        let strength = param(cell.params, 2).clamp(0.0, 1.0);
        let aurora = param(cell.params, 3).clamp(0.0, 1.0);
        let stars = param(cell.params, 4).clamp(0.0, 3.0);

        let (t, uv) = (cell.time, cell.uv);
        let aspect = cell.resolution.x / cell.resolution.y;
        let world = Vec2::new(uv.x * aspect, uv.y);
        let p = Vec2::new((uv.x - 0.5) * aspect, uv.y);

        let radius = 0.6 * 8f32.powf(1.0 - curve);
        let e = p.distance(Vec2::new(0.0, height - radius)) - radius;
        let (above, below) = (e.max(0.0), (-e).max(0.0));

        let warp = Vec2::new(
            fbm(world * 1.3 + Vec2::new(t * 0.021, 0.0)),
            fbm(world * 1.3 + Vec2::new(5.2, 1.3) - Vec2::new(0.0, t * 0.017)),
        );
        let n = fbm(world * Vec2::new(1.1, 2.6) + 1.7 * warp + Vec2::new(t * 0.013, 0.0));
        let curtains = mix(0.7, fbm(Vec2::new(world.x * 5.0 + 2.0 * warp.x - t * 0.03, above * 6.0 - t * 0.01)), aurora);
        let lit = 0.35 + 0.9 * n * curtains + 0.25 * n;
        let glow = (1.15 * (-above * 11.0).exp() * lit + 0.3 * (-above * 5.0).exp() * curtains * curtains) * strength;
        let rim = (-below * 60.0).exp() * (0.85 + 0.4 * n) * mix(0.6, 1.0, strength);
        let tone = if e > 0.0 { glow } else { rim };

        let spot = (uv * cell.resolution).floor();
        let seed = hash21(Vec2::new(hash21(spot), hash21(Vec2::new(spot.y, spot.x) + 31.7)) * 97.0);
        let star = if seed >= 1.0 - 0.0012 * stars { smoothstep(0.08, 0.3, above) } else { 0.0 };
        tone.max(star)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Palette;

    fn column(params: &[f32]) -> Vec<f32> {
        let palette = Palette::new(&[]);
        (0..50)
            .map(|row| {
                let uv = Vec2::new(0.5, (row as f32 + 0.5) / 50.0);
                Horizon.tone(&Cell { uv, resolution: Vec2::new(200.0, 100.0), time: 14.0, params, palette: &palette })
            })
            .collect()
    }

    #[test]
    fn the_glow_hugs_the_horizon_over_a_dark_planet() {
        let tones = column(&[0.28, 0.6, 1.0, 1.0, 0.0]);
        let at = |height: f32| tones[(height * 50.0) as usize];
        assert!(at(0.3) > at(0.6), "brightest just above the horizon");
        assert!(at(0.3) > at(0.1), "the planet below is dark");
    }

    #[test]
    fn no_glow_leaves_only_the_rim() {
        let tones = column(&[0.28, 0.6, 0.0, 1.0, 0.0]);
        assert!(tones[30..].iter().all(|t| *t == 0.0));
    }

    #[test]
    fn stars_appear_only_away_from_the_glow() {
        let palette = Palette::new(&[]);
        let params = [0.28, 0.6, 0.0, 1.0, 3.0];
        let lit = (0..200 * 100)
            .filter(|index| {
                let uv = Vec2::new(((index % 200) as f32 + 0.5) / 200.0, ((index / 200) as f32 + 0.5) / 100.0);
                Horizon.tone(&Cell { uv, resolution: Vec2::new(200.0, 100.0), time: 14.0, params: &params, palette: &palette }) == 1.0 && uv.y > 0.6
            })
            .count();
        assert!(lit > 0);
    }
}
