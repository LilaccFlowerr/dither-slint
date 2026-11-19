use slint::{Color, Image, Rgba8Pixel, SharedPixelBuffer};

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

fn pixel(color: Color) -> Rgba8Pixel {
    Rgba8Pixel::new(color.red(), color.green(), color.blue(), color.alpha())
}

pub struct GradientOptions {
    pub angle: f32,
    pub from: f32,
    pub to: f32,
    pub matrix: Matrix,
    pub ink: Color,
    pub paper: Color,
}

pub fn linear_gradient(columns: u32, rows: u32, options: &GradientOptions) -> Image {
    let GradientOptions { angle, from, to, matrix, ink, paper } = *options;
    let (columns, rows) = (columns.max(1), rows.max(1));
    let (sin, cos) = angle.to_radians().sin_cos();
    let (dx, dy) = (sin, -cos);
    let length = (columns as f32 * dx).abs() + (rows as f32 * dy).abs();
    let (cx, cy) = (columns as f32 / 2.0, rows as f32 / 2.0);
    render(columns, rows, matrix, pixel(ink), pixel(paper), |x, y| {
        let along = (x as f32 + 0.5 - cx) * dx + (y as f32 + 0.5 - cy) * dy;
        let t = (along / length + 0.5).clamp(0.0, 1.0);
        from + (to - from) * t
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gradient(columns: u32, rows: u32, angle: f32, from: f32, to: f32) -> Image {
        let ink = Color::from_rgb_u8(0, 0, 0);
        let paper = Color::from_rgb_u8(255, 255, 255);
        linear_gradient(columns, rows, &GradientOptions { angle, from, to, matrix: Matrix::Bayer4, ink, paper })
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
}
