use dither_slint::Matrix;
use dither_slint::render::{self, FillOptions, GradientOptions, linear_gradient};
use dither_slint::diffuse::Method;
use dither_slint::field::{self, Shader};
use dither_slint::glow::{self, Glow, GlowOptions};
use dither_slint::lively::{self, Hover, Idle, Layer, Press};
use dither_slint::style::{FillEdge, FillVariant};
use dither_slint::slint::{Color, Image, Rgba8Pixel, SharedPixelBuffer};
use wasm_bindgen::prelude::*;

fn colors(rgba: &[u8]) -> Vec<Color> {
    rgba.chunks_exact(4).map(|c| Color::from_argb_u8(c[3], c[0], c[1], c[2])).collect()
}

#[wasm_bindgen]
pub fn gradient(columns: u32, rows: u32, angle: f32, from: f32, to: f32, matrix: usize, palette: &[u8], levels: u32) -> Vec<u8> {
    let options = GradientOptions { angle, from, to, matrix: Matrix::ALL[matrix], palette: colors(palette), levels };
    bytes(linear_gradient(columns, rows, &options))
}

#[wasm_bindgen]
pub fn fill(
    columns: u32,
    rows: u32,
    variant: usize,
    edge: usize,
    length: f32,
    from: f32,
    to: f32,
    color: &[u8],
    frame: bool,
) -> Vec<u8> {
    let color = colors(color)[0];
    let options = FillOptions {
        variant: FillVariant::ALL[variant],
        edge: FillEdge::ALL[edge],
        length,
        from,
        to,
        matrix: Matrix::Bayer4,
        color,
        frame,
    };
    bytes(render::fill(columns, rows, &options))
}

#[wasm_bindgen]
pub fn mix(base: &[u8], other: &[u8], amount: f32) -> Vec<u8> {
    let (base, other) = (colors(base)[0], colors(other)[0]);
    let mixed = render::mix(base, other, amount);
    vec![mixed.red(), mixed.green(), mixed.blue(), mixed.alpha()]
}

fn bytes(image: Image) -> Vec<u8> {
    image.to_rgba8().map(|buffer| buffer.as_bytes().to_vec()).unwrap_or_default()
}

fn layer(columns: u32, rows: u32, strength: f32, color: &[u8], matrix: usize) -> Layer {
    Layer::new(columns as i32, rows as i32, strength, colors(color)[0], Matrix::ALL[matrix])
}

#[wasm_bindgen]
pub fn hover(kind: usize, columns: u32, rows: u32, t: f32, x: f32, y: f32, level: f32, strength: f32, color: &[u8], matrix: usize) -> Vec<u8> {
    bytes(lively::hover(Hover::ALL[kind], &layer(columns, rows, strength, color, matrix), t, (x, y), level))
}

#[wasm_bindgen]
pub fn press(kind: usize, columns: u32, rows: u32, t: f32, x: f32, y: f32, seed: i32, strength: f32, color: &[u8], matrix: usize) -> Vec<u8> {
    bytes(lively::press(Press::ALL[kind], &layer(columns, rows, strength, color, matrix), t, (x, y), seed))
}

#[wasm_bindgen]
pub fn idle(kind: usize, columns: u32, rows: u32, t: f32, strength: f32, color: &[u8], cell: f32, matrix: usize) -> Vec<u8> {
    bytes(lively::idle(Idle::ALL[kind], &layer(columns, rows, strength, color, matrix), t, cell))
}

#[wasm_bindgen]
pub fn shader(field: &str, columns: u32, rows: u32, time: f32, params: &[f32], palette: &[u8], matrix: usize, levels: u32) -> Vec<u8> {
    let Some(found) = field::lookup(field) else { return Vec::new() };
    let palette = colors(palette);
    bytes(field::render(&Shader {
        field: found.as_ref(),
        columns,
        rows,
        time,
        params,
        palette: &palette,
        matrix: Matrix::ALL[matrix],
        levels,
    }))
}

#[wasm_bindgen]
pub fn card_glow(columns: u32, rows: u32, corner: usize, reach: f32, strength: f32, boost: f32, rim: bool, radius: f32, variant: usize, matrix: usize, color: &[u8]) -> Vec<u8> {
    let options = GlowOptions {
        glow: Glow::ALL[corner],
        reach,
        strength,
        boost,
        rim,
        radius,
        fill: FillVariant::ALL[variant],
        matrix: Matrix::ALL[matrix],
        color: colors(color)[0],
    };
    bytes(glow::card_glow(columns, rows, &options))
}

#[wasm_bindgen]
pub fn card_sparkles(columns: u32, rows: u32, cell: f32, tick: f32, corner: usize, reach: f32, color: &[u8]) -> Vec<u8> {
    let layer = layer(columns, rows, 1.0, color, 1);
    bytes(lively::glow_sparkles(&layer, cell, tick, Glow::ALL[corner], reach))
}

#[wasm_bindgen]
pub fn picture(pixels: &[u8], width: u32, height: u32, columns: u32, rows: u32, method: usize, threshold: f32, palette: &[u8]) -> Vec<u8> {
    let source = Image::from_rgba8(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(pixels, width, height));
    let options = render::PictureOptions { method: Method::ALL[method], threshold, palette: colors(palette) };
    bytes(render::picture(&source, columns, rows, &options))
}

#[wasm_bindgen]
pub fn rule(
    columns: u32,
    rows: u32,
    vertical: bool,
    fade_start: bool,
    fade_end: bool,
    fade: f32,
    strength: f32,
    variant: usize,
    matrix: usize,
    color: &[u8],
) -> Vec<u8> {
    let options = render::RuleOptions {
        vertical,
        fade_start,
        fade_end,
        fade,
        strength,
        variant: FillVariant::ALL[variant],
        matrix: Matrix::ALL[matrix],
        color: colors(color)[0],
    };
    bytes(render::rule(columns, rows, &options))
}

#[wasm_bindgen]
pub fn slider_fill(columns: u32, rows: u32, end: i32, hot: bool, matrix: usize, color: &[u8]) -> Vec<u8> {
    let options = render::SliderOptions { end, hot, matrix: Matrix::ALL[matrix], color: colors(color)[0] };
    bytes(render::slider_fill(columns, rows, &options))
}
