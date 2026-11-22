use glam::Vec2;

use super::{mix, param, smoothstep};
use crate::field::{Cell, Field};

pub struct RetroGrid;

/// A grid line at every whole value of `v`, about one cell wide whatever the perspective.
/// Lines closer than a couple of cells fade out instead of turning into moiré.
fn line(v: f32, vx: f32, vy: f32) -> f32 {
    let offset = ((v + 0.5) - (v + 0.5).floor() - 0.5).abs();
    let per_cell = Vec2::new(vx - v, vy - v).length().max(1e-4);
    (1.0 - smoothstep(0.35, 1.1, offset / per_cell)) * (1.0 - smoothstep(0.2, 0.45, per_cell))
}

fn floor(uv: Vec2, aspect: f32, t: f32, horizon: f32, columns: f32) -> Vec2 {
    let below = (horizon - uv.y).max(1e-4) / horizon.max(1e-3);
    let depth = 1.0 / below;
    let across = (uv.x - 0.5) * aspect * depth * columns / 1.6;
    let rows = 1.5 * horizon * columns / 1.6;
    Vec2::new(across, depth * rows + t * 0.35)
}

impl Field for RetroGrid {
    // params: horizon, sun, columns
    fn tone(&self, cell: &Cell) -> f32 {
        let horizon = param(cell.params, 0).clamp(0.05, 0.95);
        let sun = param(cell.params, 1).clamp(0.0, 1.0);
        let columns = param(cell.params, 2).max(2.0);

        let (t, uv, resolution) = (cell.time, cell.uv, cell.resolution);
        let aspect = resolution.x / resolution.y;
        let step = 1.0 / resolution;

        if uv.y < horizon {
            let g = floor(uv, aspect, t, horizon, columns);
            let gx = floor(uv + Vec2::new(step.x, 0.0), aspect, t, horizon, columns);
            let gy = floor(uv + Vec2::new(0.0, step.y), aspect, t, horizon, columns);
            let lines = line(g.x, gx.x, gy.x).max(line(g.y, gx.y, gy.y));
            let near = (horizon - uv.y) / horizon.max(1e-3);
            let haze = (-near * 14.0).exp() * 0.4;
            let glare = if sun > 0.001 { (-((uv.x - 0.5) * aspect * 2.6).powi(2)).exp() * (-near * 5.0).exp() * 0.25 } else { 0.0 };
            let tone = (lines * mix(0.6, 1.0, smoothstep(0.0, 0.6, near))).max(haze + glare);
            let edge = (1.0 - smoothstep(0.0, 1.6, (horizon - uv.y) * resolution.y)) * 0.95;
            return tone.max(edge).clamp(0.0, 1.0);
        }

        let mut tone = (-(uv.y - horizon) * 28.0).exp() * 0.3;
        if sun > 0.001 {
            let r = sun * 0.5;
            let c = Vec2::new((uv.x - 0.5) * aspect, uv.y - (horizon + r * 0.2));
            let distance = c.length();
            let disc = 1.0 - smoothstep(r - 1.2 / resolution.y, r, distance);
            let body = mix(0.62, 1.0, smoothstep(-r, r, c.y));
            let k = ((r * 0.6 - c.y) / (r * 0.85)).clamp(0.0, 1.0);
            let bands = c.y / r * 6.0 + t * 0.08;
            let bands = bands - bands.floor();
            let cut = if k * 0.6 >= bands && k >= 0.0001 { 1.0 } else { 0.0 };
            let glow = (-(distance - r).max(0.0) * 9.0 / (r * 2.0).max(0.2)).exp() * 0.4;
            tone = tone.max((disc * body * (1.0 - cut)).max(glow * (1.0 - disc)));
        }
        tone.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Palette;

    fn tone(params: &[f32], uv: Vec2) -> f32 {
        let palette = Palette::new(&[]);
        RetroGrid.tone(&Cell { uv, resolution: Vec2::new(160.0, 100.0), time: 3.0, params, palette: &palette })
    }

    #[test]
    fn the_sun_sits_on_the_horizon_and_the_floor_has_lines() {
        let params = [0.42, 0.4, 12.0];
        assert!(tone(&params, Vec2::new(0.5, 0.6)) > 0.6, "inside the sun");
        assert!(tone(&params, Vec2::new(0.05, 0.9)) < 0.1, "the sky far from the sun");
        let floor = (0..160).map(|x| tone(&params, Vec2::new((x as f32 + 0.5) / 160.0, 0.1))).collect::<Vec<_>>();
        assert!(floor.iter().any(|&t| t > 0.5) && floor.iter().any(|&t| t < 0.2), "grid lines with gaps");
    }

    #[test]
    fn no_sun_leaves_the_sky_empty() {
        assert!(tone(&[0.42, 0.0, 12.0], Vec2::new(0.5, 0.6)) < 0.1);
    }
}
