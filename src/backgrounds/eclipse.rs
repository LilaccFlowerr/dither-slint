use glam::Vec2;

use super::{param, smoothstep};
use crate::field::{Cell, Field};
use crate::noise::fbm;

pub struct Eclipse;

impl Field for Eclipse {
    // params: centre x, centre y (fractions of the box from the top left), size, corona,
    // light in degrees as in CSS, pulse
    fn tone(&self, cell: &Cell) -> f32 {
        let [centre_x, centre_y, size, corona, light, pulse] = std::array::from_fn(|index| param(cell.params, index));
        let centre = Vec2::new(centre_x, centre_y);
        let size = size.max(0.01);
        let corona = corona.clamp(0.0, 1.0);
        let light = light.to_radians();
        let pulse = pulse.clamp(0.0, 1.0);

        let t = cell.time;
        let aspect = cell.resolution.x / cell.resolution.y;
        let p = Vec2::new(cell.uv.x * aspect, cell.uv.y);
        let d = p - Vec2::new(centre.x * aspect, 1.0 - centre.y);
        let r = d.length();
        let e = r - size;
        if e < 0.0 {
            return 0.0;
        }

        let around = d / r.max(0.0001);
        let facing = 0.5 + 0.5 * (d.x.atan2(d.y) - light).cos();
        let breathe = 1.0 + pulse * 0.12 * (t * 0.6).sin();
        let reach = size * (0.1 + 0.5 * corona) * breathe;

        let streak = fbm(around * 2.2 + Vec2::new(t * 0.02, -t * 0.015));
        let halo = (-e / reach).exp() * (1.0 - smoothstep(reach, reach * 3.2, e)) * (0.7 + 0.6 * streak);
        let rim = (-e / (size * 0.03)).exp();
        let glow = halo * (0.3 + 0.6 * facing) + rim * (0.3 + facing.powf(1.5));
        (glow * breathe).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Palette;

    const DEFAULTS: [f32; 6] = [0.5, 0.5, 0.24, 0.5, 315.0, 0.5];

    fn tone(uv: Vec2) -> f32 {
        let palette = Palette::new(&[]);
        Eclipse.tone(&Cell { uv, resolution: Vec2::new(100.0, 100.0), time: 5.0, params: &DEFAULTS, palette: &palette })
    }

    #[test]
    fn the_disc_is_dark_and_its_rim_bright() {
        assert_eq!(tone(Vec2::new(0.5, 0.5)), 0.0);
        assert!(tone(Vec2::new(0.5, 0.5 + 0.245)) > 0.5);
        assert!(tone(Vec2::new(0.5, 0.5 + 0.245)) > tone(Vec2::new(0.5, 0.95)));
    }

    #[test]
    fn the_lit_side_is_brightest() {
        let top_left = tone(Vec2::new(0.5 - 0.175, 0.5 + 0.175));
        let bottom_right = tone(Vec2::new(0.5 + 0.175, 0.5 - 0.175));
        assert!(top_left > bottom_right);
    }
}
