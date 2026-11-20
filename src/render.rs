use slint::{Color, Image, Rgba8Pixel, SharedPixelBuffer};

use crate::Matrix;
use crate::matrix::quantize;
use crate::color::mix_oklab;
use crate::diffuse::{Method, Options, dither_image};
use crate::style::{FillEdge, FillVariant, texture_inked};

pub(crate) fn paint(width: u32, height: u32, pixel_at: impl Fn(u32, u32) -> Rgba8Pixel) -> Image {
    let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);
    let pixels = buffer.make_mut_slice();
    for y in 0..height {
        for x in 0..width {
            pixels[(y * width + x) as usize] = pixel_at(x, y);
        }
    }
    Image::from_rgba8(buffer)
}

pub fn render(
    width: u32,
    height: u32,
    matrix: Matrix,
    palette: &[Rgba8Pixel],
    levels: u32,
    tone: impl Fn(u32, u32) -> f32,
) -> Image {
    if palette.is_empty() {
        return paint(width, height, |_, _| Rgba8Pixel::new(0, 0, 0, 0));
    }
    paint(width, height, |x, y| {
        let tone = quantize(tone(x, y), levels);
        palette[matrix.palette_index(x as i32, y as i32, tone, palette.len())]
    })
}

pub(crate) fn pixel(color: Color) -> Rgba8Pixel {
    Rgba8Pixel::new(color.red(), color.green(), color.blue(), color.alpha())
}

pub fn mix(base: Color, other: Color, amount: f32) -> Color {
    let rgba = |c: Color| [c.red(), c.green(), c.blue(), c.alpha()].map(|v| v as f32 / 255.0);
    let [r, g, b, a] = mix_oklab(rgba(base), rgba(other), amount).map(|v| (v * 255.0).round() as u8);
    Color::from_argb_u8(a, r, g, b)
}

pub struct FillOptions {
    pub variant: FillVariant,
    pub edge: FillEdge,
    pub length: f32,
    pub from: f32,
    pub to: f32,
    pub matrix: Matrix,
    pub color: Color,
    pub frame: bool,
}

fn frame_alpha(y: u32, rows: u32) -> f32 {
    let at = (y as f32 + 0.5) / rows as f32;
    match at {
        at if at <= 0.30 => 1.0,
        at if at < 0.34 => (0.34 - at) / 0.04,
        at if at <= 0.66 => 0.0,
        at if at < 0.70 => (at - 0.66) / 0.04,
        _ => 1.0,
    }
}

pub fn fill(columns: u32, rows: u32, options: &FillOptions) -> Image {
    let FillOptions { variant, edge, length, from, to, matrix, color, frame } = *options;
    let (columns, rows) = (columns.max(1), rows.max(1));
    let fade = length.round().max(1.0);
    let inked = |x: u32, y: u32| match variant {
        FillVariant::Gradient => {
            let (step, mx, my) = match edge {
                FillEdge::Top => (y, x as i32, y as i32),
                FillEdge::Left => (x, x as i32, y as i32),
                FillEdge::Bottom => (rows - 1 - y, x as i32, y as i32 - rows as i32),
                FillEdge::Right => (columns - 1 - x, x as i32 - columns as i32, y as i32),
            };
            let step = step as f32;
            let level = if step >= fade { to } else { from + (to - from) * ((step + 0.5) / fade) };
            matrix.is_inked(mx, my, level.max(to))
        }
        textured => texture_inked(textured, x as usize, y as usize),
    };
    let clear = Rgba8Pixel::new(0, 0, 0, 0);
    paint(columns, rows, |x, y| {
        if !inked(x, y) {
            return clear;
        }
        let alpha = if frame { frame_alpha(y, rows) } else { 1.0 };
        let mut ink = pixel(color);
        ink.a = (ink.a as f32 * alpha).round() as u8;
        ink
    })
}

const RULE_STEPS: usize = 10;

pub struct RuleOptions {
    pub vertical: bool,
    pub fade_start: bool,
    pub fade_end: bool,
    pub fade: f32,
    pub strength: f32,
    pub variant: FillVariant,
    pub matrix: Matrix,
    pub color: Color,
}

fn rule_level(percent: f32, options: &RuleOptions) -> usize {
    let ramps = options.fade_start as u8 + options.fade_end as u8;
    if ramps == 0 {
        return RULE_STEPS;
    }
    let ramp = (options.fade * 50.0).min(100.0 / ramps as f32);
    let step = ramp / RULE_STEPS as f32;
    let index = |from: f32| (((percent - from) / step) as usize).min(RULE_STEPS - 1);
    if options.fade_start && percent < ramp {
        index(0.0)
    } else if options.fade_end && percent >= 100.0 - ramp {
        RULE_STEPS - 1 - index(100.0 - ramp)
    } else {
        RULE_STEPS
    }
}

pub fn rule(columns: u32, rows: u32, options: &RuleOptions) -> Image {
    let (columns, rows) = (columns.max(1), rows.max(1));
    let ink = pixel(options.color);
    paint(columns, rows, |x, y| {
        let along = if options.vertical { (y as f32 + 0.5) / rows as f32 } else { (x as f32 + 0.5) / columns as f32 };
        let level = rule_level(along * 100.0, options);
        let tone = options.strength * ((level + 1) as f32 / (RULE_STEPS + 1) as f32).powf(1.6);
        let (cx, cy) = (x as i32, y as i32);
        match options.variant {
            FillVariant::Gradient if options.matrix.is_inked(cx, cy, tone) => ink,
            FillVariant::Gradient => Rgba8Pixel::new(0, 0, 0, 0),
            textured if texture_inked(textured, x as usize, y as usize) => {
                Rgba8Pixel::new(ink.r, ink.g, ink.b, (ink.a as f32 * tone).round() as u8)
            }
            _ => Rgba8Pixel::new(0, 0, 0, 0),
        }
    })
}

pub struct SliderOptions {
    pub end: i32,
    pub hot: bool,
    pub matrix: Matrix,
    pub color: Color,
}

pub fn slider_fill(columns: u32, rows: u32, options: &SliderOptions) -> Image {
    let (columns, rows) = (columns.max(1), rows.max(1));
    let (floor, length) = if options.hot { (5.0 / 16.0, 60.0) } else { (2.0 / 16.0, 30.0) };
    let (ink, clear) = (pixel(options.color), Rgba8Pixel::new(0, 0, 0, 0));
    paint(columns, rows, |x, y| {
        let corner = (x == 0 || x + 1 == columns) && (y == 0 || y + 1 == rows);
        if corner || x as i32 >= options.end {
            return clear;
        }
        let distance = (options.end - 1 - x as i32) as f32;
        let level = if distance < 2.0 { 1.0 } else { (1.0 - (distance - 1.5) / length * (1.0 - floor)).max(floor) };
        if options.matrix.is_inked(x as i32, y as i32, level) { ink } else { clear }
    })
}

pub struct GradientOptions {
    pub angle: f32,
    pub from: f32,
    pub to: f32,
    pub matrix: Matrix,
    pub palette: Vec<Color>,
    pub levels: u32,
}

pub fn linear_gradient(columns: u32, rows: u32, options: &GradientOptions) -> Image {
    let GradientOptions { angle, from, to, matrix, palette, levels } = options;
    let (angle, from, to, matrix, levels) = (*angle, *from, *to, *matrix, *levels);
    let palette: Vec<Rgba8Pixel> = palette.iter().copied().map(pixel).collect();
    let (columns, rows) = (columns.max(1), rows.max(1));
    let (sin, cos) = angle.to_radians().sin_cos();
    let (dx, dy) = (sin, -cos);
    let length = (columns as f32 * dx).abs() + (rows as f32 * dy).abs();
    let (cx, cy) = (columns as f32 / 2.0, rows as f32 / 2.0);
    render(columns, rows, matrix, &palette, levels, |x, y| {
        let along = (x as f32 + 0.5 - cx) * dx + (y as f32 + 0.5 - cy) * dy;
        let t = (along / length + 0.5).clamp(0.0, 1.0);
        from + (to - from) * t
    })
}

pub struct PictureOptions {
    pub method: Method,
    pub threshold: f32,
    pub palette: Vec<Color>,
}

fn shrink(source: &[Rgba8Pixel], width: u32, height: u32, columns: u32, rows: u32) -> Vec<u8> {
    let mut out = Vec::with_capacity((columns * rows * 4) as usize);
    for ty in 0..rows {
        let (y0, y1) = (ty * height / rows, ((ty + 1) * height / rows).max(ty * height / rows + 1));
        for tx in 0..columns {
            let (x0, x1) = (tx * width / columns, ((tx + 1) * width / columns).max(tx * width / columns + 1));
            let mut sum = [0u32; 4];
            for y in y0..y1 {
                for x in x0..x1 {
                    let p = source[(y * width + x) as usize];
                    for (total, value) in sum.iter_mut().zip([p.r, p.g, p.b, p.a]) {
                        *total += value as u32;
                    }
                }
            }
            let count = (y1 - y0) * (x1 - x0);
            out.extend(sum.map(|total| (total / count) as u8));
        }
    }
    out
}

pub fn picture(source: &Image, columns: u32, rows: u32, options: &PictureOptions) -> Image {
    let blank = || Image::from_rgba8(SharedPixelBuffer::<Rgba8Pixel>::new(1, 1));
    let Some(buffer) = source.to_rgba8() else { return blank() };
    let (width, height) = (buffer.width(), buffer.height());
    if width == 0 || height == 0 || columns == 0 || rows == 0 {
        return blank();
    }
    let scale = (columns as f32 / width as f32).min(rows as f32 / height as f32);
    let columns = ((width as f32 * scale).round() as u32).max(1);
    let rows = ((height as f32 * scale).round() as u32).max(1);

    let small = shrink(buffer.as_slice(), width, height, columns, rows);
    let rgb = |c: Color| [c.red(), c.green(), c.blue()].map(|v| v as f32 / 255.0);
    let dither = Options {
        method: options.method,
        palette: options.palette.iter().copied().map(rgb).collect(),
        threshold: options.threshold,
        serpentine: true,
    };
    let bytes = dither_image(&small, columns as usize, rows as usize, &dither);
    Image::from_rgba8(SharedPixelBuffer::clone_from_slice(&bytes, columns, rows))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gradient(columns: u32, rows: u32, angle: f32, from: f32, to: f32) -> Image {
        let palette = vec![Color::from_rgb_u8(255, 255, 255), Color::from_rgb_u8(0, 0, 0)];
        linear_gradient(columns, rows, &GradientOptions { angle, from, to, matrix: Matrix::Bayer4, palette, levels: 0 })
    }

    fn inked_columns(image: &Image) -> Vec<usize> {
        let buffer = image.to_rgba8().unwrap();
        let width = buffer.width() as usize;
        let mut counts = vec![0; width];
        for (index, pixel) in buffer.as_slice().iter().enumerate() {
            if pixel.r == 0 {
                counts[index % width] += 1;
            }
        }
        counts
    }

    fn layer(variant: FillVariant, edge: FillEdge, from: f32, frame: bool) -> Vec<Rgba8Pixel> {
        let color = Color::from_rgb_u8(255, 0, 0);
        let options = FillOptions { variant, edge, length: 4.0, from, to: 0.0, matrix: Matrix::Bayer4, color, frame };
        fill(8, 10, &options).to_rgba8().unwrap().as_slice().to_vec()
    }

    fn inked_in_row(pixels: &[Rgba8Pixel], y: usize) -> usize {
        pixels[y * 8..(y + 1) * 8].iter().filter(|p| p.a > 0).count()
    }

    #[test]
    fn gradient_is_dense_at_its_edge() {
        let pixels = layer(FillVariant::Gradient, FillEdge::Bottom, 1.0, false);
        assert!(inked_in_row(&pixels, 9) > inked_in_row(&pixels, 7));
        assert!(inked_in_row(&pixels, 9) >= 6);
        assert_eq!(inked_in_row(&pixels, 0), 0);
    }

    #[test]
    fn frame_clears_the_label_band() {
        let pixels = layer(FillVariant::Solid, FillEdge::Bottom, 1.0, true);
        assert_eq!(inked_in_row(&pixels, 0), 8);
        assert!(pixels[5 * 8..6 * 8].iter().all(|p| p.a == 0));
        assert_eq!(pixels[9 * 8].a, 255);
    }

    #[test]
    fn textures_tile_every_four_cells() {
        let pixels = layer(FillVariant::Hatched, FillEdge::Bottom, 1.0, false);
        assert_eq!(pixels[3].a, 255);
        assert_eq!(pixels[7].a, 255);
        assert_eq!(pixels[0].a, 0);
    }

    #[test]
    fn keeps_the_cell_grid_size() {
        let image = gradient(10, 4, 180.0, 0.0, 1.0);
        assert_eq!((image.size().width, image.size().height), (10, 4));
    }

    #[test]
    fn empty_box_gets_one_cell() {
        let image = gradient(0, 0, 180.0, 0.0, 1.0);
        assert_eq!((image.size().width, image.size().height), (1, 1));
    }

    #[test]
    fn flat_tone_matches_the_matrix() {
        let counts = inked_columns(&gradient(4, 4, 180.0, 0.5, 0.5));
        assert_eq!(counts.iter().sum::<usize>(), 8);
    }

    #[test]
    fn ninety_degrees_runs_left_to_right() {
        let counts = inked_columns(&gradient(16, 16, 90.0, 0.0, 1.0));
        assert_eq!(counts[0], 0);
        assert_eq!(counts[15], 16);
    }

    fn flat(width: u32, height: u32, grey: u8) -> Image {
        let pixels = [grey, grey, grey, 255].repeat((width * height) as usize);
        Image::from_rgba8(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(&pixels, width, height))
    }

    fn picture_of(source: &Image, columns: u32, rows: u32) -> Image {
        let options = PictureOptions {
            method: Method::FloydSteinberg,
            threshold: 0.5,
            palette: vec![Color::from_rgb_u8(0, 0, 0), Color::from_rgb_u8(255, 255, 255)],
        };
        picture(source, columns, rows, &options)
    }

    #[test]
    fn picture_keeps_the_aspect_ratio() {
        let image = picture_of(&flat(100, 50, 128), 20, 20);
        assert_eq!((image.size().width, image.size().height), (20, 10));
    }

    #[test]
    fn white_picture_is_all_ink_and_black_all_paper() {
        let white = picture_of(&flat(8, 8, 255), 8, 8).to_rgba8().unwrap();
        let black = picture_of(&flat(8, 8, 0), 8, 8).to_rgba8().unwrap();
        assert!(white.as_slice().iter().all(|p| p.r == 255));
        assert!(black.as_slice().iter().all(|p| p.r == 0));
    }

    #[test]
    fn a_three_colour_palette_runs_through_the_middle_colour() {
        let palette = vec![Color::from_rgb_u8(0, 0, 0), Color::from_rgb_u8(255, 0, 0), Color::from_rgb_u8(255, 255, 255)];
        let options = GradientOptions { angle: 90.0, from: 0.0, to: 1.0, matrix: Matrix::Bayer4, palette, levels: 0 };
        let image = linear_gradient(32, 4, &options).to_rgba8().unwrap();
        let at = |x: usize, y: usize| image.as_slice()[y * 32 + x];
        assert!((0..4).all(|y| at(0, y).g == 0 && at(0, y).r == 0));
        assert!((0..4).all(|y| at(31, y).g == 255));
        assert!((0..32).any(|x| at(x, 0).r == 255 && at(x, 0).g == 0));
    }

    fn rule_of(columns: u32, options: RuleOptions) -> Vec<Rgba8Pixel> {
        rule(columns, 1, &options).to_rgba8().unwrap().as_slice().to_vec()
    }

    fn faded(strength: f32) -> RuleOptions {
        RuleOptions {
            vertical: false,
            fade_start: true,
            fade_end: true,
            fade: 0.6,
            strength,
            variant: FillVariant::Gradient,
            matrix: Matrix::Bayer2,
            color: Color::from_rgb_u8(255, 0, 0),
        }
    }

    #[test]
    fn a_rule_is_solid_in_the_middle_and_thins_towards_both_tips() {
        let pixels = rule_of(100, faded(1.0));
        let inked = |range: std::ops::Range<usize>| pixels[range].iter().filter(|p| p.a > 0).count();
        assert!(inked(40..60) > inked(0..10));
        assert!(inked(40..60) > inked(90..100));
        assert_eq!(inked(0..2), 0);
    }

    #[test]
    fn a_rule_without_fade_keeps_the_same_density_along_its_length() {
        let options = RuleOptions { fade_start: false, fade_end: false, ..faded(1.0) };
        let pixels = rule_of(100, options);
        assert_eq!(pixels[..50].iter().filter(|p| p.a > 0).count(), pixels[50..].iter().filter(|p| p.a > 0).count());
    }

    #[test]
    fn the_slider_fill_is_solid_at_the_thumb_and_thins_away_from_it() {
        let options = SliderOptions { end: 60, hot: false, matrix: Matrix::Bayer4, color: Color::from_rgb_u8(255, 0, 0) };
        let image = slider_fill(100, 8, &options).to_rgba8().unwrap();
        let inked = |x: usize| (1..7).filter(|&y| image.as_slice()[y * 100 + x].a > 0).count();
        assert_eq!(inked(59), 6);
        assert!(inked(58) == 6);
        assert!(inked(30) < inked(55));
        assert_eq!(inked(60), 0);
    }
}
