use std::f32::consts::{PI, TAU};

use glam::Vec2;

use super::{param, smoothstep};
use crate::field::{Cell, Field};
use crate::noise::{fbm, snoise};

pub struct Beams;

impl Field for Beams {
    // params: origin x, origin y (fractions of the box from the top left), angle in degrees
    // as in CSS, spread in degrees, rays, sweep in degrees, reach
    fn tone(&self, cell: &Cell) -> f32 {
        let origin = Vec2::new(param(cell.params, 0), param(cell.params, 1));
        let angle = param(cell.params, 2).to_radians();
        let spread = param(cell.params, 3).clamp(1.0, 359.0).to_radians();
        let rays = param(cell.params, 4).max(1.0);
        let sweep = param(cell.params, 5).to_radians();
        let reach = param(cell.params, 6).max(0.05);

        let t = cell.time;
        let aspect = cell.resolution.x / cell.resolution.y;
        let p = Vec2::new(cell.uv.x * aspect, cell.uv.y);
        let o = Vec2::new(origin.x * aspect, 1.0 - origin.y);
        let d = p - o;
        let half_box = Vec2::new(aspect, 1.0) * 0.5;
        let entry = ((o - half_box).abs() - half_box).max(Vec2::ZERO).length();
        let travelled = (d.length() - entry).max(0.0);

        let axis = angle + sweep * (t * 0.11).sin();
        let a = (d.x.atan2(d.y) - axis + PI).rem_euclid(TAU) - PI;
        let half_width = spread * 0.5;
        let cone = smoothstep(half_width, 0.0, a.abs()).sqrt();

        let k = rays / half_width.max(0.02);
        let r1 = 0.5 + 0.5 * snoise(Vec2::new(a * k * 0.5, t * 0.05));
        let r2 = 0.5 + 0.5 * snoise(Vec2::new(a * k * 1.4 + 7.0, t * 0.08 + 3.0));
        let shafts = 0.12 + 1.1 * r1.powi(2) + 0.25 * r2.powi(3);
        let dust = 0.88 + 0.12 * fbm(Vec2::new(a * k * 0.3, travelled * 3.0 - t * 0.06));

        let reach = Vec2::new(aspect, 1.0).length() * reach;
        let fall = (-travelled / reach * 2.4).exp();
        let core = (-travelled * 8.0).exp() * 0.35;
        (cone * (shafts * dust * fall * 1.3 + core)).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Palette;

    const DEFAULTS: [f32; 7] = [0.18, -0.12, 160.0, 80.0, 9.0, 6.0, 1.0];

    fn tone(params: &[f32], uv: Vec2, time: f32) -> f32 {
        let palette = Palette::new(&[]);
        Beams.tone(&Cell { uv, resolution: Vec2::new(200.0, 100.0), time, params, palette: &palette })
    }

    #[test]
    fn the_light_falls_inside_its_cone() {
        let along = (0..10).map(|step| tone(&DEFAULTS, Vec2::new(0.3, 0.9 - step as f32 * 0.08), 4.0)).sum::<f32>();
        let beside = (0..10).map(|step| tone(&DEFAULTS, Vec2::new(0.95, 0.9 - step as f32 * 0.08), 4.0)).sum::<f32>();
        assert!(along > beside * 2.0);
    }

    #[test]
    fn the_light_fades_with_distance() {
        let near = tone(&DEFAULTS, Vec2::new(0.22, 0.95), 4.0);
        let far = tone(&DEFAULTS, Vec2::new(0.45, 0.05), 4.0);
        assert!(near > far);
    }

    #[test]
    fn the_rays_sweep_over_time() {
        let at = |time: f32| (0..20).map(|step| tone(&DEFAULTS, Vec2::new(step as f32 / 20.0, 0.6), time)).collect::<Vec<_>>();
        assert_ne!(at(4.0), at(14.0));
    }
}
