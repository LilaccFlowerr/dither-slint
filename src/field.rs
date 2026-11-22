use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

use glam::{Vec2, Vec4};
use slint::{Color, Image, Rgba8Pixel, SharedPixelBuffer};

use crate::Matrix;

pub struct Palette(Vec<Vec4>);

impl Palette {
    pub fn new(colors: &[Color]) -> Self {
        let channel = |value: u8| value as f32 / 255.0;
        Self(
            colors
                .iter()
                .map(|c| Vec4::new(channel(c.red()), channel(c.green()), channel(c.blue()), channel(c.alpha())))
                .collect(),
        )
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn entry(&self, index: usize) -> Vec4 {
        self.0[index.min(self.0.len() - 1)]
    }

    pub fn ramp(&self, x: f32) -> Vec4 {
        let last = self.0.len() - 1;
        let scaled = x.clamp(0.0, 1.0) * (last as f32 - 1.0);
        let index = scaled.floor();
        let from = self.entry(1 + index as usize);
        let to = self.entry((2 + index as usize).min(last));
        from.lerp(to, scaled - index)
    }
}

pub struct Cell<'a> {
    pub uv: Vec2,
    pub resolution: Vec2,
    pub time: f32,
    pub params: &'a [f32],
    pub palette: &'a Palette,
}

/// What every cell of one frame shares.
pub struct Frame<'a> {
    pub resolution: Vec2,
    pub time: f32,
    pub params: &'a [f32],
}

pub trait Field: Send + Sync {
    fn tone(&self, cell: &Cell) -> f32;

    fn color(&self, _cell: &Cell) -> Option<Vec4> {
        None
    }

    /// This field for one frame, for a field that can work out shared values once
    /// instead of in every cell. `None` renders every cell with the field itself.
    fn prepare(&self, _frame: &Frame) -> Option<Box<dyn Field + '_>> {
        None
    }
}

impl<F: Fn(&Cell) -> f32 + Send + Sync> Field for F {
    fn tone(&self, cell: &Cell) -> f32 {
        self(cell)
    }
}

type Registry = RwLock<HashMap<String, Arc<dyn Field>>>;

fn registry() -> &'static Registry {
    static FIELDS: OnceLock<Registry> = OnceLock::new();
    FIELDS.get_or_init(|| RwLock::new(crate::backgrounds::all()))
}

pub fn register(name: &str, field: impl Field + 'static) {
    registry().write().unwrap().insert(name.to_string(), Arc::new(field));
}

pub fn lookup(name: &str) -> Option<Arc<dyn Field>> {
    registry().read().unwrap().get(name).cloned()
}

pub struct Shader<'a> {
    pub field: &'a dyn Field,
    pub columns: u32,
    pub rows: u32,
    pub time: f32,
    pub params: &'a [f32],
    pub palette: &'a [Color],
    pub matrix: Matrix,
    pub levels: u32,
}

const MAX_THREADS: usize = 8;

fn to_pixel(color: Vec4) -> Rgba8Pixel {
    let [r, g, b, a] = (color.clamp(Vec4::ZERO, Vec4::ONE) * 255.0).round().to_array();
    Rgba8Pixel::new(r as u8, g as u8, b as u8, a as u8)
}

fn shade(shader: &Shader, palette: &Palette, steps: f32, x: u32, y: u32) -> Rgba8Pixel {
    let (columns, rows) = (shader.columns as f32, shader.rows as f32);
    let up = shader.rows - 1 - y;
    let cell = Cell {
        uv: Vec2::new((x as f32 + 0.5) / columns, (up as f32 + 0.5) / rows),
        resolution: Vec2::new(columns, rows),
        time: shader.time,
        params: shader.params,
        palette,
    };
    let tone = shader.field.tone(&cell).clamp(0.0, 1.0);
    let tone = (tone * steps + 0.5).floor() / steps;
    let (at_x, at_y) = (x as i32, up as i32);

    let color = if shader.matrix.is_inked(at_x, at_y, tone) {
        shader.field.color(&cell)
    } else {
        None
    };
    let color = color.unwrap_or_else(|| {
        palette.entry(shader.matrix.palette_index(at_x, at_y, tone, palette.len()))
    });
    to_pixel(color)
}

pub fn render(shader: &Shader) -> Image {
    let palette = Palette::new(shader.palette);
    let (columns, rows) = (shader.columns.max(1) as usize, shader.rows.max(1) as usize);
    let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(columns as u32, rows as u32);
    if palette.len() < 2 {
        return Image::from_rgba8(buffer);
    }
    let frame = Frame { resolution: Vec2::new(columns as f32, rows as f32), time: shader.time, params: shader.params };
    let prepared = shader.field.prepare(&frame);
    let field = prepared.as_deref().unwrap_or(shader.field);
    let shader = Shader { field, columns: columns as u32, rows: rows as u32, ..*shader };

    let levels = if shader.levels == 0 { shader.matrix.map().steps as u32 + 1 } else { shader.levels };
    let steps = (levels.max(2) - 1) as f32;

    let threads = std::thread::available_parallelism().map_or(1, |n| n.get()).min(MAX_THREADS);
    let band = rows.div_ceil(threads);
    let fill = |index: usize, chunk: &mut [Rgba8Pixel]| {
        for (offset, pixel) in chunk.iter_mut().enumerate() {
            let (x, y) = ((offset % columns) as u32, (index * band + offset / columns) as u32);
            *pixel = shade(&shader, &palette, steps, x, y);
        }
    };

    let chunks = buffer.make_mut_slice().chunks_mut(band * columns);
    if threads == 1 {
        chunks.enumerate().for_each(|(index, chunk)| fill(index, chunk));
    } else {
        std::thread::scope(|scope| {
            for (index, chunk) in chunks.enumerate() {
                let fill = &fill;
                scope.spawn(move || fill(index, chunk));
            }
        });
    }
    Image::from_rgba8(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLACK: Color = Color::from_rgb_u8(0, 0, 0);
    const WHITE: Color = Color::from_rgb_u8(255, 255, 255);

    fn shader<'a>(field: &'a dyn Field, palette: &'a [Color], params: &'a [f32]) -> Shader<'a> {
        Shader { field, columns: 32, rows: 16, time: 0.0, params, palette, matrix: Matrix::Bayer4, levels: 0 }
    }

    fn lit_per_row(image: &Image) -> Vec<usize> {
        let buffer = image.to_rgba8().unwrap();
        let width = buffer.width() as usize;
        buffer.as_slice().chunks(width).map(|row| row.iter().filter(|p| p.r > 128).count()).collect()
    }

    #[test]
    fn uv_points_up() {
        let field = |cell: &Cell| cell.uv.y;
        let rows = lit_per_row(&render(&shader(&field, &[BLACK, WHITE], &[])));
        assert!(rows[0] > rows[8] && rows[8] > rows[15]);
    }

    #[test]
    fn a_field_gets_its_params_and_the_time() {
        let field = |cell: &Cell| cell.params[0] * cell.time;
        let mut input = shader(&field, &[BLACK, WHITE], &[0.5]);
        assert_eq!(lit_per_row(&render(&input)).iter().sum::<usize>(), 0);
        input.time = 2.0;
        assert_eq!(lit_per_row(&render(&input)).iter().sum::<usize>(), 32 * 16);
    }

    #[test]
    fn three_colours_use_the_middle_one() {
        let field = |_: &Cell| 0.5;
        let red = Color::from_rgb_u8(255, 0, 0);
        let image = render(&shader(&field, &[BLACK, red, WHITE], &[])).to_rgba8().unwrap();
        assert!(image.as_slice().iter().all(|p| p.r == 255 && (p.g == 0 || p.g == 255)));
    }

    struct Rainbow;

    impl Field for Rainbow {
        fn tone(&self, _: &Cell) -> f32 {
            0.5
        }

        fn color(&self, cell: &Cell) -> Option<Vec4> {
            Some(Vec4::new(cell.uv.x, 0.0, 0.0, 1.0))
        }
    }

    #[test]
    fn a_colour_field_paints_the_dots_and_leaves_the_rest_paper() {
        let image = render(&shader(&Rainbow, &[BLACK, WHITE], &[])).to_rgba8().unwrap();
        let (dots, paper): (Vec<&Rgba8Pixel>, Vec<&Rgba8Pixel>) =
            image.as_slice().iter().partition(|p| p.r > 0);
        assert!(!dots.is_empty() && !paper.is_empty());
        assert!(paper.iter().all(|p| p.g == 0 && p.b == 0));
        assert!(dots.iter().all(|p| p.g == 0 && p.b == 0));
    }

    #[test]
    fn fields_can_be_registered_by_name() {
        register("test-flat", |_: &Cell| 1.0);
        assert!(lookup("test-flat").is_some());
        assert!(lookup("nothing-here").is_none());
        assert!(lookup("ripples").is_some());
    }

    #[test]
    fn fewer_than_two_colours_draw_nothing() {
        let field = |_: &Cell| 1.0;
        let image = render(&shader(&field, &[WHITE], &[])).to_rgba8().unwrap();
        assert!(image.as_slice().iter().all(|p| p.a == 0));
    }
}
