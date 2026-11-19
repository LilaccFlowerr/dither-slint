use std::sync::LazyLock;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use crate::blue_noise::BLUE_NOISE_64_BASE64;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Matrix {
    Bayer2,
    Bayer4,
    Bayer8,
    BlueNoise,
}

pub struct ThresholdMap {
    pub size: usize,
    pub steps: usize,
    pub ranks: Vec<u8>,
    pub thresholds: Vec<f32>,
}

impl ThresholdMap {
    fn from_ranks(size: usize, ranks: Vec<u8>, steps: usize) -> Self {
        let thresholds = ranks
            .iter()
            .map(|&rank| (rank as f32 + 0.5) / steps as f32)
            .collect();
        Self { size, steps, ranks, thresholds }
    }
}

fn bayer_ranks(size: usize) -> Vec<u8> {
    let mut ranks = vec![0u8; size * size];
    for y in 0..size {
        for x in 0..size {
            let mut rank = 0usize;
            let mut bit = 1;
            while bit < size {
                let x_bit = (x / bit) % 2;
                let y_bit = (y / bit) % 2;
                rank = rank * 4 + 2 * ((x_bit + y_bit) % 2) + y_bit;
                bit *= 2;
            }
            ranks[y * size + x] = rank as u8;
        }
    }
    ranks
}

static BAYER_2: LazyLock<ThresholdMap> =
    LazyLock::new(|| ThresholdMap::from_ranks(2, bayer_ranks(2), 4));
static BAYER_4: LazyLock<ThresholdMap> =
    LazyLock::new(|| ThresholdMap::from_ranks(4, bayer_ranks(4), 16));
static BAYER_8: LazyLock<ThresholdMap> =
    LazyLock::new(|| ThresholdMap::from_ranks(8, bayer_ranks(8), 64));

static BLUE_NOISE_64: LazyLock<ThresholdMap> = LazyLock::new(|| {
    let ranks = STANDARD
        .decode(BLUE_NOISE_64_BASE64)
        .expect("blue noise table is valid base64");
    ThresholdMap::from_ranks(64, ranks, 256)
});

impl Matrix {
    pub const ALL: [Self; 4] = [Self::Bayer2, Self::Bayer4, Self::Bayer8, Self::BlueNoise];

    pub fn map(self) -> &'static ThresholdMap {
        match self {
            Matrix::Bayer2 => &BAYER_2,
            Matrix::Bayer4 => &BAYER_4,
            Matrix::Bayer8 => &BAYER_8,
            Matrix::BlueNoise => &BLUE_NOISE_64,
        }
    }

    pub fn threshold_at(self, x: i32, y: i32) -> f32 {
        let map = self.map();
        let size = map.size as i32;
        let x = x.rem_euclid(size) as usize;
        let y = y.rem_euclid(size) as usize;
        map.thresholds[y * map.size + x]
    }

    pub fn is_inked(self, x: i32, y: i32, tone: f32) -> bool {
        tone > self.threshold_at(x, y)
    }

    pub fn palette_index(self, x: i32, y: i32, tone: f32, colors: usize) -> usize {
        let scaled = tone.clamp(0.0, 1.0) * (colors - 1) as f32;
        let index = scaled.floor();
        let up = scaled - index > self.threshold_at(x, y);
        (index as usize + up as usize).min(colors - 1)
    }
}

pub fn quantize(tone: f32, levels: u32) -> f32 {
    if levels < 2 {
        return tone;
    }
    let steps = (levels - 1) as f32;
    (tone * steps + 0.5).floor() / steps
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Matrix; 4] = [
        Matrix::Bayer2,
        Matrix::Bayer4,
        Matrix::Bayer8,
        Matrix::BlueNoise,
    ];

    #[test]
    fn bayer2_ranks() {
        assert_eq!(Matrix::Bayer2.map().ranks, [0, 2, 3, 1]);
    }

    #[test]
    fn ranks_are_a_permutation() {
        for matrix in ALL {
            let map = matrix.map();
            let mut ranks = map.ranks.clone();
            ranks.sort();
            if matrix == Matrix::BlueNoise {
                assert_eq!(ranks.len(), 64 * 64);
                assert!(ranks.iter().all(|&rank| rank as usize <= 255));
                continue;
            }
            let expected: Vec<u8> = (0..map.steps as u8).collect();
            assert_eq!(ranks, expected);
        }
    }

    #[test]
    fn tone_extremes() {
        for matrix in ALL {
            let size = matrix.map().size as i32;
            for y in 0..size {
                for x in 0..size {
                    assert!(!matrix.is_inked(x, y, 0.0));
                    assert!(matrix.is_inked(x, y, 1.0));
                }
            }
        }
    }

    #[test]
    fn quarter_tone_inks_a_quarter() {
        let size = Matrix::Bayer4.map().size as i32;
        let inked = (0..size)
            .flat_map(|y| (0..size).map(move |x| (x, y)))
            .filter(|&(x, y)| Matrix::Bayer4.is_inked(x, y, 0.25))
            .count();
        assert_eq!(inked, 4);
    }

    #[test]
    fn coordinates_wrap() {
        assert_eq!(
            Matrix::Bayer4.threshold_at(-1, -1),
            Matrix::Bayer4.threshold_at(3, 3)
        );
    }

    #[test]
    fn two_colours_pick_like_ink_and_paper() {
        for tone in [0.0, 0.2, 0.5, 0.9, 1.0] {
            for (x, y) in [(0, 0), (1, 2), (3, 1)] {
                let picked = Matrix::Bayer4.palette_index(x, y, tone, 2) == 1;
                assert_eq!(picked, Matrix::Bayer4.is_inked(x, y, tone));
            }
        }
    }

    #[test]
    fn three_colours_use_the_middle_one_around_half() {
        let middle = (0..4)
            .flat_map(|y| (0..4).map(move |x| (x, y)))
            .filter(|&(x, y)| Matrix::Bayer4.palette_index(x, y, 0.25, 3) == 1)
            .count();
        assert_eq!(middle, 8);
    }

    #[test]
    fn levels_snap_the_tone() {
        assert_eq!(quantize(0.4, 2), 0.0);
        assert_eq!(quantize(0.6, 2), 1.0);
        assert_eq!(quantize(0.4, 0), 0.4);
        assert_eq!(quantize(0.3, 5), 0.25);
    }
}
