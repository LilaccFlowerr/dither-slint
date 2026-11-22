use glam::{Vec2, Vec4};

use super::param;
use crate::field::{Cell, Field};

pub struct LightRays;

/// The time the original starts its rays at, past their first flickers.
const START: f32 = 1000.0;

struct Beam {
    source: Vec2,
    direction: Vec2,
    seeds: (f32, f32),
    pace: f32,
}

fn strength(beam: &Beam, coord: Vec2, resolution: Vec2, speed: f32, t: f32, reach: f32) -> f32 {
    let to = coord - beam.source;
    let facing = to.normalize_or_zero().dot(beam.direction);
    let diagonal = resolution.length();
    let wave = (0.45 + 0.15 * (facing * beam.seeds.0 * speed + t * beam.pace * speed).sin())
        + (0.30 + 0.20 * (-facing * beam.seeds.1 * speed + t * beam.pace * speed).cos());
    wave.clamp(reach, 1.0) * ((diagonal - to.length()) / diagonal).clamp(reach, 1.0)
}

/// How much light each of the two beams gives this cell.
fn alpha(cell: &Cell) -> Vec2 {
    let [count, intensity, position, rate, reach] = std::array::from_fn(|index| param(cell.params, index));
    let count = count.clamp(0.0, 100.0) / 100.0 * 0.3;
    let intensity = intensity.clamp(0.0, 100.0) / 100.0 * 0.5;
    let position = position / 100.0;
    let rate = rate.max(0.0) / 10.0;
    let reach = reach.clamp(0.0, 100.0) / 100.0 * 0.5;

    let resolution = cell.resolution;
    let t = cell.time * rate + START;
    let coord = Vec2::new(cell.uv.x * resolution.x, (1.0 - cell.uv.y) * resolution.y);
    let speed = count * 10.0;
    let first = Beam {
        source: Vec2::new(position * resolution.x, -0.4 * resolution.y),
        direction: Vec2::new(1.0, -0.116).normalize(),
        seeds: (36.2214, 21.11349),
        pace: 1.5,
    };
    let second = Beam {
        source: Vec2::new((position + 0.02) * resolution.x, -0.5 * resolution.y),
        direction: Vec2::new(1.0, 0.241).normalize(),
        seeds: (22.3991, 18.0234),
        pace: 1.1,
    };
    let attenuation = (reach - coord.y / resolution.y + 0.5 + intensity).clamp(0.0, 1.0);
    Vec2::new(strength(&first, coord, resolution, speed, t, reach), strength(&second, coord, resolution, speed, t, reach)) * attenuation
}

impl Field for LightRays {
    // params, on the 0 to 100 scale of the original's knobs: rays, intensity, position, speed, reach
    fn tone(&self, cell: &Cell) -> f32 {
        let a = alpha(cell);
        a.x + a.y * (1.0 - a.x)
    }

    /// The two beams blend by their strength: the first colour of the palette after the paper, then the second.
    fn color(&self, cell: &Cell) -> Option<Vec4> {
        let a = alpha(cell);
        Some(cell.palette.ramp(a.y / (a.x + a.y).max(0.0001)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Palette;
    use slint::Color;

    const CYBER_BLUE: [f32; 5] = [32.0, 13.0, 50.0, 10.0, 16.0];

    fn cell<'a>(params: &'a [f32], palette: &'a Palette, uv: Vec2, time: f32) -> Cell<'a> {
        Cell { uv, resolution: Vec2::new(200.0, 100.0), time, params, palette }
    }

    #[test]
    fn the_rays_are_brightest_up_top_and_flicker() {
        let palette = Palette::new(&[]);
        let row = |y: f32, time: f32| (0..20).map(|x| LightRays.tone(&cell(&CYBER_BLUE, &palette, Vec2::new(x as f32 / 20.0, y), time))).collect::<Vec<_>>();
        let (top, bottom) = (row(0.95, 1.0), row(0.05, 1.0));
        assert!(top.iter().all(|t| (0.0..=1.0).contains(t)));
        assert!(top.iter().sum::<f32>() > bottom.iter().sum::<f32>());
        assert_ne!(top, row(0.95, 3.0));
    }

    #[test]
    fn the_colour_comes_from_the_two_beams() {
        let red = Color::from_rgb_u8(255, 0, 0);
        let blue = Color::from_rgb_u8(0, 0, 255);
        let palette = Palette::new(&[Color::from_rgb_u8(0, 0, 0), red, blue]);
        let colour = LightRays.color(&cell(&CYBER_BLUE, &palette, Vec2::new(0.5, 0.8), 1.0)).unwrap();
        assert!(colour.x > 0.0 && colour.z > 0.0, "a blend of both beams");
        assert_eq!(colour.y, 0.0);
    }
}
