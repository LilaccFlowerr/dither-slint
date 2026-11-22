use glam::Vec2;

use super::{mix, param, smoothstep};
use crate::field::{Cell, Field, Frame};
use crate::noise::{fbm, snoise};

const MOST_CURTAINS: usize = 6;

pub struct Aurora;

struct Settings {
    curtains: usize,
    height: f32,
    spread: f32,
    sway: f32,
    turbulence: f32,
    rays: f32,
    decay: f32,
    glow: f32,
    intensity: f32,
    shimmer: f32,
    scale: f32,
    tilt: (f32, f32),
    drift: f32,
    down: bool,
    mask: f32,
    time: f32,
}

impl Settings {
    // params: curtains, height, spread, sway, turbulence, rays, length, glow, intensity,
    // shimmer, scale, tilt in degrees, drift, down (1 hangs the rays from the top),
    // mask (0 none, 1 top, 2 bottom, 3 edges)
    fn new(params: &[f32], time: f32) -> Self {
        Self {
            curtains: param(params, 0).round().clamp(1.0, MOST_CURTAINS as f32) as usize,
            height: param(params, 1),
            spread: param(params, 2).clamp(0.0, 0.5),
            sway: param(params, 3).max(0.0),
            turbulence: param(params, 4).max(0.0),
            rays: param(params, 5).clamp(1.0, 32.0),
            decay: mix(15.0, 2.2, param(params, 6).clamp(0.0, 1.0)),
            glow: param(params, 7).clamp(0.0, 2.0),
            intensity: param(params, 8).clamp(0.0, 1.0),
            shimmer: param(params, 9).clamp(0.0, 1.0),
            scale: param(params, 10).max(0.1),
            tilt: param(params, 11).clamp(-45.0, 45.0).to_radians().sin_cos(),
            drift: param(params, 12).clamp(-2.0, 2.0),
            down: param(params, 13) > 0.5,
            mask: param(params, 14),
            time,
        }
    }

    /// The point in the tilted sky that a cell shows.
    fn point(&self, uv: Vec2, aspect: f32) -> Vec2 {
        let mut p = Vec2::new((uv.x - 0.5) * aspect, uv.y - 0.5);
        if self.down {
            p.y = -p.y;
        }
        let (sin, cos) = self.tilt;
        Vec2::new(cos * p.x + sin * p.y, -sin * p.x + cos * p.y) + Vec2::new(0.0, 0.5)
    }
}

/// Everything about a column of the sky that does not depend on height:
/// for each curtain its edge, rays and patchiness, and the shimmer's noise.
#[derive(Clone, Copy, Default)]
struct Column {
    edge: [f32; MOST_CURTAINS],
    beams: [f32; MOST_CURTAINS],
    patch: [f32; MOST_CURTAINS],
    ripple: f32,
}

impl Column {
    fn at(settings: &Settings, across: f32) -> Self {
        let t = settings.time;
        let x = across * settings.scale + t * settings.drift * 0.04;
        let mut column = Column { ripple: snoise(Vec2::new(across * 2.5, t * 0.15)), ..Default::default() };
        for index in 0..settings.curtains {
            let i = index as f32;
            column.edge[index] = settings.height
                + i * settings.spread
                + settings.sway * 0.11 * snoise(Vec2::new(x * 0.55 + i * 3.1, t * 0.035 + i * 1.7))
                + settings.turbulence * 0.05 * snoise(Vec2::new(x * 2.1 - i * 1.3, t * 0.07 + i));
            let bend = snoise(Vec2::new(x * 0.8, t * 0.03 + i)) * (0.25 + 0.35 * settings.sway);
            let rx = (x + bend) * settings.rays;
            let beams = mix(0.2, 1.0, smoothstep(0.25, 0.75, fbm(Vec2::new(rx + i * 9.0, t * 0.045 + i))));
            column.beams[index] = beams * (0.75 + 0.25 * snoise(Vec2::new(rx * 3.2, i * 4.0 + t * 0.02)));
            column.patch[index] = mix(0.3, 1.0, smoothstep(0.25, 0.7, fbm(Vec2::new(x * 0.45 + i * 5.0, t * 0.02 + i * 2.0))));
        }
        column
    }

    fn lerp(&self, other: &Column, t: f32) -> Column {
        let blend = |a: &[f32; MOST_CURTAINS], b: &[f32; MOST_CURTAINS]| std::array::from_fn(|index| mix(a[index], b[index], t));
        Column {
            edge: blend(&self.edge, &other.edge),
            beams: blend(&self.beams, &other.beams),
            patch: blend(&self.patch, &other.patch),
            ripple: mix(self.ripple, other.ripple, t),
        }
    }
}

fn shade(settings: &Settings, column: &Column, p: Vec2, uv: Vec2) -> f32 {
    let t = settings.time;
    let curtains = settings.curtains as f32;
    let mut light = 0.0;
    for index in 0..settings.curtains {
        let i = index as f32;
        let depth = i / (curtains - 1.0).max(1.0);
        let above = p.y - column.edge[index];
        let curtain = if above > 0.0 { (-above * settings.decay * (1.0 + i * 0.4)).exp() } else { (above * 48.0).exp() };
        let rim = (-above.abs() * 34.0).exp() * settings.glow;
        let haze = if above > 0.0 { (-above * 5.0).exp() * 0.09 } else { (above * 14.0).exp() * 0.05 };
        let beams = column.beams[index];
        light += (curtain * beams + rim * (0.5 + 0.5 * beams) + haze) * column.patch[index] * mix(1.0, 0.5, depth);
    }
    let ripple = 0.5 + 0.5 * (t * 0.8 + p.x * 4.0 + 2.0 * column.ripple).sin();
    let tone = (1.0 - (-light * 1.25).exp()) * settings.intensity * (1.0 - settings.shimmer * 0.45 * ripple);

    let keep = if settings.mask > 2.5 {
        smoothstep(0.0, 0.22, uv.x) * smoothstep(1.0, 0.78, uv.x)
    } else if settings.mask > 1.5 {
        smoothstep(0.02, 0.4, uv.y)
    } else if settings.mask > 0.5 {
        smoothstep(0.98, 0.6, uv.y)
    } else {
        1.0
    };
    tone * keep
}

impl Field for Aurora {
    fn tone(&self, cell: &Cell) -> f32 {
        let settings = Settings::new(cell.params, cell.time);
        let p = settings.point(cell.uv, cell.resolution.x / cell.resolution.y);
        shade(&settings, &Column::at(&settings, p.x), p, cell.uv)
    }

    fn prepare(&self, frame: &Frame) -> Option<Box<dyn Field + '_>> {
        Some(Box::new(Sky::new(frame)))
    }
}

const SAMPLES_PER_CELL: f32 = 4.0;

/// One frame of the aurora, with its columns worked out once, a few per cell.
/// Untilted, every fourth sample lies on the centre of a column of cells.
struct Sky {
    settings: Settings,
    first: f32,
    spacing: f32,
    columns: Vec<Column>,
}

impl Sky {
    fn new(frame: &Frame) -> Self {
        let settings = Settings::new(frame.params, frame.time);
        let aspect = frame.resolution.x / frame.resolution.y;
        let half = Vec2::new(0.5 / frame.resolution.x, 0.5 / frame.resolution.y);
        let corners = [half, Vec2::new(1.0 - half.x, half.y), Vec2::new(half.x, 1.0 - half.y), Vec2::ONE - half];
        let across = corners.map(|corner| settings.point(corner, aspect).x);
        let first = across.iter().copied().fold(f32::INFINITY, f32::min);
        let last = across.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let spacing = aspect / frame.resolution.x / SAMPLES_PER_CELL;
        let count = ((last - first) / spacing).round() as usize + 1;
        let columns = (0..count).map(|index| Column::at(&settings, first + index as f32 * spacing)).collect();
        Self { settings, first, spacing, columns }
    }

    fn column(&self, across: f32) -> Column {
        let at = ((across - self.first) / self.spacing).clamp(0.0, (self.columns.len() - 1) as f32);
        let index = (at.floor() as usize).min(self.columns.len() - 1);
        let next = (index + 1).min(self.columns.len() - 1);
        self.columns[index].lerp(&self.columns[next], at - index as f32)
    }
}

impl Field for Sky {
    fn tone(&self, cell: &Cell) -> f32 {
        let p = self.settings.point(cell.uv, cell.resolution.x / cell.resolution.y);
        shade(&self.settings, &self.column(p.x), p, cell.uv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Palette;

    const DEFAULTS: [f32; 15] = [3.0, 0.42, 0.11, 1.0, 0.6, 8.0, 0.45, 0.9, 0.95, 0.2, 1.0, 0.0, 0.0, 0.0, 0.0];
    const RESOLUTION: Vec2 = Vec2::new(240.0, 120.0);
    const DITHER_STEP: f32 = 1.0 / 16.0;

    fn column(params: &[f32], time: f32) -> Vec<f32> {
        let palette = Palette::new(&[]);
        (0..40)
            .map(|row| {
                let uv = Vec2::new(0.5, (row as f32 + 0.5) / 40.0);
                Aurora.tone(&Cell { uv, resolution: Vec2::new(200.0, 100.0), time, params, palette: &palette })
            })
            .collect()
    }

    /// The largest difference between the aurora worked out per cell and per frame.
    fn prepared_drift(params: &[f32]) -> f32 {
        let palette = Palette::new(&[]);
        let frame = Frame { resolution: RESOLUTION, time: 4.0, params };
        let sky = Aurora.prepare(&frame).unwrap();
        (0..(RESOLUTION.x * RESOLUTION.y) as usize)
            .map(|index| {
                let (x, y) = ((index % RESOLUTION.x as usize) as f32, (index / RESOLUTION.x as usize) as f32);
                let uv = Vec2::new((x + 0.5) / RESOLUTION.x, (y + 0.5) / RESOLUTION.y);
                let cell = Cell { uv, resolution: RESOLUTION, time: 4.0, params, palette: &palette };
                (sky.tone(&cell) - Aurora.tone(&cell)).abs()
            })
            .fold(0.0, f32::max)
    }

    #[test]
    fn the_curtains_glow_above_their_edge_and_fade_below() {
        let tones = column(&DEFAULTS, 4.0);
        assert!(tones.iter().all(|t| (0.0..=1.0).contains(t)));
        assert!(tones[2] < tones[20], "the bottom of the sky is darker than the curtains");
        assert_ne!(tones, column(&DEFAULTS, 20.0));
    }

    #[test]
    fn the_top_mask_clears_the_top() {
        let mut masked = DEFAULTS;
        masked[14] = 1.0;
        assert!(column(&masked, 4.0)[39] < 0.001);
    }

    #[test]
    fn down_hangs_the_curtains_from_the_top() {
        let mut down = DEFAULTS;
        down[13] = 1.0;
        let (up, mut down) = (column(&DEFAULTS, 4.0), column(&down, 4.0));
        down.reverse();
        assert!(up.iter().zip(&down).all(|(a, b)| (a - b).abs() < 1e-4), "down is up mirrored");
    }

    #[test]
    fn a_prepared_frame_draws_the_same_sky() {
        assert!(prepared_drift(&DEFAULTS) < 1e-4, "untilted, every cell is exact");
        let mut tilted = DEFAULTS;
        tilted[11] = 30.0;
        assert!(prepared_drift(&tilted) < DITHER_STEP, "tilted, no cell is off by a whole dither step");
    }
}
