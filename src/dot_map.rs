use slint::{Color, Image, Rgba8Pixel};

use crate::Matrix;
use crate::render::paint;
use crate::world::{Marker, Place, Route, land_coverage};

/// The part of the world to draw, in degrees.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Bounds {
    pub north: f32,
    pub south: f32,
    pub west: f32,
    pub east: f32,
}

impl Bounds {
    /// The inhabited world: 78 degrees north to 56 south, all longitudes.
    pub const WORLD: Self = Self { north: 78.0, south: -56.0, west: -180.0, east: 180.0 };
}

pub struct DotMap<'a> {
    pub markers: &'a [Marker],
    pub arcs: &'a [Route],
    pub color: Color,
    pub accent: Color,
    /// Dots across the full width, at least 24.
    pub columns: u32,
    pub round: bool,
    /// Radius in degrees of the dithered glow each marker casts on the land around it.
    pub halo: f32,
    /// Share of sea cells with a faint dot, 0 to 1.
    pub ocean: f32,
    pub bounds: Bounds,
}

/// The dots of a map: how many there are and how many degrees one covers.
#[derive(Clone, Copy)]
pub struct Grid {
    pub columns: u32,
    pub rows: u32,
    step: f32,
    bounds: Bounds,
}

impl Grid {
    pub fn new(columns: u32, bounds: Bounds) -> Self {
        let columns = columns.max(24);
        let step = (bounds.east - bounds.west) / columns as f32;
        let rows = (((bounds.north - bounds.south) / step).round() as u32).max(1);
        Self { columns, rows, step, bounds }
    }

    /// A place in cells from the top left corner.
    pub fn project(&self, place: Place) -> (f32, f32) {
        ((place.lon - self.bounds.west) / self.step, (self.bounds.north - place.lat) / self.step)
    }
}

const LAND_OPACITY: f32 = 0.6;
const OCEAN_OPACITY: f32 = 0.25;
const PULSE: f32 = 2.8;
const RUN: f32 = 3.2;

/// Straight colour with the alpha as a separate factor, drawn source-over.
#[derive(Clone, Copy)]
struct Ink([f32; 3], f32);

impl Ink {
    fn new(color: Color) -> Self {
        let channel = |value: u8| value as f32 / 255.0;
        Self([channel(color.red()), channel(color.green()), channel(color.blue())], channel(color.alpha()))
    }
}

struct Surface {
    width: usize,
    height: usize,
    pixels: Vec<[f32; 4]>,
}

impl Surface {
    fn blend(&mut self, x: usize, y: usize, ink: Ink, coverage: f32) {
        let alpha = ink.1 * coverage;
        let below = &mut self.pixels[y * self.width + x];
        for (channel, value) in below.iter_mut().zip(ink.0) {
            *channel = value * alpha + *channel * (1.0 - alpha);
        }
        below[3] = alpha + below[3] * (1.0 - alpha);
    }

    fn apply(&mut self, mask: &Mask, ink: Ink, opacity: f32) {
        for (index, &coverage) in mask.0.iter().enumerate() {
            if coverage > 0.0 {
                self.blend(index % self.width, index / self.width, Ink(ink.0, ink.1 * opacity), coverage);
            }
        }
    }

    fn into_image(self) -> Image {
        paint(self.width as u32, self.height as u32, |x, y| {
            let [r, g, b, a] = self.pixels[y as usize * self.width + x as usize];
            let straight = |value: f32| if a > 0.0 { (value / a * 255.0).round() as u8 } else { 0 };
            Rgba8Pixel::new(straight(r), straight(g), straight(b), (a * 255.0).round() as u8)
        })
    }
}

/// Where a layer has ink, so overlapping strokes of one layer don't pile up.
struct Mask(Vec<f32>);

struct Canvas {
    surface: Surface,
    scale: f32,
}

impl Canvas {
    fn mask(&self) -> Mask {
        Mask(vec![0.0; self.surface.pixels.len()])
    }

    /// Fills the part of a rectangle in cells that lies over each pixel.
    fn fill(&self, mask: &mut Mask, (x0, y0, x1, y1): (f32, f32, f32, f32)) {
        let (width, height) = (self.surface.width, self.surface.height);
        let (px0, py0, px1, py1) = (x0 * self.scale, y0 * self.scale, x1 * self.scale, y1 * self.scale);
        for y in (py0.floor().max(0.0) as usize)..(py1.ceil().max(0.0) as usize).min(height) {
            for x in (px0.floor().max(0.0) as usize)..(px1.ceil().max(0.0) as usize).min(width) {
                let across = |from: f32, to: f32, at: usize| (to.min(at as f32 + 1.0) - from.max(at as f32)).clamp(0.0, 1.0);
                let coverage = across(px0, px1, x) * across(py0, py1, y);
                let cell = &mut mask.0[y * width + x];
                *cell = cell.max(coverage);
            }
        }
    }

    /// A stamp of at least one pixel, centred on a point in cells.
    fn stamp(&self, mask: &mut Mask, (x, y): (f32, f32), width: f32) {
        let half = (width * self.scale / 2.0).max(0.5) / self.scale;
        self.fill(mask, (x - half, y - half, x + half, y + half));
    }

    fn ring(&self, mask: &mut Mask, (x, y): (f32, f32), half: f32, stroke: f32) {
        let mut outer = self.mask();
        let mut inner = self.mask();
        self.fill(&mut outer, (x - half - stroke / 2.0, y - half - stroke / 2.0, x + half + stroke / 2.0, y + half + stroke / 2.0));
        self.fill(&mut inner, (x - half + stroke / 2.0, y - half + stroke / 2.0, x + half - stroke / 2.0, y + half - stroke / 2.0));
        for ((cell, outer), inner) in mask.0.iter_mut().zip(outer.0).zip(inner.0) {
            *cell = cell.max((outer - inner).max(0.0));
        }
    }
}

/// How much of each pixel of a cell the dot covers.
fn sprite(scale: usize, round: bool) -> Vec<f32> {
    const SAMPLES: usize = 4;
    let size = scale as f32;
    (0..scale * scale)
        .map(|index| {
            let (px, py) = ((index % scale) as f32, (index / scale) as f32);
            if !round {
                let across = |at: f32| ((at + 1.0).min(0.8 * size) - at.max(0.2 * size)).clamp(0.0, 1.0);
                return across(px) * across(py);
            }
            let inside = (0..SAMPLES * SAMPLES)
                .filter(|sample| {
                    let x = px + ((sample % SAMPLES) as f32 + 0.5) / SAMPLES as f32 - size / 2.0;
                    let y = py + ((sample / SAMPLES) as f32 + 0.5) / SAMPLES as f32 - size / 2.0;
                    x.hypot(y) <= 0.32 * size
                })
                .count();
            inside as f32 / (SAMPLES * SAMPLES) as f32
        })
        .collect()
}

fn land_cells(grid: &Grid) -> Vec<bool> {
    let Grid { columns, rows, step, bounds } = *grid;
    (0..rows)
        .flat_map(|y| (0..columns).map(move |x| (x, y)))
        .map(|(x, y)| {
            let lat = bounds.north - (y as f32 + 0.5) * step;
            let lon = bounds.west + (x as f32 + 0.5) * step;
            land_coverage(lat, lon) >= 0.5
        })
        .collect()
}

/// The marker whose halo covers a cell most strongly, if the dither lets it through.
fn halo_owner(centres: &[(f32, f32)], radius: f32, x: u32, y: u32) -> Option<usize> {
    let (mut owner, mut best) = (None, 0.0);
    for (index, &(cx, cy)) in centres.iter().enumerate() {
        let tone = 1.0 - (x as f32 + 0.5 - cx).hypot(y as f32 + 0.5 - cy) / radius;
        if tone > best {
            (best, owner) = (tone, Some(index));
        }
    }
    // Dense at the marker, thinning out through the Bayer levels.
    owner.filter(|_| best.sqrt() > Matrix::Bayer4.threshold_at(x as i32, y as i32))
}

/// A lifted quadratic curve between two points, bowing towards the top.
fn curve(grid: &Grid, route: &Route) -> impl Fn(f32) -> (f32, f32) {
    let ((x1, y1), (x2, y2)) = (grid.project(route.from), grid.project(route.to));
    let lift = (x2 - x1).hypot(y2 - y1) * 0.32;
    let (mx, my) = ((x1 + x2) / 2.0, (y1 + y2) / 2.0 - lift);
    move |t| {
        let u = 1.0 - t;
        (u * u * x1 + 2.0 * u * t * mx + t * t * x2, u * u * y1 + 2.0 * u * t * my + t * t * y2)
    }
}

fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 { 4.0 * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0 }
}

fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(5)
}

fn paint_arcs(map: &DotMap, grid: &Grid, canvas: &mut Canvas, time: Option<f32>) {
    for (index, route) in map.arcs.iter().enumerate() {
        let ink = Ink::new(route.color.unwrap_or(map.accent));
        let at = curve(grid, route);
        let samples = (grid.columns as f32 * canvas.scale * 2.0) as usize;
        let mut along = Vec::with_capacity(samples + 1);
        let mut length = 0.0;
        let mut last = at(0.0);
        for i in 0..=samples {
            let point = at(i as f32 / samples as f32);
            length += (point.0 - last.0).hypot(point.1 - last.1);
            last = point;
            along.push((point, length));
        }

        let mut dotted = canvas.mask();
        for &(point, at) in &along {
            if at % 1.5 < 0.6 {
                canvas.stamp(&mut dotted, point, 0.45);
            }
        }
        canvas.surface.apply(&dotted, ink, 0.7);

        if let Some(time) = time {
            // A bright stretch, 14 per cent of the curve, runs along it and wraps round.
            let head = ease_in_out(((time + index as f32 * 0.7) / RUN).rem_euclid(1.0)) * 100.0;
            let mut bright = canvas.mask();
            for &(point, at) in &along {
                if (at / length.max(f32::EPSILON) * 100.0 - head).rem_euclid(100.0) < 14.0 {
                    canvas.stamp(&mut bright, point, 0.6);
                }
            }
            canvas.surface.apply(&bright, ink, 1.0);
        }
    }
}

fn paint_markers(map: &DotMap, grid: &Grid, canvas: &mut Canvas, time: Option<f32>) {
    for (index, marker) in map.markers.iter().enumerate() {
        let ink = Ink::new(marker.color.unwrap_or(map.accent));
        let (x, y) = grid.project(marker.at);
        if let Some(time) = time {
            // A square ring that grows and fades, each marker a little behind the last.
            let delay = (index as f32 * 0.53) % PULSE;
            let phase = if time < delay { 0.0 } else { ((time - delay) / PULSE).rem_euclid(1.0) };
            let eased = ease_out(phase);
            let mut pulse = canvas.mask();
            canvas.ring(&mut pulse, (x, y), 0.8 * (0.6 + 2.8 * eased), 0.25 * (0.6 + 2.8 * eased));
            canvas.surface.apply(&pulse, ink, 0.9 * (1.0 - eased));
        }
        let mut square = canvas.mask();
        canvas.fill(&mut square, (x - 0.8, y - 0.8, x + 0.8, y + 0.8));
        canvas.surface.apply(&square, ink, 1.0);
    }
}

/// The map as an image `scale` pixels per dot wide. Marker pulses and the light running along the
/// arcs follow `time` in seconds; with `None` the map is still.
pub fn dot_map(map: &DotMap, scale: u32, time: Option<f32>) -> Image {
    let grid = Grid::new(map.columns, map.bounds);
    let scale = scale.max(2) as usize;
    let mut canvas = Canvas {
        surface: Surface {
            width: grid.columns as usize * scale,
            height: grid.rows as usize * scale,
            pixels: vec![[0.0; 4]; grid.columns as usize * grid.rows as usize * scale * scale],
        },
        scale: scale as f32,
    };

    let land = land_cells(&grid);
    let centres: Vec<_> = map.markers.iter().map(|marker| grid.project(marker.at)).collect();
    let radius = map.halo / grid.step;
    let owners: Vec<Option<usize>> = land
        .iter()
        .enumerate()
        .map(|(index, &on)| {
            let (x, y) = (index as u32 % grid.columns, index as u32 / grid.columns);
            if on && radius > 0.0 { halo_owner(&centres, radius, x, y) } else { None }
        })
        .collect();

    let dot = sprite(scale, map.round);
    let draw = |surface: &mut Surface, index: usize, ink: Ink, opacity: f32| {
        let (cx, cy) = ((index % grid.columns as usize) * scale, (index / grid.columns as usize) * scale);
        for (offset, &coverage) in dot.iter().enumerate() {
            if coverage > 0.0 {
                surface.blend(cx + offset % scale, cy + offset / scale, Ink(ink.0, ink.1 * opacity), coverage);
            }
        }
    };

    let (land_ink, accent) = (Ink::new(map.color), Ink::new(map.accent));
    for (index, &on) in land.iter().enumerate() {
        let (x, y) = (index as i32 % grid.columns as i32, index as i32 / grid.columns as i32);
        if !on && map.ocean > Matrix::BlueNoise.threshold_at(x, y) {
            draw(&mut canvas.surface, index, land_ink, OCEAN_OPACITY);
        }
    }
    for (index, &on) in land.iter().enumerate() {
        if on && owners[index].is_none() {
            draw(&mut canvas.surface, index, land_ink, LAND_OPACITY);
        }
    }
    for (index, owner) in owners.iter().enumerate() {
        if let Some(owner) = owner {
            let ink = map.markers[*owner].color.map_or(accent, Ink::new);
            draw(&mut canvas.surface, index, ink, 1.0);
        }
    }

    paint_arcs(map, &grid, &mut canvas, time);
    paint_markers(map, &grid, &mut canvas, time);
    canvas.surface.into_image()
}

#[cfg(test)]
mod tests {
    use super::*;

    const GREY: Color = Color::from_rgb_u8(128, 128, 128);
    const ORANGE: Color = Color::from_rgb_u8(255, 120, 0);
    const PARIS: Place = Place { lat: 48.9, lon: 2.3 };

    fn map<'a>(markers: &'a [Marker], arcs: &'a [Route]) -> DotMap<'a> {
        DotMap { markers, arcs, color: GREY, accent: ORANGE, columns: 80, round: false, halo: 14.0, ocean: 0.0, bounds: Bounds::WORLD }
    }

    fn pixels(map: &DotMap, time: Option<f32>) -> Vec<Rgba8Pixel> {
        dot_map(map, 6, time).to_rgba8().unwrap().as_slice().to_vec()
    }

    fn coloured(pixels: &[Rgba8Pixel], color: Color) -> usize {
        pixels.iter().filter(|p| p.a > 150 && p.r == color.red() && p.g == color.green()).count()
    }

    #[test]
    fn the_grid_follows_the_bounds_and_has_a_minimum_width() {
        let grid = Grid::new(160, Bounds::WORLD);
        assert_eq!((grid.columns, grid.rows), (160, 60));
        assert_eq!(Grid::new(5, Bounds::WORLD).columns, 24);
    }

    #[test]
    fn the_image_is_the_grid_scaled() {
        let image = dot_map(&map(&[], &[]), 6, None);
        let grid = Grid::new(80, Bounds::WORLD);
        assert_eq!((image.size().width, image.size().height), (grid.columns * 6, grid.rows * 6));
    }

    #[test]
    fn land_is_drawn_faintly_and_the_sea_stays_empty() {
        let pixels = pixels(&map(&[], &[]), None);
        let size = 80 * 6;
        let at = |lat: f32, lon: f32| {
            let (x, y) = Grid::new(80, Bounds::WORLD).project(Place { lat, lon });
            pixels[(y as usize * 6 + 3) * size + x as usize * 6 + 3]
        };
        assert!(pixels.iter().any(|p| p.a > 100 && p.a < 200));
        assert_eq!(at(0.0, -30.0).a, 0);
    }

    #[test]
    fn a_share_of_the_sea_can_get_faint_dots() {
        let dry = pixels(&map(&[], &[]), None).iter().filter(|p| p.a > 0).count();
        let wet = pixels(&DotMap { ocean: 0.5, ..map(&[], &[]) }, None).iter().filter(|p| p.a > 0).count();
        assert!(wet > dry);
    }

    #[test]
    fn a_marker_is_a_solid_square_in_the_accent_colour() {
        let markers = [Marker { at: PARIS, color: None }];
        assert!(coloured(&pixels(&map(&markers, &[]), None), ORANGE) > 20);
    }

    #[test]
    fn a_marker_with_its_own_colour_leaves_the_accent_unused() {
        let blue = Color::from_rgb_u8(0, 0, 255);
        let markers = [Marker { at: PARIS, color: Some(blue) }];
        let pixels = pixels(&map(&markers, &[]), None);
        assert!(pixels.iter().any(|p| p.b == 255 && p.r == 0 && p.a > 200));
        assert_eq!(coloured(&pixels, ORANGE), 0);
    }

    #[test]
    fn the_halo_recolours_land_around_a_marker_and_can_be_switched_off() {
        let markers = [Marker { at: PARIS, color: None }];
        let with = coloured(&pixels(&map(&markers, &[]), None), ORANGE);
        let without = coloured(&pixels(&DotMap { halo: 0.0, ..map(&markers, &[]) }, None), ORANGE);
        assert!(with > without * 2);
    }

    #[test]
    fn an_arc_joins_two_places() {
        let arcs = [Route { from: PARIS, to: Place { lat: 40.7, lon: -74.0 }, color: None }];
        let with = coloured(&pixels(&map(&[], &arcs), None), ORANGE);
        assert!(with > 30);
    }

    #[test]
    fn animation_adds_pulses_and_a_running_light_and_moves_them() {
        let markers = [Marker { at: PARIS, color: None }];
        let arcs = [Route { from: PARIS, to: Place { lat: 40.7, lon: -74.0 }, color: None }];
        let still = pixels(&map(&markers, &arcs), None);
        let early = pixels(&map(&markers, &arcs), Some(1.0));
        let later = pixels(&map(&markers, &arcs), Some(2.0));
        assert_ne!(still, early);
        assert_ne!(early, later);
    }

    #[test]
    fn round_dots_cover_less_than_square_ones() {
        let area = |round: bool| pixels(&DotMap { round, ..map(&[], &[]) }, None).iter().map(|p| p.a as u32).sum::<u32>();
        assert!(area(true) < area(false));
    }
}
