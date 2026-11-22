use dither_slint::field::{self, Cell};
use dither_slint::glam::Vec2;
use dither_slint::noise::fbm;
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};

#[allow(dead_code)]
pub const PAGES: [&str; 25] =
    ["gradient", "button", "badge", "card", "image", "backgrounds", "motion", "divider", "switch", "slider", "input", "avatar", "loader", "skeleton", "empty", "progress", "stat", "sparkline", "uptime", "gauge", "number", "image-compare", "scroll-reveal", "logo", "treemap"];

pub fn sample() -> Image {
    let (width, height) = (200u32, 130u32);
    let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);
    for (index, pixel) in buffer.make_mut_slice().iter_mut().enumerate() {
        let (x, y) = ((index as u32 % width) as f32, (index as u32 / width) as f32);
        let sky = y / height as f32;
        let sun = ((x - 130.0).hypot(y - 45.0) / 28.0).min(1.0);
        let light = (0.15 + 0.5 * (1.0 - sky)) * sun + (1.0 - sun);
        let hill = (y > 95.0 + 12.0 * (x / 35.0).sin()) as u8 as f32;
        let value = (light * (1.0 - hill) + 0.12 * hill) * 255.0;
        *pixel = Rgba8Pixel::new(value as u8, (value * 0.85) as u8, (value * 0.7) as u8, 255);
    }
    Image::from_rgba8(buffer)
}

pub fn register_fields() {
    field::register("rings", |cell: &Cell| {
        let aspect = cell.resolution.x / cell.resolution.y;
        let p = Vec2::new(cell.uv.x * aspect, cell.uv.y);
        let drop = Vec2::new(cell.params[0] * aspect, 1.0 - cell.params[1]);
        let wobble = fbm(p * 2.2 + Vec2::new(cell.time * 0.03, 0.0)) - 0.5;
        let d = p.distance(drop) + wobble * 0.08;
        let rings = 0.5 + 0.5 * (d * cell.params[2] - cell.time * 0.9).cos();
        cell.params[3] * rings * (-d * 2.6).exp()
    });
}
