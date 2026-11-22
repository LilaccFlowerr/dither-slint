use std::f32::consts::{PI, TAU};

use glam::Vec2;

use super::{param, smoothstep};
use crate::field::{Cell, Field};

pub struct AnimatedGradient;

fn rotate(p: Vec2, angle: f32) -> Vec2 {
    let (sin, cos) = angle.sin_cos();
    Vec2::new(cos * p.x - sin * p.y, sin * p.x + cos * p.y)
}

fn random(p: Vec2) -> f32 {
    let value = p.dot(Vec2::new(12.9898, 78.233)).sin() * 43758.547;
    value - value.floor()
}

fn noise(p: Vec2) -> f32 {
    let (i, f) = (p.floor(), p - p.floor());
    let (a, b) = (random(i), random(i + Vec2::X));
    let (c, d) = (random(i + Vec2::Y), random(i + Vec2::ONE));
    let u = f * f * (3.0 - 2.0 * f);
    let top = a + (b - a) * u.x;
    let bottom = c + (d - c) * u.x;
    top + (bottom - top) * u.y
}

impl Field for AnimatedGradient {
    // params, on the 0 to 100 scale of the original's knobs where it has one:
    // cell, rotation, proportion, scale, speed, distortion, swirl, swirl iterations,
    // softness, offset, shape (0 checks, 1 stripes, 2 edge), shape size
    fn tone(&self, cell: &Cell) -> f32 {
        let [size, rotation, proportion, scale, rate, distortion, swirl, iterations, softness, offset, shape, shape_scale] = std::array::from_fn(|index| param(cell.params, index));
        let size = size.max(1.0);
        let rotation = rotation.to_radians();
        let proportion = (proportion / 100.0).clamp(0.0, 1.0);
        let rate = rate / 100.0 * 5.0;
        let distortion = distortion / 50.0;
        let swirl = (swirl / 100.0).clamp(0.0, 2.0);
        let iterations = if swirl == 0.0 { 0.0 } else { iterations.ceil().clamp(1.0, 30.0) };
        let softness = (softness / 100.0).clamp(0.0, 1.0);
        let offset = offset * 0.01;
        let shape_scale = shape_scale / 100.0;

        let px = cell.resolution * size;
        let t = 0.5 * (cell.time * rate + offset);
        let noise_scale = 0.0005 + 0.006 * scale;

        let mut p = (cell.uv - 0.5) * noise_scale * px;
        p = rotate(p, rotation * 0.5 * PI) + 0.5;

        let n1 = noise(p + t);
        let n2 = noise(p * 2.0 - t);
        let angle = n1 * TAU;
        p += 4.0 * distortion * n2 * Vec2::new(angle.cos(), angle.sin());

        let mut step = 1.0;
        while step <= iterations {
            p.x += swirl / step * (t + step * 1.5 * p.y).cos();
            p.y += swirl / step * (t + step * p.x).cos();
            step += 1.0;
        }

        let bias = 0.48 * (proportion - 0.5).signum() * (proportion - 0.5).abs().sqrt();
        let mixer = if shape < 0.5 {
            let q = p * (0.5 + 3.5 * shape_scale);
            0.5 + 0.5 * q.x.sin() * q.y.cos() + bias
        } else if shape < 1.5 {
            let f = p.y * (0.25 + 3.0 * shape_scale);
            let f = f - f.floor();
            smoothstep(0.0, 0.55, f) * smoothstep(1.0, 0.45, f) + bias
        } else {
            let height = (0.5 - p.y) / (noise_scale * px.y) + 0.5;
            let spread = 0.2 * (1.0 - shape_scale);
            smoothstep(0.45 - spread, 0.55 + spread, height + 0.3 * (proportion - 0.5))
        };

        let edges = 1.0 - softness;
        let blur = 0.01 + 0.01 * scale;
        let r1 = smoothstep(0.35 * edges, 0.7 - 0.35 * edges + 0.5 * blur, mixer);
        let r2 = smoothstep(0.3 + 0.35 * edges, 1.0 - 0.35 * edges + blur, mixer);
        0.5 * r1 + 0.5 * r2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Palette;

    const PRISM: [f32; 12] = [3.0, -50.0, 1.0, 0.01, 30.0, 0.0, 50.0, 16.0, 47.0, -299.0, 0.0, 45.0];
    const LAVA: [f32; 12] = [3.0, 114.0, 100.0, 0.52, 30.0, 7.0, 18.0, 20.0, 100.0, 717.0, 2.0, 12.0];

    fn tones(params: &[f32], time: f32) -> Vec<f32> {
        let palette = Palette::new(&[]);
        (0..400)
            .map(|i| {
                let uv = Vec2::new((i % 20) as f32 / 20.0, (i / 20) as f32 / 20.0);
                AnimatedGradient.tone(&Cell { uv, resolution: Vec2::new(200.0, 100.0), time, params, palette: &palette })
            })
            .collect()
    }

    #[test]
    fn the_gradient_stays_in_range_and_swirls() {
        for params in [PRISM, LAVA] {
            let now = tones(&params, 2.5);
            assert!(now.iter().all(|t| (0.0..=1.0).contains(t)));
            assert_ne!(now, tones(&params, 9.0));
        }
    }

    #[test]
    fn the_edge_shape_runs_from_one_colour_to_the_other() {
        let now = tones(&LAVA, 2.5);
        assert!(now.iter().any(|&t| t < 0.2) && now.iter().any(|&t| t > 0.8));
    }
}
