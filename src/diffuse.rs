use crate::Matrix;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Method {
    Bayer4,
    Bayer8,
    BlueNoise,
    FloydSteinberg,
    Atkinson,
    SierraLite,
}

pub type PaletteColor = [f32; 3];

pub struct Options {
    pub method: Method,
    pub palette: Vec<PaletteColor>,
    pub threshold: f32,
    pub serpentine: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            method: Method::FloydSteinberg,
            palette: vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]],
            threshold: 0.5,
            serpentine: true,
        }
    }
}

type Kernel = &'static [(i32, i32, f32)];

const ATKINSON: Kernel = &[
    (1, 0, 1.0 / 8.0),
    (2, 0, 1.0 / 8.0),
    (-1, 1, 1.0 / 8.0),
    (0, 1, 1.0 / 8.0),
    (1, 1, 1.0 / 8.0),
    (0, 2, 1.0 / 8.0),
];

const FLOYD_STEINBERG: Kernel = &[
    (1, 0, 7.0 / 16.0),
    (-1, 1, 3.0 / 16.0),
    (0, 1, 5.0 / 16.0),
    (1, 1, 1.0 / 16.0),
];

const SIERRA_LITE: Kernel = &[(1, 0, 2.0 / 4.0), (-1, 1, 1.0 / 4.0), (0, 1, 1.0 / 4.0)];

fn nearest(palette: &[PaletteColor], r: f32, g: f32, b: f32) -> usize {
    let mut best = 0;
    let mut best_distance = f32::INFINITY;
    for (index, color) in palette.iter().enumerate() {
        let dr = r - color[0];
        let dg = g - color[1];
        let db = b - color[2];
        let distance = 0.3 * dr * dr + 0.59 * dg * dg + 0.11 * db * db;
        if distance < best_distance {
            best_distance = distance;
            best = index;
        }
    }
    best
}

fn write(out: &mut [u8], offset: usize, color: PaletteColor, alpha: u8) {
    for channel in 0..3 {
        out[offset + channel] = (color[channel] * 255.0).round_ties_even().clamp(0.0, 255.0) as u8;
    }
    out[offset + 3] = alpha;
}

fn dither_ordered(
    data: &[u8],
    width: usize,
    height: usize,
    matrix: Matrix,
    palette: &[PaletteColor],
    bias: f32,
) -> Vec<u8> {
    let mut out = vec![0u8; data.len()];
    let map = matrix.map();
    let spread = 1.0 / (palette.len() - 1) as f32;
    for y in 0..height {
        for x in 0..width {
            let offset = (y * width + x) * 4;
            let threshold = map.thresholds[(y % map.size) * map.size + (x % map.size)];
            let shift = (0.5 - threshold) * spread + bias;
            let r = data[offset] as f32 / 255.0 + shift;
            let g = data[offset + 1] as f32 / 255.0 + shift;
            let b = data[offset + 2] as f32 / 255.0 + shift;
            let color = palette[nearest(palette, r, g, b)];
            write(&mut out, offset, color, data[offset + 3]);
        }
    }
    out
}

fn dither_diffusion(
    data: &[u8],
    width: usize,
    height: usize,
    kernel: Kernel,
    palette: &[PaletteColor],
    bias: f32,
    serpentine: bool,
) -> Vec<u8> {
    let mut out = vec![0u8; data.len()];
    let mut work = vec![0f32; width * height * 3];
    for index in 0..width * height {
        for channel in 0..3 {
            work[index * 3 + channel] = data[index * 4 + channel] as f32 / 255.0 + bias;
        }
    }

    for y in 0..height {
        let reverse = serpentine && y % 2 == 1;
        for step in 0..width {
            let x = if reverse { width - 1 - step } else { step };
            let index = y * width + x;
            let r = work[index * 3];
            let g = work[index * 3 + 1];
            let b = work[index * 3 + 2];
            let color = palette[nearest(palette, r, g, b)];
            write(&mut out, index * 4, color, data[index * 4 + 3]);

            let error = [r - color[0], g - color[1], b - color[2]];
            for &(dx, dy, weight) in kernel {
                let nx = if reverse { x as i32 - dx } else { x as i32 + dx };
                let ny = y as i32 + dy;
                if nx < 0 || nx >= width as i32 || ny >= height as i32 {
                    continue;
                }
                let target = (ny as usize * width + nx as usize) * 3;
                for channel in 0..3 {
                    work[target + channel] += error[channel] * weight;
                }
            }
        }
    }
    out
}

pub fn dither_image(data: &[u8], width: usize, height: usize, options: &Options) -> Vec<u8> {
    let palette = &options.palette;
    let bias = 0.5 - options.threshold;
    match options.method {
        Method::Bayer4 => dither_ordered(data, width, height, Matrix::Bayer4, palette, bias),
        Method::Bayer8 => dither_ordered(data, width, height, Matrix::Bayer8, palette, bias),
        Method::BlueNoise => dither_ordered(data, width, height, Matrix::BlueNoise, palette, bias),
        Method::FloydSteinberg => {
            dither_diffusion(data, width, height, FLOYD_STEINBERG, palette, bias, options.serpentine)
        }
        Method::Atkinson => {
            dither_diffusion(data, width, height, ATKINSON, palette, bias, options.serpentine)
        }
        Method::SierraLite => {
            dither_diffusion(data, width, height, SIERRA_LITE, palette, bias, options.serpentine)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Method; 6] = [
        Method::Bayer4,
        Method::Bayer8,
        Method::BlueNoise,
        Method::FloydSteinberg,
        Method::Atkinson,
        Method::SierraLite,
    ];

    fn flat(value: u8, width: usize, height: usize) -> Vec<u8> {
        [value, value, value, 255].repeat(width * height)
    }

    fn white_pixels(data: &[u8]) -> usize {
        data.chunks(4).filter(|pixel| pixel[0] == 255).count()
    }

    fn options(method: Method) -> Options {
        Options { method, ..Options::default() }
    }

    #[test]
    fn black_stays_black_and_white_stays_white() {
        for method in ALL {
            let black = dither_image(&flat(0, 16, 16), 16, 16, &options(method));
            let white = dither_image(&flat(255, 16, 16), 16, 16, &options(method));
            assert_eq!(white_pixels(&black), 0, "{method:?}");
            assert_eq!(white_pixels(&white), 256, "{method:?}");
        }
    }

    #[test]
    fn mid_grey_is_about_half_white() {
        for method in [Method::Bayer4, Method::Bayer8, Method::FloydSteinberg] {
            let out = dither_image(&flat(128, 32, 32), 32, 32, &options(method));
            let white = white_pixels(&out);
            assert!((480..=544).contains(&white), "{method:?}: {white}");
        }
    }
}
