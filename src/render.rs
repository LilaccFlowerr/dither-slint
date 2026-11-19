use slint::{Image, Rgba8Pixel, SharedPixelBuffer};

use crate::Matrix;

pub fn render(
    width: u32,
    height: u32,
    matrix: Matrix,
    ink: Rgba8Pixel,
    paper: Rgba8Pixel,
    tone: impl Fn(u32, u32) -> f32,
) -> Image {
    let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);
    let pixels = buffer.make_mut_slice();
    for y in 0..height {
        for x in 0..width {
            let inked = matrix.is_inked(x as i32, y as i32, tone(x, y));
            pixels[(y * width + x) as usize] = if inked { ink } else { paper };
        }
    }
    Image::from_rgba8(buffer)
}
