use glam::{Vec2, Vec3};

use super::{param, smoothstep};
use crate::field::{Cell, Field};
use crate::noise::{fbm, snoise};

pub struct Orb;

impl Field for Orb {
    // params: glow, lean x and y (-1 to 1, y grows downwards), morph, size. With a highlight
    // colour in the palette the body stays below the top tone.
    fn tone(&self, cell: &Cell) -> f32 {
        let body = if cell.palette.len() > 2 { 0.55 } else { 1.0 };
        let glow = param(cell.params, 0).clamp(0.0, 1.0);
        let lean = Vec2::new(param(cell.params, 1), -param(cell.params, 2)).clamp(Vec2::splat(-1.0), Vec2::splat(1.0));
        let morph = param(cell.params, 3).max(0.0);
        let size = param(cell.params, 4).clamp(0.1, 0.5);

        let aspect = cell.resolution.x / cell.resolution.y;
        let fit = aspect.min(1.0);
        let t = cell.time;
        let p = (cell.uv - 0.5) * Vec2::new(aspect, 1.0) * 2.0 / fit;
        let q = p - lean * 0.16;
        let r = q.length();
        let angle = q.y.atan2(q.x);

        // A slowly breathing outline: two low harmonics plus soft noise.
        let wobble = 0.045 * (3.0 * angle + t * 0.6).sin()
            + 0.03 * (5.0 * angle - t * 0.8 + 1.3).sin()
            + 0.05 * snoise(q * 1.2 + Vec2::new(t * 0.12, -t * 0.09));
        let radius = size * 2.0 * (1.0 + morph * wobble);
        let d = r / radius;

        let light = Vec3::new(-0.5 + lean.x * 0.35, 0.55 + lean.y * 0.35, 0.8).normalize();
        if d < 1.0 {
            let n = Vec3::new(q.x / radius, q.y / radius, (1.0 - d * d).sqrt());
            let diffuse = n.dot(light).max(0.0);
            let reflected = -light - 2.0 * n.dot(-light) * n;
            let spec = reflected.z.max(0.0).powf(60.0);
            let rim = (1.0 - n.z).powi(3);
            // Faint currents inside the orb so the surface never looks flat.
            let swirl = fbm(n.truncate() * 1.8 + Vec2::new(t * 0.05, t * 0.03)) - 0.5;
            let shade = 0.09 + 0.82 * diffuse.powf(1.4) + 0.28 * rim + 0.14 * swirl * diffuse;
            return (shade * body + spec * 0.8).clamp(0.0, 1.0);
        }
        // Outside, a glow that falls off with distance and is brighter on the lit side.
        let side = 0.25 + 0.75 * smoothstep(-0.2, 1.0, q.normalize_or_zero().dot(light.truncate().normalize()));
        glow * 0.4 * (-(d - 1.0) * 7.0).exp() * side
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Palette;
    use slint::Color;

    const RESTING: [f32; 5] = [0.5, 0.0, 0.0, 1.0, 0.26];

    fn tone(params: &[f32], uv: Vec2, time: f32) -> f32 {
        let palette = Palette::new(&[]);
        Orb.tone(&Cell { uv, resolution: Vec2::new(200.0, 200.0), time, params, palette: &palette })
    }

    #[test]
    fn the_orb_is_lit_and_the_corners_are_dark() {
        assert!(tone(&RESTING, Vec2::new(0.45, 0.55), 2.0) > 0.4);
        assert!(tone(&RESTING, Vec2::new(0.02, 0.98), 2.0) < 0.01);
    }

    #[test]
    fn a_highlight_colour_keeps_the_body_below_the_top_tone() {
        let palette = Palette::new(&[Color::from_rgb_u8(0, 0, 0); 3]);
        let uv = Vec2::new(0.45, 0.55);
        let cell = Cell { uv, resolution: Vec2::new(200.0, 200.0), time: 2.0, params: &RESTING, palette: &palette };
        assert!(Orb.tone(&cell) < tone(&RESTING, uv, 2.0));
    }

    #[test]
    fn the_glow_surrounds_the_orb_and_can_be_switched_off() {
        let beside = Vec2::new(0.5 + 0.26 + 0.03, 0.4);
        assert!(tone(&RESTING, beside, 2.0) > 0.0);
        let dark = [0.0, 0.0, 0.0, 1.0, 0.26];
        assert_eq!(tone(&dark, beside, 2.0), 0.0);
    }

    #[test]
    fn a_bigger_size_covers_more_of_the_box() {
        let covered = |size: f32| {
            let params = [0.0, 0.0, 0.0, 0.0, size];
            (0..100).filter(|&i| tone(&params, Vec2::new(0.5 + i as f32 * 0.005, 0.5), 0.0) > 0.0).count()
        };
        assert!(covered(0.4) > covered(0.2));
    }

    #[test]
    fn the_light_leans_towards_the_pointer() {
        let lit_left = |lean_x: f32| {
            let params = [0.0, lean_x, 0.0, 0.0, 0.26];
            tone(&params, Vec2::new(0.42, 0.5), 0.0)
        };
        assert_ne!(lit_left(-1.0), lit_left(1.0));
    }

    #[test]
    fn morph_changes_the_outline_over_time() {
        let edge = |time: f32| tone(&[0.0, 0.0, 0.0, 1.0, 0.26], Vec2::new(0.72, 0.5), time);
        assert_ne!(edge(0.0), edge(5.0));
    }
}
