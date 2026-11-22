use glam::Vec2;

use super::{mix, param, smoothstep};
use crate::field::{Cell, Field};
use crate::noise::{fbm, hash21};

pub struct Starfield;

struct Layer {
    grid: f32,
    offset: Vec2,
    density: f32,
    peak: f32,
    seed: f32,
}

/// One layer of stars on a grid of `grid` cells, at most one star per square, so they never clump.
fn layer(cell: Vec2, layer: &Layer, t: f32, twinkle: f32, steps: f32) -> f32 {
    let Layer { grid, offset, density, peak, seed } = *layer;
    let q = cell + offset;
    let id = (q / grid).floor();
    let local = (q - id * grid).floor();
    if hash21(id + seed) > density {
        return 0.0;
    }
    let at = (Vec2::new(hash21(id + seed + 7.1), hash21(id + seed + 13.7)) * (grid - 2.0)).floor() + 1.0;
    let d = (local - at).abs();
    let rate = 0.5 + 1.7 * hash21(id + seed + 5.3);
    let pulse = 0.5 + 0.5 * (t * rate + hash21(id + seed + 2.9) * 40.0).sin();
    let brightness = peak * (0.45 + 0.55 * hash21(id + seed + 3.3)) * mix(1.0, pulse, twinkle);
    let core = if d.x + d.y <= 0.5 { brightness } else { 0.0 };
    let arms = if d.x + d.y <= 1.5 && brightness >= 0.82 { brightness - 1.0 / steps } else { 0.0 };
    core.max(arms)
}

impl Field for Starfield {
    // params: density, twinkle, angle in degrees as in CSS, drift in cells per second, nebula,
    // steps (palette colours after the paper)
    fn tone(&self, cell: &Cell) -> f32 {
        let [density, twinkle, angle, drift, nebula, steps] = std::array::from_fn(|index| param(cell.params, index));
        let density = density.clamp(0.0, 2.0);
        let twinkle = twinkle.clamp(0.0, 1.0);
        let angle = angle.to_radians();
        let nebula = nebula.clamp(0.0, 1.0);
        let steps = steps.max(1.0);

        let t = cell.time;
        let position = cell.uv * cell.resolution;
        let travel = -Vec2::new(angle.sin(), angle.cos()) * t * drift;
        let layers = [
            Layer { grid: 7.0, offset: travel * 0.3, density: 0.32 * density, peak: 0.55, seed: 0.0 },
            Layer { grid: 13.0, offset: travel * 0.6, density: 0.42 * density, peak: 0.85, seed: 41.0 },
            Layer { grid: 23.0, offset: travel, density: 0.5 * density, peak: 1.0, seed: 83.0 },
        ];
        let stars = layers.iter().map(|each| layer(position, each, t, twinkle, steps)).fold(0.0, f32::max);
        // Snap to whole palette steps, so a star is one solid colour and never flickers between thresholds.
        let stars = if stars > 0.12 { ((stars * steps + 0.35).floor() / steps).max(1.0 / steps) } else { 0.0 };

        let aspect = cell.resolution.x / cell.resolution.y;
        let p = Vec2::new(cell.uv.x * aspect, cell.uv.y) + travel * 0.15 / cell.resolution.y;
        let haze = smoothstep(0.52, 0.95, fbm(p * 1.4 + Vec2::new(2.3, 7.1)));
        stars.max(haze * 0.22 * nebula)
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
                Starfield.tone(&Cell { uv, resolution: Vec2::new(120.0, 80.0), time, params, palette: &palette })
            })
            .collect()
    }

    #[test]
    fn stars_snap_to_whole_palette_steps() {
        let tones = tones(&[1.0, 0.6, 270.0, 1.6, 0.0, 2.0], 6.0);
        let stars: Vec<f32> = tones.into_iter().filter(|&t| t > 0.0).collect();
        assert!(!stars.is_empty());
        assert!(stars.iter().all(|t| (t * 2.0 - (t * 2.0).round()).abs() < 1e-5));
    }

    #[test]
    fn no_density_and_no_nebula_is_an_empty_sky() {
        assert!(tones(&[0.0, 0.6, 270.0, 1.6, 0.0, 2.0], 6.0).iter().all(|&t| t == 0.0));
    }

    #[test]
    fn the_stars_drift() {
        let params = [1.0, 0.0, 270.0, 1.6, 0.0, 2.0];
        assert_ne!(tones(&params, 6.0), tones(&params, 16.0));
    }
}
