use slint::{Color, Image, Rgba8Pixel};

use crate::render::{paint, pixel};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bloom {
    Off,
    Low,
    High,
}

impl Bloom {
    pub const ALL: [Self; 3] = [Self::Off, Self::Low, Self::High];

    /// Blur in pixels and opacity of the glow.
    fn level(self) -> Option<(f32, f32)> {
        match self {
            Bloom::Off => None,
            Bloom::Low => Some((10.0, 0.3)),
            Bloom::High => Some((18.0, 0.55)),
        }
    }
}

const FAINTEST: u8 = 3;

fn blur_radius(blur: f32, cell: f32) -> usize {
    let sigma = blur / cell;
    (((1.0 + 4.0 * sigma * sigma).sqrt() - 1.0) / 2.0).round().max(1.0) as usize
}

fn box_blur(values: &mut [f32], width: usize, height: usize, radius: usize) {
    let mut scratch = vec![0.0; values.len()];
    let window = (2 * radius + 1) as f32;
    for horizontal in [true, false] {
        let (lines, length) = if horizontal { (height, width) } else { (width, height) };
        for line in 0..lines {
            let at = |index: usize| if horizontal { line * width + index } else { index * width + line };
            let mut sum: f32 = (0..=radius.min(length - 1)).map(|index| values[at(index)]).sum();
            for index in 0..length {
                scratch[at(index)] = sum / window;
                if index + radius + 1 < length {
                    sum += values[at(index + radius + 1)];
                }
                if index >= radius {
                    sum -= values[at(index - radius)];
                }
            }
        }
        values.copy_from_slice(&scratch);
    }
}

/// How many cells of room the glow needs on every side.
pub fn padding(bloom: Bloom, cell: f32) -> u32 {
    bloom.level().map_or(0, |(blur, _)| 3 * blur_radius(blur, cell) as u32)
}

/// Blurs a mask of `width` by `height` cells, padding included, into a glow in `color`.
pub(crate) fn glow(mut mask: Vec<f32>, width: usize, height: usize, cell: f32, bloom: Bloom, color: Color) -> Image {
    let ink = pixel(color);
    let Some((blur, strength)) = bloom.level() else {
        return paint(width as u32, height as u32, |_, _| Rgba8Pixel::new(0, 0, 0, 0));
    };
    let radius = blur_radius(blur, cell);
    for _ in 0..3 {
        box_blur(&mut mask, width, height, radius);
    }
    paint(width as u32, height as u32, |x, y| {
        let alpha = (mask[y as usize * width + x as usize] * strength * ink.a as f32).round() as u8;
        // Tails this faint add no glow, they only darken the background through rounding when blended.
        Rgba8Pixel::new(ink.r, ink.g, ink.b, if alpha < FAINTEST { 0 } else { alpha })
    })
}

/// A glow around a block of `columns` by `rows` cells, padded on every side.
pub fn rect(columns: u32, rows: u32, cell: f32, bloom: Bloom, color: Color) -> Image {
    let pad = padding(bloom, cell) as usize;
    let (columns, rows) = (columns.max(1) as usize, rows.max(1) as usize);
    let (width, height) = (columns + 2 * pad, rows + 2 * pad);
    let mask = (0..width * height)
        .map(|index| {
            let (x, y) = (index % width, index / width);
            let inside = (pad..pad + columns).contains(&x) && (pad..pad + rows).contains(&y);
            if inside { 1.0 } else { 0.0 }
        })
        .collect();
    glow(mask, width, height, cell, bloom, color)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alpha(image: &Image) -> Vec<u8> {
        image.to_rgba8().unwrap().as_slice().iter().map(|pixel| pixel.a).collect()
    }

    #[test]
    fn off_needs_no_room_and_draws_nothing() {
        assert_eq!(padding(Bloom::Off, 2.0), 0);
        assert!(alpha(&rect(4, 4, 2.0, Bloom::Off, Color::from_rgb_u8(255, 0, 0))).iter().all(|a| *a == 0));
    }

    #[test]
    fn high_reaches_further_than_low() {
        assert!(padding(Bloom::High, 2.0) > padding(Bloom::Low, 2.0));
    }

    #[test]
    fn the_padding_covers_the_whole_blur() {
        assert!(padding(Bloom::High, 2.0) as f32 * 2.0 >= 2.5 * 18.0);
        assert!(padding(Bloom::Low, 3.0) as f32 * 3.0 >= 2.5 * 10.0);
    }

    #[test]
    fn a_block_glows_strongest_in_its_middle_and_fades_to_nothing() {
        let image = rect(6, 10, 2.0, Bloom::High, Color::from_rgb_u8(255, 255, 255));
        let size = image.size();
        let alpha = alpha(&image);
        let at = |x: u32, y: u32| alpha[(y * size.width + x) as usize];
        let (middle_x, middle_y) = (size.width / 2, size.height / 2);
        assert!(at(middle_x, middle_y) > at(middle_x, 1));
        assert!(at(0, 0) <= 2 && at(size.width - 1, size.height - 1) <= 2);
        assert!(alpha.iter().all(|a| *a <= 141), "never stronger than the high strength");
    }
}
