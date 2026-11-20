use slint::{Color, Image, Rgba8Pixel};

use crate::Matrix;
use crate::render::{paint, pixel};

pub const COLUMNS: u32 = 112;
pub const ROWS: u32 = 72;
const CX: f32 = COLUMNS as f32 / 2.0;
const CY: f32 = 32.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Illustration {
    Box,
    Search,
    Folder,
    Planet,
}

impl Illustration {
    pub const ALL: [Self; 4] = [Self::Box, Self::Search, Self::Folder, Self::Planet];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Layer {
    Object,
    Backdrop,
}

type Point = (f32, f32);
type Shade = (f32, u8);

fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

fn in_polygon(x: f32, y: f32, points: &[Point]) -> bool {
    let mut inside = false;
    let mut previous = points.len() - 1;
    for (index, &(xi, yi)) in points.iter().enumerate() {
        let (xj, yj) = points[previous];
        if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
            inside = !inside;
        }
        previous = index;
    }
    inside
}

fn shift((x, y): Point, dx: f32, dy: f32) -> Point {
    (x + dx, y + dy)
}

fn face(tone: f32, y: f32, top: f32, height: f32) -> f32 {
    clamp01(tone * (1.15 - 0.35 * (y - top) / height))
}

fn in_rect(x: f32, y: f32, (left, top): Point, (width, height): Point) -> bool {
    x >= left && x < left + width && y >= top && y < top + height
}

fn to_segment(x: f32, y: f32, a: Point, b: Point) -> f32 {
    let length = (b.0 - a.0).powi(2) + (b.1 - a.1).powi(2);
    let t = clamp01(((x - a.0) * (b.0 - a.0) + (y - a.1) * (b.1 - a.1)) / length);
    (x - (a.0 + t * (b.0 - a.0))).hypot(y - (a.1 + t * (b.1 - a.1)))
}

fn open_box(x: f32, y: f32) -> Option<Shade> {
    let top = (CX, CY - 14.0);
    let left = (CX - 26.0, CY - 1.0);
    let right = (CX + 26.0, CY - 1.0);
    let front = (CX, CY + 12.0);
    let (depth, lift) = (20.0, 11.0);
    let flaps = [
        ([top, left, shift(left, -9.0, -lift), shift(top, -9.0, -lift)], 0.72),
        ([top, right, shift(right, 9.0, -lift), shift(top, 9.0, -lift)], 0.42),
    ];
    for (index, (points, tone)) in flaps.iter().enumerate() {
        if in_polygon(x, y, points) {
            return Some((face(*tone, y, CY - 14.0 - lift, lift + 14.0), index as u8));
        }
    }
    if in_polygon(x, y, &[left, front, shift(front, 0.0, depth), shift(left, 0.0, depth)]) {
        return Some((face(0.62, y, CY - 1.0, depth + 13.0), 2));
    }
    if in_polygon(x, y, &[front, right, shift(right, 0.0, depth), shift(front, 0.0, depth)]) {
        return Some((face(0.3, y, CY - 1.0, depth + 13.0), 3));
    }
    let opening = [top, right, front, left];
    let inner = opening.map(|(px, py)| (CX + (px - CX) * 0.8, CY - 1.0 + (py - (CY - 1.0)) * 0.74));
    if in_polygon(x, y, &opening) && !in_polygon(x, y, &inner) {
        return Some((0.9, 5));
    }
    if in_polygon(x, y, &opening) {
        let near = (y - (CY - 14.0)) / 26.0;
        return Some((0.06 + 0.32 * (1.0 - near).powi(2), 4));
    }
    None
}

fn magnifier(x: f32, y: f32) -> Option<Shade> {
    let lens = (CX - 7.0, CY - 4.0);
    let (dx, dy) = (x - lens.0, y - lens.1);
    let radius = dx.hypot(dy);
    if radius < 21.0 && radius > 15.5 {
        let facing = (-dx * 0.7 - dy * 0.7) / radius;
        return Some((clamp01(0.55 + 0.42 * facing), 0));
    }
    if radius <= 15.5 {
        let glint = (dx + 6.0).hypot((dy + 6.0) * 1.6);
        return Some((if glint < 4.5 { 0.85 } else { 0.05 * (1.0 - radius / 15.5) }, 1));
    }
    let start = (lens.0 + 16.0, lens.1 + 16.0);
    let end = (lens.0 + 31.0, lens.1 + 31.0);
    let distance = to_segment(x, y, start, end);
    if distance < 4.6 {
        let side = (x - y - (start.0 - start.1)) / 6.5;
        return Some((clamp01(0.5 - 0.42 * side + 0.15 * (1.0 - distance / 4.6)), 2));
    }
    None
}

fn folder(x: f32, y: f32) -> Option<Shade> {
    let left = CX - 30.0;
    let top = CY - 12.0;
    let front_top = top + 9.0;
    let lean = (front_top + 26.0 - y) / 26.0;
    if y >= front_top && y < front_top + 26.0 {
        let inset = (6.0 * lean).round();
        if x >= left - 2.0 + inset && x < left + 62.0 - inset {
            return Some((face(0.6, y, front_top, 26.0), 0));
        }
    }
    if in_rect(x, y, (left + 14.0, top - 4.0), (34.0, 16.0)) {
        let line = y == top - 1.0 || y == top + 3.0;
        return Some((if line && x > left + 18.0 && x < left + 42.0 { 0.15 } else { 0.95 }, 1));
    }
    if in_rect(x, y, (left, top), (60.0, 12.0)) || in_rect(x, y, (left, top - 6.0), (20.0, 7.0)) {
        return Some((face(0.42, y, top - 6.0, 18.0), 2));
    }
    None
}

fn planet(x: f32, y: f32) -> Option<Shade> {
    let px = x + 0.5 - CX;
    let py = y + 0.5 - (CY + 2.0);
    let r = 17.0;
    let ring_x = px * 0.94 + py * 0.34;
    let ring_y = -px * 0.34 + py * 0.94;
    let ring = (ring_x / 33.0).hypot(ring_y / 8.5);
    let on_ring = ring > 0.82 && ring < 1.0;
    let front = ring_y > 0.0;
    let sphere = px.hypot(py) < r;
    if on_ring && (front || !sphere) {
        return Some((0.42 + 0.4 * (1.0 - (ring_x + 33.0) / 66.0), 0));
    }
    if sphere {
        let (nx, ny) = (px / r, py / r);
        let nz = (1.0 - nx * nx - ny * ny).max(0.0).sqrt();
        let light = -0.55 * nx - 0.6 * ny + 0.58 * nz;
        return Some((clamp01(0.04 + 0.98 * light.max(0.0).powf(1.2)), 1));
    }
    None
}

fn object(illustration: Illustration, x: u32, y: u32) -> Option<Shade> {
    let (x, y) = (x as f32, y as f32);
    match illustration {
        Illustration::Box => open_box(x, y),
        Illustration::Search => magnifier(x, y),
        Illustration::Folder => folder(x, y),
        Illustration::Planet => planet(x, y),
    }
}

fn backdrop(illustration: Illustration, x: u32, y: u32) -> f32 {
    let halo = ((x as f32 + 0.5 - CX) / 54.0).hypot((y as f32 + 0.5 - CY) / 34.0);
    let glow = 0.12 * (1.0 - halo).max(0.0).powi(2);
    if illustration == Illustration::Planet {
        return glow;
    }
    let floor = ((x as f32 + 0.5 - CX) / 36.0).hypot((y as f32 + 0.5 - 67.0) / 3.6);
    glow.max(0.42 * (1.0 - floor).max(0.0).powf(0.8))
}

pub fn empty(illustration: Illustration, layer: Layer, matrix: Matrix, color: Color) -> Image {
    let tones: Vec<Option<Shade>> = (0..ROWS).flat_map(|y| (0..COLUMNS).map(move |x| object(illustration, x, y))).collect();
    let at = |x: i32, y: i32| {
        if (0..COLUMNS as i32).contains(&x) && (0..ROWS as i32).contains(&y) { tones[(y as u32 * COLUMNS + x as u32) as usize] } else { None }
    };
    let near = |x: i32, y: i32| (-2..=2).any(|oy| (-2..=2).any(|ox| at(x + ox, y + oy).is_some()));
    let seam = |x: i32, y: i32, part: u8| [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)].iter().any(|&(nx, ny)| at(nx, ny).is_some_and(|other| other.1 < part));
    let (ink, clear) = (pixel(color), Rgba8Pixel::new(0, 0, 0, 0));
    paint(COLUMNS, ROWS, |x, y| {
        let (cx, cy) = (x as i32, y as i32);
        let on = match (at(cx, cy), layer) {
            (Some((tone, part)), Layer::Object) => !seam(cx, cy, part) && matrix.is_inked(cx, cy, tone),
            (None, Layer::Backdrop) => !near(cx, cy) && matrix.is_inked(cx, cy, backdrop(illustration, x, y)),
            _ => false,
        };
        if on { ink } else { clear }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inked(illustration: Illustration, layer: Layer) -> Vec<bool> {
        let image = empty(illustration, layer, Matrix::Bayer4, Color::from_rgb_u8(255, 0, 0));
        image.to_rgba8().unwrap().as_slice().iter().map(|p| p.a > 0).collect()
    }

    #[test]
    fn every_illustration_draws_an_object_and_a_backdrop() {
        for illustration in Illustration::ALL {
            assert!(inked(illustration, Layer::Object).iter().filter(|on| **on).count() > 100, "{illustration:?}");
            assert!(inked(illustration, Layer::Backdrop).iter().any(|on| *on), "{illustration:?}");
        }
    }

    #[test]
    fn the_object_and_the_backdrop_never_share_a_cell() {
        for illustration in Illustration::ALL {
            let (object, backdrop) = (inked(illustration, Layer::Object), inked(illustration, Layer::Backdrop));
            assert!(object.iter().zip(&backdrop).all(|(a, b)| !(*a && *b)), "{illustration:?}");
        }
    }

    #[test]
    fn the_object_stays_clear_of_the_edges_of_the_scene() {
        for illustration in Illustration::ALL {
            let object = inked(illustration, Layer::Object);
            assert!(object[..COLUMNS as usize * 4].iter().all(|on| !on), "{illustration:?}");
        }
    }

    #[test]
    fn a_planet_has_no_floor_shadow() {
        let planet = inked(Illustration::Planet, Layer::Backdrop);
        let floor = (COLUMNS as usize * 66)..(COLUMNS as usize * 69);
        assert!(planet[floor.clone()].iter().all(|on| !on));
        assert!(inked(Illustration::Box, Layer::Backdrop)[floor].iter().any(|on| *on));
    }
}
