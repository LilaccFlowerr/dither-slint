use glam::Vec2;

use super::param;
use crate::field::{Cell, Field};
use crate::noise::fbm;

pub struct Fade;

impl Field for Fade {
    // params: side, 0 for the bottom edge, 1 for the top, 2 for the right and 3 for the left.
    // The uv of a cell points up.
    fn tone(&self, cell: &Cell) -> f32 {
        let aspect = cell.resolution.x / cell.resolution.y;
        let (s, along) = match param(cell.params, 0).round() as i32 {
            1 => (cell.uv.y, cell.uv.x * aspect),
            2 => (cell.uv.x, cell.uv.y / aspect),
            3 => (1.0 - cell.uv.x, cell.uv.y / aspect),
            _ => (1.0 - cell.uv.y, cell.uv.x * aspect),
        };
        let t = cell.time;
        let drift = (fbm(Vec2::new(along * 1.6 + t * 0.05, s * 1.2 - t * 0.03)) - 0.5) * 0.35;
        (s + drift * (1.0 - s)).clamp(0.0, 1.0).powf(1.6)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Palette;

    fn tone(side: f32, uv: Vec2) -> f32 {
        let palette = Palette::new(&[]);
        Fade.tone(&Cell { uv, resolution: Vec2::new(200.0, 100.0), time: 2.0, params: &[side], palette: &palette })
    }

    #[test]
    fn each_side_is_solid_at_its_own_edge_and_clear_at_the_other() {
        let edges = [
            (0.0, Vec2::new(0.5, 0.0), Vec2::new(0.5, 1.0)),
            (1.0, Vec2::new(0.5, 1.0), Vec2::new(0.5, 0.0)),
            (2.0, Vec2::new(1.0, 0.5), Vec2::new(0.0, 0.5)),
            (3.0, Vec2::new(0.0, 0.5), Vec2::new(1.0, 0.5)),
        ];
        for (side, solid, clear) in edges {
            assert_eq!(tone(side, solid), 1.0, "side {side}");
            assert!(tone(side, clear) < 0.1, "side {side}");
        }
    }

    #[test]
    fn the_fade_thickens_towards_the_edge() {
        let tones: Vec<f32> = (0..=10).map(|i| tone(0.0, Vec2::new(0.5, 1.0 - i as f32 / 10.0))).collect();
        let average = |from: usize, to: usize| tones[from..to].iter().sum::<f32>() / (to - from) as f32;
        assert!(average(6, 11) > average(0, 5));
    }
}
