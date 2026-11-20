use slint::{Color, Image, Rgba8Pixel};

use crate::Matrix;
use crate::render::{paint, pixel};
use crate::style::FillVariant;

const GRID: usize = 5;
const HALF: usize = 3;
const SUB: i32 = 4;
const PAD: f32 = 1.5;
const MODULUS: u64 = 2_147_483_647;

pub const SIZE: u32 = (GRID as i32 * SUB + (PAD * 2.0) as i32 * SUB) as u32;
pub const HUES: usize = 6;

fn seed(value: &str) -> u64 {
    let hash = value.chars().fold(7, |hash, c| (hash * 31 + c as u64) % MODULUS);
    if hash == 0 { 1 } else { hash }
}

struct Random(u64);

impl Random {
    fn next(&mut self) -> f32 {
        self.0 = self.0 * 48_271 % MODULUS;
        self.0 as f32 / MODULUS as f32
    }
}

pub fn hue(name: &str) -> usize {
    seed(&format!("{name}:hue")) as usize % HUES
}

pub fn face(name: &str) -> [[f32; GRID]; GRID] {
    let mut random = Random(seed(name));
    let centre = (GRID - 1) as f32 / 2.0;
    let reach = centre.hypot(centre);
    let mut rows = [[0.0; GRID]; GRID];
    for (y, row) in rows.iter_mut().enumerate() {
        let mut half = [0.0; HALF];
        for (x, tone) in half.iter_mut().enumerate() {
            let near = 1.0 - (x as f32 - centre).hypot(y as f32 - centre) / reach;
            if random.next() > 0.3 + 0.75 * near || near < 0.05 {
                continue;
            }
            *tone = if random.next() < 0.25 + 0.6 * near { 1.0 } else { 0.5 };
        }
        row[..HALF].copy_from_slice(&half);
        row[HALF] = half[1];
        row[HALF + 1] = half[0];
    }
    let spine = rows.iter().filter(|row| row[2] > 0.0).count();
    if spine < 2 {
        for row in &mut rows[1..4] {
            row[2] = 1.0;
        }
    }
    rows
}

fn near_face(face: &[[f32; GRID]; GRID], x: i32, y: i32) -> bool {
    let pad = (PAD * SUB as f32) as i32;
    (-1..=1).any(|oy| {
        (-1..=1).any(|ox| {
            let bx = (x + ox - pad).div_euclid(SUB);
            let by = (y + oy - pad).div_euclid(SUB);
            (0..GRID as i32).contains(&bx) && (0..GRID as i32).contains(&by) && face[by as usize][bx as usize] > 0.0
        })
    })
}

fn disc_tone(x: i32, y: i32) -> f32 {
    let half = SIZE as f32 / 2.0;
    let nx = (x as f32 + 0.5 - half) / half;
    let ny = (y as f32 + 0.5 - half) / half;
    let light = 1.0 - (nx + 0.45).hypot(ny + 0.45) / 1.6;
    0.26 * light.max(0.0).powf(1.3)
}

fn inked(fill: FillVariant, tone: f32, x: i32, y: i32) -> bool {
    match fill {
        FillVariant::Gradient => Matrix::Bayer4.is_inked(x, y, tone),
        FillVariant::Dotted => tone >= 1.0 || (y % 2 == 0 && (x % 4 + y / 2) % 2 == 0),
        FillVariant::Hatched => tone >= 1.0 || (x + y) % 4 == 3,
        FillVariant::Solid => true,
    }
}

pub fn avatar(name: &str, fill: FillVariant, color: Color) -> Image {
    let face = face(name);
    let (ink, clear) = (pixel(color), Rgba8Pixel::new(0, 0, 0, 0));
    let pad = (PAD * SUB as f32) as i32;
    paint(SIZE, SIZE, |x, y| {
        let (x, y) = (x as i32, y as i32);
        let (bx, by) = ((x - pad).div_euclid(SUB), (y - pad).div_euclid(SUB));
        if (0..GRID as i32).contains(&bx) && (0..GRID as i32).contains(&by) {
            let tone = face[by as usize][bx as usize];
            if tone > 0.0 && inked(fill, tone, (x - pad).rem_euclid(SUB), (y - pad).rem_euclid(SUB)) {
                let opacity = if fill == FillVariant::Solid && tone < 1.0 { 0.45 } else { 1.0 };
                return Rgba8Pixel::new(ink.r, ink.g, ink.b, (ink.a as f32 * opacity).round() as u8);
            }
            return clear;
        }
        if !near_face(&face, x, y) && Matrix::Bayer4.is_inked(x, y, disc_tone(x, y)) { ink } else { clear }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_name_gives_the_same_face() {
        assert_eq!(face("Ada Lovelace"), face("Ada Lovelace"));
        assert_ne!(face("Ada Lovelace"), face("Grace Hopper"));
    }

    #[test]
    fn a_face_is_mirrored_and_has_a_spine() {
        for name in ["a", "bb", "Ada", "Grace Hopper", "x@y.z"] {
            let rows = face(name);
            assert!(rows.iter().all(|row| row[0] == row[4] && row[1] == row[3]));
            assert!(rows.iter().filter(|row| row[2] > 0.0).count() >= 2);
        }
    }

    #[test]
    fn corners_stay_empty_so_a_circle_never_clips_the_face() {
        for name in ["a", "bb", "Ada", "Grace Hopper"] {
            let rows = face(name);
            assert_eq!(rows[0][0], 0.0);
            assert_eq!(rows[4][4], 0.0);
        }
    }

    #[test]
    fn the_hue_is_stable_and_within_the_palette() {
        assert_eq!(hue("Ada"), hue("Ada"));
        assert!((0..50).all(|i| hue(&format!("user{i}")) < HUES));
    }
}
