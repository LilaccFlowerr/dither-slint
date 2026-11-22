use glam::Vec2;

use super::{param, smoothstep};
use crate::field::{Cell, Field};
use crate::noise::{fbm_octaves, rotate, snoise};

pub struct NoiseField;

fn ridges(mut p: Vec2, octaves: usize) -> f32 {
    let (mut sum, mut amplitude, mut norm) = (0.0, 0.5, 0.0);
    for _ in 0..octaves {
        let r = 1.0 - snoise(p).abs();
        sum += amplitude * r * r;
        norm += amplitude;
        p = rotate(p);
        amplitude *= 0.5;
    }
    sum / norm
}

#[derive(Clone, Copy)]
enum Variant {
    Smoke,
    Ridges,
    Contours,
    Marble,
}

struct Settings {
    angle: f32,
    bands: f32,
    contrast: f32,
    coverage: f32,
    detail: usize,
    flow: f32,
    intensity: f32,
    invert: bool,
    mask: usize,
    mask_size: f32,
    scale: f32,
    variant: Variant,
    warp: f32,
}

impl Settings {
    fn from(params: &[f32]) -> Self {
        let [angle, bands, contrast, coverage, detail, flow, intensity, invert, mask, mask_size, scale, variant, warp] = std::array::from_fn(|index| param(params, index));
        Self {
            angle,
            bands: bands.clamp(1.0, 48.0),
            contrast: contrast.max(0.0),
            coverage: coverage.clamp(-1.0, 1.0),
            detail: detail.round().clamp(1.0, 6.0) as usize,
            flow: flow.max(0.0),
            intensity: intensity.clamp(0.0, 1.0),
            invert: invert > 0.5,
            mask: mask.round() as usize,
            mask_size: mask_size.clamp(0.0, 1.0),
            scale: scale.max(0.1),
            variant: match variant.round() as usize {
                1 => Variant::Ridges,
                2 => Variant::Contours,
                3 => Variant::Marble,
                _ => Variant::Smoke,
            },
            warp: warp.max(0.0),
        }
    }
}

fn keep(mask: usize, size: f32, uv: Vec2) -> f32 {
    let oval = Vec2::new((uv.x - 0.5) * 0.85, (uv.y - 0.5) * 1.7);
    match mask {
        1 => smoothstep(0.36 * size, 0.36 * size + 0.44, oval.length()),
        2 => smoothstep(size * 0.8, size * 0.8 + 0.55, 1.0 - uv.y),
        3 => smoothstep(size * 0.8, size * 0.8 + 0.55, uv.y),
        4 => 1.0 - smoothstep(0.2 + 0.45 * (1.0 - size), 0.2 + 0.45 * (1.0 - size) + 0.4, oval.length()),
        _ => 1.0,
    }
}

impl Field for NoiseField {
    // params: angle (radians), bands, contrast, coverage, detail, flow, intensity,
    // invert, mask, mask size, scale, variant, warp
    fn tone(&self, cell: &Cell) -> f32 {
        let s = Settings::from(cell.params);
        let (uv, t) = (cell.uv, cell.time);
        let aspect = cell.resolution.x / cell.resolution.y;

        let p = Vec2::new(uv.x * aspect, uv.y) * s.scale;
        let drift = Vec2::new(s.angle.cos(), s.angle.sin()) * t * 0.05 * s.flow;
        let q = Vec2::new(
            fbm_octaves(p + Vec2::new(0.0, t * 0.02) - drift, s.detail),
            fbm_octaves(p + Vec2::new(5.2, 1.3) - Vec2::new(t * 0.015, 0.0) - drift * 0.8, s.detail),
        );
        let w = p + s.warp * q - drift * 0.6;

        let mut n = match s.variant {
            Variant::Marble => {
                let stripe = (w.dot(Vec2::new(0.35, 1.0)) * s.bands * 2.4 + fbm_octaves(w * 0.7, s.detail) * 2.5).sin();
                (0.5 + 0.5 * stripe).powf(1.6)
            }
            Variant::Contours => {
                let h = fbm_octaves(p * 0.6 + s.warp * 0.3 * q - drift * 0.6, s.detail.min(2));
                let d = ((h * s.bands).fract() - 0.5).abs();
                smoothstep(0.38, 0.47, d).max(h * h * 0.2)
            }
            Variant::Ridges => ridges(w, s.detail).powf(1.4),
            Variant::Smoke => fbm_octaves(w, s.detail),
        };

        n = match s.variant {
            Variant::Contours => (n + s.coverage).clamp(0.0, 1.0),
            variant => {
                let slope = if matches!(variant, Variant::Smoke) { 1.0 } else { 1.5 };
                let n = ((n - 0.5 + s.coverage) * s.contrast * slope + 0.5).clamp(0.0, 1.0);
                n * n * (3.0 - 2.0 * n)
            }
        };
        if matches!(s.variant, Variant::Smoke) {
            n = n.powf(2.8);
        }
        if s.invert {
            n = 1.0 - n;
        }
        n * keep(s.mask, s.mask_size, uv) * s.intensity
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_PI_2;

    use super::*;
    use crate::field::Palette;

    fn tones(variant: f32, mask: f32) -> Vec<f32> {
        let palette = Palette::new(&[]);
        let params = [FRAC_PI_2, 8.0, 1.2, 0.0, 4.0, 1.0, 0.9, 0.0, mask, 0.5, 1.3, variant, 1.7];
        (0..400)
            .map(|i| {
                let uv = Vec2::new((i % 20) as f32 / 20.0, (i / 20) as f32 / 20.0);
                let cell = Cell { uv, resolution: Vec2::new(200.0, 100.0), time: 3.0, params: &params, palette: &palette };
                NoiseField.tone(&cell)
            })
            .collect()
    }

    #[test]
    fn every_variant_draws_something_in_range() {
        for variant in [0.0, 1.0, 2.0, 3.0] {
            let tones = tones(variant, 0.0);
            assert!(tones.iter().all(|t| (0.0..=1.0).contains(t)), "variant {variant}");
            assert!(tones.iter().any(|&t| t > 0.1), "variant {variant}");
        }
    }

    #[test]
    fn the_mask_clears_the_centre() {
        let plain = tones(1.0, 0.0);
        let masked = tones(1.0, 1.0);
        let centre = |tones: &[f32]| tones[190..210].iter().sum::<f32>();
        assert!(centre(&masked) < centre(&plain));
    }
}
