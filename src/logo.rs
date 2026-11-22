use slint::{Color, Image, Rgba8Pixel};

use crate::Matrix;
use crate::render::{fit, mix, paint, pixel, shrink};
use crate::style::{FillVariant, texture_inked};

const COLOR_SEAM: f32 = 0.3;
const BAND: f32 = 0.14;
const TRAIL: f32 = 0.35;
const GLINT_WHITE: f32 = 0.65;
const SHIMMER_SLOPE: f32 = 0.3;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogoSource {
    Alpha,
    Dark,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogoTone {
    Mono,
    Original,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogoEffect {
    None,
    Shimmer,
    Scan,
}

pub struct LogoOptions {
    pub source: LogoSource,
    pub tone: LogoTone,
    pub fill: FillVariant,
    pub outline: bool,
    pub floor: f32,
    pub matrix: Matrix,
    pub color: Color,
    pub effect: LogoEffect,
    pub sweep: f32,
}

struct Shape {
    columns: usize,
    inside: Vec<bool>,
    rim: Vec<bool>,
    top: usize,
    bottom: usize,
}

fn coverage(pixel: &[u8], source: LogoSource) -> f32 {
    let alpha = pixel[3] as f32 / 255.0;
    match source {
        LogoSource::Alpha => alpha,
        LogoSource::Dark => {
            let light = (0.2126 * pixel[0] as f32 + 0.7152 * pixel[1] as f32 + 0.0722 * pixel[2] as f32) / 255.0;
            (1.0 - light) * alpha
        }
    }
}

fn color_gap(pixels: &[u8], a: usize, b: usize) -> f32 {
    (0..3).map(|channel| (pixels[a * 4 + channel] as f32 - pixels[b * 4 + channel] as f32).abs()).sum::<f32>() / 255.0
}

fn shape_of(pixels: &[u8], columns: usize, rows: usize, source: LogoSource) -> Shape {
    let inside: Vec<bool> = pixels.as_chunks::<4>().0.iter().map(|p| coverage(p, source) >= 0.5).collect();
    let covered = |x: i32, y: i32| x >= 0 && y >= 0 && (x as usize) < columns && (y as usize) < rows && inside[y as usize * columns + x as usize];
    let seam = |x: i32, y: i32, dx: i32, dy: i32| {
        covered(x + dx, y + dy) && color_gap(pixels, y as usize * columns + x as usize, (y + dy) as usize * columns + (x + dx) as usize) > COLOR_SEAM
    };
    let mut rim = vec![false; inside.len()];
    let (mut top, mut bottom) = (rows, 0);
    for y in 0..rows as i32 {
        for x in 0..columns as i32 {
            if !covered(x, y) {
                continue;
            }
            top = top.min(y as usize);
            bottom = bottom.max(y as usize);
            rim[y as usize * columns + x as usize] =
                !covered(x - 1, y) || !covered(x + 1, y) || !covered(x, y - 1) || !covered(x, y + 1) || seam(x, y, 1, 0) || seam(x, y, 0, 1);
        }
    }
    Shape { columns, inside, rim, top, bottom }
}

fn boost_at(effect: LogoEffect, sweep: f32, x: usize, y: usize, shape: &Shape, height: f32) -> (f32, bool) {
    match effect {
        LogoEffect::None => (0.0, false),
        LogoEffect::Shimmer => {
            let along = x as f32 / shape.columns as f32 - (y as f32 - shape.top as f32) / height * SHIMMER_SLOPE;
            let center = -BAND * 2.0 + sweep * (1.0 + BAND * 4.0);
            ((-((along - center) / BAND).powi(2)).exp(), false)
        }
        LogoEffect::Scan => {
            let row = shape.top as f32 - height * TRAIL + sweep * height * (1.0 + TRAIL * 2.0);
            let behind = row - y as f32;
            let boost = if behind >= 0.0 && behind < height * TRAIL { 1.0 - behind / (height * TRAIL) } else { 0.0 };
            (boost, row.round() == y as f32)
        }
    }
}

fn fill_inks(fill: FillVariant, matrix: Matrix, x: usize, y: usize, density: f32) -> bool {
    match fill {
        FillVariant::Gradient => matrix.is_inked(x as i32, y as i32, density),
        FillVariant::Solid => true,
        FillVariant::Dotted | FillVariant::Hatched => texture_inked(fill, x, y),
    }
}

pub fn logo_width(size: (u32, u32), height: f32) -> f32 {
    size.0 as f32 / size.1.max(1) as f32 * height
}

pub fn row_width(sizes: impl Iterator<Item = (u32, u32)>, height: f32, gap: f32) -> f32 {
    sizes.map(|size| logo_width(size, height) + gap).sum()
}

pub fn logo(source: &Image, columns: u32, rows: u32, options: &LogoOptions) -> Image {
    let Some(buffer) = source.to_rgba8() else { return Image::default() };
    if buffer.width() == 0 || buffer.height() == 0 || columns == 0 || rows == 0 {
        return Image::default();
    }
    let (columns, rows) = fit(buffer.width(), buffer.height(), columns, rows);
    let pixels = shrink(buffer.as_slice(), buffer.width(), buffer.height(), columns, rows);
    let shape = shape_of(&pixels, columns as usize, rows as usize, options.source);
    let height = shape.bottom.saturating_sub(shape.top).max(1) as f32;
    let ink = pixel(options.color);
    let glint = pixel(mix(options.color, Color::from_rgb_u8(255, 255, 255), GLINT_WHITE));
    let clear = Rgba8Pixel::new(0, 0, 0, 0);

    paint(columns, rows, |x, y| {
        let (x, y) = (x as usize, y as usize);
        let index = y * shape.columns + x;
        if !shape.inside[index] {
            return clear;
        }
        let fall = 1.0 - (1.0 - options.floor) * ((y as f32 - shape.top as f32) / height).clamp(0.0, 1.0);
        let (boost, line) = boost_at(options.effect, options.sweep, x, y, &shape, height);
        let density = fall + (1.0 - fall) * boost;
        let inked = (options.outline && shape.rim[index])
            || fill_inks(options.fill, options.matrix, x, y, density)
            || options.matrix.is_inked(x as i32, y as i32, boost);
        if !inked {
            return clear;
        }
        if line {
            return glint;
        }
        match options.tone {
            LogoTone::Mono => ink,
            LogoTone::Original => Rgba8Pixel::new(pixels[index * 4], pixels[index * 4 + 1], pixels[index * 4 + 2], 255),
        }
    })
}

#[cfg(test)]
mod tests {
    use slint::SharedPixelBuffer;

    use super::*;

    fn square() -> Image {
        let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(12, 12);
        for (index, pixel) in buffer.make_mut_slice().iter_mut().enumerate() {
            let (x, y) = (index % 12, index / 12);
            let inside = (2..10).contains(&x) && (2..10).contains(&y);
            *pixel = if inside { Rgba8Pixel::new(10, 20, 30, 255) } else { Rgba8Pixel::new(0, 0, 0, 0) };
        }
        Image::from_rgba8(buffer)
    }

    fn options(fill: FillVariant, outline: bool) -> LogoOptions {
        LogoOptions {
            source: LogoSource::Alpha,
            tone: LogoTone::Mono,
            fill,
            outline,
            floor: 0.3,
            matrix: Matrix::Bayer4,
            color: Color::from_rgb_u8(255, 0, 0),
            effect: LogoEffect::None,
            sweep: 0.0,
        }
    }

    fn inked(image: &Image) -> Vec<bool> {
        image.to_rgba8().unwrap().as_slice().iter().map(|p| p.a > 0).collect()
    }

    #[test]
    fn a_row_is_each_logo_at_its_ratio_plus_a_gap() {
        let sizes = [(100, 50), (30, 30)];
        assert_eq!(logo_width((100, 50), 28.0), 56.0);
        assert_eq!(row_width(sizes.into_iter(), 28.0, 64.0), 56.0 + 64.0 + 28.0 + 64.0);
        assert_eq!(row_width([].into_iter(), 28.0, 64.0), 0.0);
    }

    #[test]
    fn only_the_shape_takes_ink() {
        let cells = inked(&logo(&square(), 12, 12, &options(FillVariant::Solid, true)));
        assert!(cells[6 * 12 + 6]);
        assert!(!cells[0]);
        assert_eq!(cells.iter().filter(|&&on| on).count(), 64);
    }

    #[test]
    fn the_outline_stays_solid_however_open_the_fill() {
        let mut open = options(FillVariant::Gradient, true);
        open.floor = 0.0;
        let cells = inked(&logo(&square(), 12, 12, &open));
        assert!(cells[2 * 12 + 2] && cells[2 * 12 + 9] && cells[9 * 12 + 2] && cells[9 * 12 + 9]);
    }

    #[test]
    fn a_gradient_thins_out_towards_the_bottom() {
        let cells = inked(&logo(&square(), 12, 12, &options(FillVariant::Gradient, false)));
        let row = |y: usize| cells[y * 12..(y + 1) * 12].iter().filter(|&&on| on).count();
        assert!(row(3) > row(8));
    }

    #[test]
    fn dark_source_reads_a_dark_logo_on_an_opaque_background() {
        let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(4, 4);
        for (index, pixel) in buffer.make_mut_slice().iter_mut().enumerate() {
            *pixel = if index % 4 < 2 { Rgba8Pixel::new(0, 0, 0, 255) } else { Rgba8Pixel::new(255, 255, 255, 255) };
        }
        let mut dark = options(FillVariant::Solid, false);
        dark.source = LogoSource::Dark;
        let cells = inked(&logo(&Image::from_rgba8(buffer), 4, 4, &dark));
        assert_eq!(cells.iter().filter(|&&on| on).count(), 8);
    }

    #[test]
    fn the_scan_effect_draws_a_bright_line() {
        let mut scan = options(FillVariant::Solid, false);
        scan.effect = LogoEffect::Scan;
        scan.sweep = 0.5;
        scan.color = Color::from_rgb_u8(255, 0, 0);
        let image = logo(&square(), 12, 12, &scan).to_rgba8().unwrap();
        assert!(image.as_slice().iter().any(|p| p.a > 0 && p.g > 100));
    }
}
