use std::f32::consts::TAU;

use glam::Vec2;

use super::{param, smoothstep};
use crate::field::{Cell, Field};
use crate::noise::snoise;

pub struct Ripples;

fn ring(p: Vec2, at: Vec2, aspect: f32, t: f32, phase: f32, rings: f32, decay: f32) -> f32 {
    let origin = Vec2::new(at.x * aspect, 1.0 - at.y);
    let d = p.distance(origin);
    let wave = 0.5 + 0.5 * (d * rings * TAU - t * 0.8 + phase).cos();
    let envelope = (-d * decay * 3.4).exp() * smoothstep(0.0, 0.05, d);
    smoothstep(0.72, 0.92, wave) * envelope
}

impl Field for Ripples {
    // params: sources (1 to 4), rings, decay, intensity, then x and y for each source.
    fn tone(&self, cell: &Cell) -> f32 {
        let params = cell.params;
        let sources = param(params, 0).round().clamp(1.0, 4.0) as usize;
        let rings = param(params, 1).max(1.0);
        let decay = param(params, 2).max(0.0);
        let intensity = param(params, 3).clamp(0.0, 1.0);
        let source = |i: usize| Vec2::new(param(params, 4 + 2 * i), param(params, 5 + 2 * i));

        let aspect = cell.resolution.x / cell.resolution.y;
        let t = cell.time;
        let mut p = Vec2::new(cell.uv.x * aspect, cell.uv.y);
        let swell = Vec2::new(snoise(p * 1.4 + Vec2::new(t * 0.03, 0.0)), snoise(p * 1.4 + Vec2::new(5.2, t * 0.025)));
        p += 0.01 * swell;

        let phases = [0.0, 2.1, 4.2, 1.3];
        let sum: f32 = (0..sources).map(|i| ring(p, source(i), aspect, t, phases[i], rings, decay)).sum();
        intensity * (sum * 0.9).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Palette;

    fn tone(params: &[f32], uv: Vec2, time: f32) -> f32 {
        let palette = Palette::new(&[]);
        let cell = Cell { uv, resolution: Vec2::new(200.0, 100.0), time, params, palette: &palette };
        Ripples.tone(&cell)
    }

    const ONE_SOURCE: [f32; 6] = [1.0, 12.0, 1.0, 1.0, 0.5, 0.5];

    #[test]
    fn rings_appear_and_move() {
        let at = |time: f32| -> Vec<f32> {
            (0..50).map(|i| tone(&ONE_SOURCE, Vec2::new(0.5 + i as f32 * 0.01, 0.5), time)).collect()
        };
        assert!(at(0.0).iter().any(|&t| t > 0.1));
        assert_ne!(at(0.0), at(3.0));
    }

    #[test]
    fn zero_intensity_draws_nothing() {
        let params = [1.0, 12.0, 1.0, 0.0, 0.5, 0.5];
        assert_eq!(tone(&params, Vec2::new(0.6, 0.5), 2.0), 0.0);
    }

    #[test]
    fn a_second_source_adds_rings() {
        let mut two = [0.0; 8];
        two[..6].copy_from_slice(&ONE_SOURCE);
        two[0] = 2.0;
        two[6] = 0.9;
        two[7] = 0.5;
        let sum = |params: &[f32]| -> f32 {
            (0..100).map(|i| tone(params, Vec2::new(0.8 + i as f32 * 0.002, 0.5), 1.0)).sum()
        };
        assert!(sum(&two) > sum(&ONE_SOURCE));
    }
}
