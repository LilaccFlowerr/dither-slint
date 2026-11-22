use glam::Vec2;

use super::{param, smoothstep};
use crate::field::{Cell, Field};
use crate::noise::fbm;

pub struct Spotlight;

impl Field for Spotlight {
    // params: x and y of the light as fractions of the box from the top left, radius, intensity.
    // The uv of a cell points up.
    fn tone(&self, cell: &Cell) -> f32 {
        let [origin_x, origin_y, radius, intensity] = std::array::from_fn(|index| param(cell.params, index));
        let origin = Vec2::new(origin_x, origin_y);
        let radius = radius.max(0.05);
        let intensity = intensity.clamp(0.0, 1.0);

        let aspect = cell.resolution.x / cell.resolution.y;
        let t = cell.time;
        let p = Vec2::new(cell.uv.x * aspect, 1.0 - cell.uv.y);
        let centre = Vec2::new(origin.x * aspect, origin.y);
        let far = Vec2::new(centre.x.max(aspect - centre.x), centre.y.max(1.0 - centre.y));
        let reach = far.length() * radius;
        let breathe = 1.0 + 0.045 * (t * 0.32).sin();
        let warp = fbm(p * 1.6 + Vec2::new(t * 0.035, -t * 0.025)) - 0.5;
        let d = (p.distance(centre) / (reach * breathe) + warp * 0.22).max(0.0);
        let core = (-d * d * 4.5).exp();
        let halo = (-d * 2.6).exp() * (1.0 - smoothstep(0.55, 1.25, d));
        intensity * (core * 0.6 + halo * 0.4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Palette;

    fn tone(params: &[f32], uv: Vec2, time: f32) -> f32 {
        let palette = Palette::new(&[]);
        Spotlight.tone(&Cell { uv, resolution: Vec2::new(200.0, 100.0), time, params, palette: &palette })
    }

    const TOP: [f32; 4] = [0.5, -0.05, 0.7, 0.75];

    #[test]
    fn the_light_is_brightest_at_its_origin_and_fades_away() {
        let near = tone(&TOP, Vec2::new(0.5, 0.98), 1.0);
        let far = tone(&TOP, Vec2::new(0.5, 0.02), 1.0);
        assert!(near > 0.4);
        assert!(far < near * 0.25);
    }

    #[test]
    fn the_origin_can_move_to_a_corner() {
        let corner = [1.0, 1.0, 0.7, 0.75];
        assert!(tone(&corner, Vec2::new(0.98, 0.02), 1.0) > tone(&corner, Vec2::new(0.02, 0.98), 1.0));
    }

    #[test]
    fn a_larger_radius_reaches_further() {
        let wide = [0.5, -0.05, 1.5, 0.75];
        let probe = Vec2::new(0.5, 0.1);
        assert!(tone(&wide, probe, 1.0) > tone(&TOP, probe, 1.0));
    }

    #[test]
    fn zero_intensity_is_dark() {
        assert_eq!(tone(&[0.5, -0.05, 0.7, 0.0], Vec2::new(0.5, 0.9), 1.0), 0.0);
    }

    #[test]
    fn the_light_breathes() {
        let at = |time: f32| tone(&TOP, Vec2::new(0.5, 0.5), time);
        assert_ne!(at(0.0), at(5.0));
    }
}
