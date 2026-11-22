use std::f32::consts::TAU;

use glam::Vec2;

use super::{mix, param, smoothstep};
use crate::field::{Cell, Field};

pub struct Tunnel;

/// A line `width` ring steps wide, at least about a cell, where `per_cell` is one cell in ring steps.
fn line(offset: f32, per_cell: f32, width: f32) -> f32 {
    let d = offset / per_cell;
    let w = (width / per_cell).max(0.55);
    1.0 - smoothstep(w - 0.5, w + 0.5, d)
}

fn wrap(v: f32) -> f32 {
    ((v + 0.5) - (v + 0.5).floor() - 0.5).abs()
}

impl Field for Tunnel {
    // params: shape (0 circle, 1 square), rings, spokes, centre x, centre y, sway, fog
    fn tone(&self, cell: &Cell) -> f32 {
        let square = param(cell.params, 0) > 0.5;
        let rings = param(cell.params, 1).max(0.25);
        let spokes = param(cell.params, 2).round().clamp(0.0, 32.0);
        let centre = Vec2::new(param(cell.params, 3), param(cell.params, 4));
        let sway = param(cell.params, 5).max(0.0);
        let fog_size = param(cell.params, 6).max(0.1);

        let (t, uv) = (cell.time, cell.uv);
        let aspect = cell.resolution.x / cell.resolution.y;
        let o = Vec2::new(centre.x * aspect, 1.0 - centre.y) + sway * Vec2::new((t * 0.07).sin() * 0.05, (t * 0.053).cos() * 0.035);
        let c = Vec2::new(uv.x * aspect, uv.y) - o;
        let cell_size = 1.0 / cell.resolution.y;
        let r = if square { c.x.abs().max(c.y.abs()) } else { c.length() }.max(cell_size * 0.5);

        let density = 7.0 * rings;
        let u = 0.32 / r * density + t * 0.36;
        let per_cell = 0.32 * density / (r * r) * cell_size;
        let ring = line(wrap(u), per_cell, 0.045);

        let (mut spoke, mut tile) = (0.0, 0.0);
        if spokes > 0.5 {
            let a = if square {
                let (side_x, side_y) = (c.x / r, c.y / r);
                if c.x.abs() >= c.y.abs() {
                    if c.x > 0.0 { (side_y + 1.0) * 0.125 } else { 0.5 + (1.0 - side_y) * 0.125 }
                } else if c.y > 0.0 {
                    0.25 + (1.0 - side_x) * 0.125
                } else {
                    0.75 + (side_x + 1.0) * 0.125
                }
            } else {
                c.y.atan2(c.x) / TAU + t * 0.006
            };
            let v = a * spokes;
            tile = (u.floor() + v.floor()).rem_euclid(2.0);
            let per_cell_around = spokes / (mix(TAU, 8.0, if square { 1.0 } else { 0.0 }) * r) * cell_size;
            spoke = line(wrap(v), per_cell_around, 0.004) * 0.8;
        }

        let fog = smoothstep(0.04 * fog_size, 0.4 * fog_size, r) * (1.0 - smoothstep(0.25, 0.6, per_cell));
        let wall = tile * 0.2 * smoothstep(0.12, 0.8, r);
        let tone = (ring * mix(0.5, 1.0, smoothstep(0.1, 0.5, r))).max(spoke);
        (tone.max(wall) * fog).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Palette;

    fn tones(params: &[f32], time: f32) -> Vec<f32> {
        let palette = Palette::new(&[]);
        (0..80 * 60)
            .map(|index| {
                let uv = Vec2::new(((index % 80) as f32 + 0.5) / 80.0, ((index / 80) as f32 + 0.5) / 60.0);
                Tunnel.tone(&Cell { uv, resolution: Vec2::new(160.0, 120.0), time, params, palette: &palette })
            })
            .collect()
    }

    #[test]
    fn the_middle_is_fog_and_the_rings_come_closer() {
        let params = [0.0, 1.0, 12.0, 0.5, 0.5, 0.0, 1.0];
        let now = tones(&params, 6.0);
        assert_eq!(now[30 * 80 + 40], 0.0, "the vanishing point is empty");
        assert!(now.iter().any(|&t| t > 0.5));
        assert_ne!(now, tones(&params, 7.0));
    }

    #[test]
    fn a_square_tunnel_differs_from_a_round_one() {
        assert_ne!(tones(&[0.0, 1.0, 12.0, 0.5, 0.5, 0.0, 1.0], 6.0), tones(&[1.0, 1.0, 12.0, 0.5, 0.5, 0.0, 1.0], 6.0));
    }
}
