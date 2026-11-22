use glam::Vec2;

use super::param;
use crate::field::{Cell, Field};
use crate::noise::fbm;

pub struct MeshGradient;

impl Field for MeshGradient {
    // params: blobs (3 to 5), size
    fn tone(&self, cell: &Cell) -> f32 {
        let [count, size] = std::array::from_fn(|index| param(cell.params, index));
        let count = count.round().clamp(3.0, 5.0);
        let size = size.max(0.05);
        let t = cell.time;
        let aspect = cell.resolution.x / cell.resolution.y;
        let mut p = Vec2::new(cell.uv.x * aspect, cell.uv.y);
        let warp = Vec2::new(fbm(p * 1.1 + Vec2::new(t * 0.02, 0.0)), fbm(p * 1.1 + Vec2::new(7.3, -t * 0.02)));
        p += (warp - 0.5) * 0.35;

        let (mut weights, mut tones) = (0.32, 0.0);
        for index in 0..count as usize {
            let i = index as f32;
            let (sx, sy) = (0.045 * (1.0 + i * 0.23), 0.038 * (1.0 + i * 0.19));
            let centre = Vec2::new((0.5 + 0.38 * (t * sx + i * 2.4).cos()) * aspect, 0.5 + 0.36 * (t * sy + i * 1.7).sin());
            let distance = p.distance(centre) / size;
            let weight = (-distance * distance * 2.2).exp();
            weights += weight;
            tones += weight * (i + 1.0) / count;
        }
        tones / weights
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Palette;

    fn tones(blobs: f32, time: f32) -> Vec<f32> {
        let palette = Palette::new(&[]);
        let params = [blobs, 0.6];
        (0..400)
            .map(|i| {
                let uv = Vec2::new((i % 20) as f32 / 20.0, (i / 20) as f32 / 20.0);
                MeshGradient.tone(&Cell { uv, resolution: Vec2::new(200.0, 100.0), time, params: &params, palette: &palette })
            })
            .collect()
    }

    #[test]
    fn the_blobs_blend_and_drift() {
        let now = tones(4.0, 3.0);
        assert!(now.iter().all(|t| (0.0..=1.0).contains(t)));
        let (low, high) = now.iter().fold((1.0_f32, 0.0_f32), |(low, high), &t| (low.min(t), high.max(t)));
        assert!(high - low > 0.3, "several blob colours show");
        assert_ne!(now, tones(4.0, 30.0));
    }

    #[test]
    fn the_blob_count_is_kept_between_three_and_five() {
        assert_eq!(tones(1.0, 3.0), tones(3.0, 3.0));
        assert_eq!(tones(9.0, 3.0), tones(5.0, 3.0));
    }
}
