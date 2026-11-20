use std::f32::consts::PI;

use glam::Vec2;

use super::{mix, param, smoothstep};
use crate::field::{Cell, Field};
use crate::noise::snoise;

pub struct Waves;

fn height(x: f32, i: f32, k: f32, t: f32) -> f32 {
    let freq = mix(7.0, 2.2, k);
    let roll = (x * freq + t * (0.32 + 0.22 * k) + i * 1.9).sin();
    let chop = snoise(Vec2::new(x * freq * 0.45 + i * 5.0, t * 0.09 + i));
    roll * 0.62 + chop * 0.38
}

fn ocean(cell: &Cell, horizon: f32, layers: f32, amplitude: f32) -> f32 {
    let (uv, t) = (cell.uv, cell.time);
    let x = uv.x * cell.resolution.x / cell.resolution.y;
    let above = uv.y - horizon;
    let mut tone = if above > 0.0 { 0.5 * (-above * 13.0).exp() } else { 0.0 };
    let glint = (-((uv.x - 0.5) * 3.2).powi(2)).exp();

    for i in (0..8).map(|i| i as f32).filter(|&i| i < layers) {
        let k = i / (layers - 1.0).max(1.0);
        let base = horizon * (1.0 - i / layers).powf(1.45);
        let crest = base + amplitude * mix(0.006, 0.075, k) * height(x, i, k, t);
        let depth = (crest - uv.y) * cell.resolution.y;
        if depth > 0.0 {
            let body = mix(0.2, 0.03, k) + 0.16 * glint * (1.0 - k);
            tone = body * (0.4 + 0.6 * (-depth * mix(0.5, 0.06, k)).exp());
            if depth < 1.6 {
                tone = mix(0.95, 0.55, k) * (0.75 + 0.25 * glint);
            }
        }
    }
    tone
}

fn lines(cell: &Cell, layers: f32, amplitude: f32) -> f32 {
    let (uv, t) = (cell.uv, cell.time);
    let x = uv.x * cell.resolution.x / cell.resolution.y;
    let envelope = (PI * uv.x.clamp(0.0, 1.0)).sin().powi(2);

    let mut tone = 0.0_f32;
    for i in (0..8).map(|i| i as f32).filter(|&i| i < layers) {
        let k = i / (layers - 1.0).max(1.0);
        let y = 0.5 + envelope * amplitude * mix(0.3, 0.08, k) * height(x * 0.8, i, k, t * 1.4);
        let distance = (uv.y - y).abs() * cell.resolution.y;
        let line = 1.0 - smoothstep(0.4, 1.4, distance);
        let glow = (-distance * 0.18).exp() * 0.22;
        tone = tone.max((line * mix(1.0, 0.45, k)).max(glow * (1.0 - k * 0.6)));
    }
    tone
}

impl Field for Waves {
    // params: mode (0 ocean, 1 lines), horizon, layers, amplitude
    fn tone(&self, cell: &Cell) -> f32 {
        let horizon = param(cell.params, 1);
        let layers = param(cell.params, 2).round().clamp(2.0, 8.0);
        let amplitude = param(cell.params, 3).max(0.0);
        if param(cell.params, 0) < 0.5 {
            ocean(cell, horizon, layers, amplitude)
        } else {
            lines(cell, layers, amplitude)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Palette;

    fn column(mode: f32, time: f32) -> Vec<f32> {
        let palette = Palette::new(&[]);
        let params = [mode, 0.62, 6.0, 1.0];
        (0..100)
            .map(|i| {
                let uv = Vec2::new(0.5, i as f32 / 100.0);
                let cell = Cell { uv, resolution: Vec2::new(200.0, 100.0), time, params: &params, palette: &palette };
                Waves.tone(&cell)
            })
            .collect()
    }

    #[test]
    fn the_ocean_glows_above_the_horizon_and_has_swells_below() {
        let tones = column(0.0, 1.0);
        assert!(tones[80] > 0.0, "sky glow");
        assert!(tones[10] > 0.0, "swells");
        assert!(tones.iter().all(|t| (0.0..=1.0).contains(t)));
    }

    #[test]
    fn the_lines_stay_around_the_middle_and_move() {
        let tones = column(1.0, 1.0);
        assert!(tones[40..60].iter().any(|&t| t > 0.1));
        assert!(tones[2] < 0.01);
        assert_ne!(tones, column(1.0, 5.0));
    }
}
