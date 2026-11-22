use std::collections::BTreeSet;

use slint::{Image, Rgba8Pixel};

use crate::Matrix;
use crate::render::{PictureOptions, fit, paint, picture, shrink};

const DITHER_SHARE: f32 = 0.78;
const LADDER_STEPS: usize = 9;
const PRECISION: f32 = 240.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RevealEnd {
    Photo,
    Dither,
}

pub struct Reveal {
    pub coarsest: u32,
    pub progress: f32,
    pub end: RevealEnd,
}

pub fn ladder(coarsest: u32) -> Vec<u32> {
    let top = coarsest.max(1) as f32;
    let steps: BTreeSet<u32> = (0..LADDER_STEPS)
        .map(|index| (top.powf(1.0 - index as f32 / (LADDER_STEPS - 1) as f32).round() as u32).max(1))
        .collect();
    steps.into_iter().rev().collect()
}

pub fn quantize(progress: f32) -> f32 {
    (progress.clamp(0.0, 1.0) * PRECISION).round() / PRECISION
}

pub fn stage_at(progress: f32, ladder: &[u32], end: RevealEnd) -> (u32, f32) {
    let dither_end = if end == RevealEnd::Photo { DITHER_SHARE } else { 1.0 };
    let along = (progress / dither_end).clamp(0.0, 1.0);
    let index = ((along * ladder.len() as f32).floor() as usize).min(ladder.len() - 1);
    let photo = if end == RevealEnd::Photo { ((progress - dither_end) / (1.0 - dither_end)).clamp(0.0, 1.0) } else { 0.0 };
    (ladder[index], photo)
}

pub fn reveal(source: &Image, columns: u32, rows: u32, options: &PictureOptions, reveal: &Reveal) -> Image {
    let (factor, photo) = stage_at(quantize(reveal.progress), &ladder(reveal.coarsest), reveal.end);
    let blocks = picture(source, (columns / factor).max(1), (rows / factor).max(1), options);
    let (Some(source), Some(blocks)) = (source.to_rgba8(), blocks.to_rgba8()) else { return Image::default() };
    if source.width() == 0 || source.height() == 0 || columns == 0 || rows == 0 {
        return Image::default();
    }
    let (width, height) = fit(source.width(), source.height(), columns, rows);
    let original = shrink(source.as_slice(), source.width(), source.height(), width, height);
    let (block_width, block_height) = (blocks.width(), blocks.height());

    paint(width, height, |x, y| {
        if Matrix::Bayer4.is_inked(x as i32, y as i32, photo) {
            let at = (y * width + x) as usize * 4;
            return Rgba8Pixel::new(original[at], original[at + 1], original[at + 2], original[at + 3]);
        }
        blocks.as_slice()[((y * block_height / height) * block_width + x * block_width / width) as usize]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ladder_runs_from_the_coarsest_blocks_to_single_cells() {
        let steps = ladder(24);
        assert_eq!(steps.first(), Some(&24));
        assert_eq!(steps.last(), Some(&1));
        assert!(steps.windows(2).all(|pair| pair[0] > pair[1]));
    }

    #[test]
    fn a_ladder_with_no_coarse_blocks_is_one_step() {
        assert_eq!(ladder(1), vec![1]);
        assert_eq!(ladder(0), vec![1]);
    }

    #[test]
    fn the_photo_only_shows_after_the_dither_has_sharpened() {
        let steps = ladder(24);
        assert_eq!(stage_at(0.0, &steps, RevealEnd::Photo), (24, 0.0));
        let (factor, photo) = stage_at(DITHER_SHARE, &steps, RevealEnd::Photo);
        assert_eq!((factor, photo), (1, 0.0));
        assert_eq!(stage_at(1.0, &steps, RevealEnd::Photo), (1, 1.0));
    }

    #[test]
    fn ending_on_the_dither_never_shows_the_photo() {
        let steps = ladder(24);
        assert_eq!(stage_at(1.0, &steps, RevealEnd::Dither), (1, 0.0));
        assert_eq!(stage_at(0.5, &steps, RevealEnd::Dither).1, 0.0);
    }

    #[test]
    fn progress_snaps_to_the_precision_steps() {
        assert_eq!(quantize(-1.0), 0.0);
        assert_eq!(quantize(2.0), 1.0);
        assert_eq!(quantize(0.5001), 0.5);
    }
}
