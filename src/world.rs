//! A land mask of the world, from Natural Earth's 1:50m land polygons (public domain,
//! naturalearthdata.com), rasterised onto a 480 by 240 equirectangular grid of 0.75 degree cells
//! and run-length encoded in `world_land.txt`. It is shared by the globe and the dot map.

use std::sync::OnceLock;

use slint::Color;

/// A place on Earth, in degrees: latitude north positive, longitude east positive.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Place {
    pub lat: f32,
    pub lon: f32,
}

/// A pulsing location. With no colour of its own it takes the accent of whatever draws it.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Marker {
    pub at: Place,
    pub color: Option<Color>,
}

/// A dotted line between two places.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Route {
    pub from: Place,
    pub to: Place,
    pub color: Option<Color>,
}

const COLUMNS: usize = 480;
const ROWS: usize = 240;
const ALPHABET: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz-_";

/// Runs alternate ocean, land, ocean and so on in row-major order from the north-west corner.
/// Each character is a run length in base 64, and `_` adds 63 and continues the same run.
fn mask() -> &'static [u8] {
    static MASK: OnceLock<Vec<u8>> = OnceLock::new();
    MASK.get_or_init(|| {
        let mut mask = vec![0; COLUMNS * ROWS];
        let (mut at, mut run, mut land) = (0, 0, false);
        for char in include_str!("world_land.txt").trim().chars() {
            let value = ALPHABET.find(char).expect("the mask only holds base 64 characters");
            run += value;
            if value == 63 {
                continue;
            }
            if land {
                mask[at..at + run].fill(1);
            }
            at += run;
            run = 0;
            land = !land;
        }
        mask
    })
}

fn wrap(value: i32, size: usize) -> usize {
    value.rem_euclid(size as i32) as usize
}

/// Land coverage around a point, 0 to 1, blended from the four nearest cells. Latitude is north
/// positive and longitude east positive, in degrees. Thresholding it at 0.5 gives smoother coasts
/// than a single cell when the mask is drawn larger than its own resolution.
pub fn land_coverage(lat: f32, lon: f32) -> f32 {
    let mask = mask();
    let x = (lon + 180.0) / 360.0 * COLUMNS as f32 - 0.5;
    let y = ((90.0 - lat) / 180.0 * ROWS as f32 - 0.5).clamp(0.0, ROWS as f32 - 1.0);
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let y1 = (y0 as usize + 1).min(ROWS - 1);
    let at = |column: i32, row: usize| mask[row * COLUMNS + wrap(column, COLUMNS)] as f32;
    let (x0, y0) = (x0 as i32, y0 as usize);
    let top = at(x0, y0) * (1.0 - fx) + at(x0 + 1, y0) * fx;
    let bottom = at(x0, y1) * (1.0 - fx) + at(x0 + 1, y1) * fx;
    top * (1.0 - fy) + bottom * fy
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_mask_fills_the_whole_grid_with_about_a_third_land() {
        let land = mask().iter().filter(|&&cell| cell == 1).count() as f32 / mask().len() as f32;
        assert!((0.2..0.35).contains(&land), "{land}");
    }

    #[test]
    fn known_places_are_land_or_sea() {
        assert!(land_coverage(48.0, 2.0) > 0.9, "Paris");
        assert!(land_coverage(-25.0, 135.0) > 0.9, "Australia");
        assert!(land_coverage(0.0, -30.0) < 0.1, "the Atlantic");
        assert!(land_coverage(-30.0, -140.0) < 0.1, "the Pacific");
        assert!(land_coverage(-80.0, 0.0) > 0.9, "Antarctica");
    }

    #[test]
    fn longitude_wraps_around_the_date_line() {
        assert!((land_coverage(60.0, 179.9) - land_coverage(60.0, -180.1)).abs() < 0.5);
        assert_eq!(land_coverage(48.0, 2.0), land_coverage(48.0, 362.0));
    }
}
