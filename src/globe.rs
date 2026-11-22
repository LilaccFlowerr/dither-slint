use glam::Vec3;
use slint::{Color, Image, Rgba8Pixel};

use crate::Matrix;
use crate::render::{paint, pixel};
use crate::world::{Marker, Place, Route, land_coverage};

const RADIUS_SHARE: f32 = 0.42;
const PULSE: f32 = 2.6;

pub struct Globe<'a> {
    pub land: Color,
    pub accent: Color,
    pub markers: &'a [Marker],
    pub arcs: &'a [Route],
    /// Where the light comes from, in degrees clockwise from the top.
    pub light: f32,
    /// Brightness of the sea on the lit side, 0 to 1.
    pub ocean: f32,
    pub atmosphere: bool,
    pub matrix: Matrix,
}

/// How the globe is turned and how far the markers have pulsed, as degrees and seconds.
#[derive(Clone, Copy)]
pub struct View {
    /// The longitude facing the viewer.
    pub spin: f32,
    /// How far the north pole leans towards the viewer.
    pub tilt: f32,
    pub time: f32,
}

/// The globe's orientation, as sines and cosines worked out once a frame.
struct Orientation {
    cos_spin: f32,
    sin_spin: f32,
    cos_tilt: f32,
    sin_tilt: f32,
}

impl Orientation {
    fn new(view: &View) -> Self {
        let (spin, tilt) = (view.spin.to_radians(), -view.tilt.to_radians());
        Self { cos_spin: spin.cos(), sin_spin: spin.sin(), cos_tilt: tilt.cos(), sin_tilt: tilt.sin() }
    }

    /// View space to the globe's own frame: undo the tilt, then the spin.
    fn to_world(&self, v: Vec3) -> Vec3 {
        let y = v.y * self.cos_tilt - v.z * self.sin_tilt;
        let z = v.y * self.sin_tilt + v.z * self.cos_tilt;
        Vec3::new(v.x * self.cos_spin + z * self.sin_spin, y, -v.x * self.sin_spin + z * self.cos_spin)
    }

    /// The globe's frame to view space.
    fn to_view(&self, v: Vec3) -> Vec3 {
        let x = v.x * self.cos_spin - v.z * self.sin_spin;
        let z = v.x * self.sin_spin + v.z * self.cos_spin;
        Vec3::new(x, v.y * self.cos_tilt + z * self.sin_tilt, -v.y * self.sin_tilt + z * self.cos_tilt)
    }
}

fn on_sphere(place: Place) -> Vec3 {
    let (phi, lambda) = (place.lat.to_radians(), place.lon.to_radians());
    Vec3::new(phi.cos() * lambda.sin(), phi.sin(), phi.cos() * lambda.cos())
}

fn light_vector(degrees: f32) -> Vec3 {
    let angle = degrees.to_radians();
    Vec3::new(angle.sin() * 0.75, angle.cos() * 0.75, 0.62).normalize()
}

struct Disc {
    columns: i32,
    rows: i32,
    centre: (f32, f32),
    radius: f32,
}

struct Projected {
    x: f32,
    y: f32,
    /// Depth towards the viewer, -1 to 1 on the surface.
    z: f32,
    /// In front of the globe, or beside it where the globe can't hide it.
    visible: bool,
}

struct Scene<'a> {
    disc: Disc,
    orientation: Orientation,
    light: Vec3,
    globe: &'a Globe<'a>,
    time: f32,
}

fn round(value: f32) -> i32 {
    (value + 0.5).floor() as i32
}

impl Scene<'_> {
    fn project(&self, point: Vec3) -> Projected {
        let v = self.orientation.to_view(point);
        Projected {
            x: self.disc.centre.0 + v.x * self.disc.radius,
            y: self.disc.centre.1 - v.y * self.disc.radius,
            z: v.z,
            visible: v.z > 0.0 || v.x * v.x + v.y * v.y > 1.0,
        }
    }

    fn surface_tone(&self, normal: Vec3) -> f32 {
        let lit = normal.dot(self.light);
        // Lambert light with a short soft edge past the terminator.
        let shade = ((lit + 0.08) / 1.08).clamp(0.0, 1.0);
        let world = self.orientation.to_world(normal);
        let lat = world.y.clamp(-1.0, 1.0).asin().to_degrees();
        let lon = world.x.atan2(world.z).to_degrees();
        if land_coverage(lat, lon) >= 0.5 {
            return 0.07 + 0.93 * shade;
        }
        let rim = (1.0 - normal.z).powi(3);
        shade * (self.globe.ocean * (0.5 + rim) + rim * 0.1)
    }

    fn inked(&self, x: i32, y: i32, tone: f32) -> bool {
        self.globe.matrix.is_inked(x, y, tone)
    }
}

/// Paints source-over, as a canvas does.
struct Canvas {
    columns: i32,
    rows: i32,
    pixels: Vec<Rgba8Pixel>,
}

impl Canvas {
    fn put(&mut self, x: i32, y: i32, color: Rgba8Pixel) {
        if (0..self.columns).contains(&x) && (0..self.rows).contains(&y) {
            let below = &mut self.pixels[(y * self.columns + x) as usize];
            let a = color.a as f32 / 255.0;
            let blend = |top: u8, under: u8| (top as f32 * a + under as f32 * (1.0 - a)).round() as u8;
            *below = if below.a == 0 || color.a == 255 {
                color
            } else {
                Rgba8Pixel::new(blend(color.r, below.r), blend(color.g, below.g), blend(color.b, below.b), below.a.max(color.a))
            };
        }
    }

    fn fill_cell(&mut self, x: f32, y: f32, size: i32, color: Rgba8Pixel) {
        let offset = size / 2;
        for dy in 0..size {
            for dx in 0..size {
                self.put(round(x) - offset + dx, round(y) - offset + dy, color);
            }
        }
    }
}

fn paint_sphere(scene: &Scene, canvas: &mut Canvas, land: Rgba8Pixel) {
    let Disc { columns, rows, centre, radius } = scene.disc;
    for y in 0..rows {
        for x in 0..columns {
            let nx = (x as f32 + 0.5 - centre.0) / radius;
            let ny = (centre.1 - y as f32 - 0.5) / radius;
            let r2 = nx * nx + ny * ny;
            let tone = if r2 <= 1.0 {
                scene.surface_tone(Vec3::new(nx, ny, (1.0 - r2).sqrt()))
            } else if scene.globe.atmosphere && r2 < 1.16 {
                // A thin glow around the sphere, strongest towards the light.
                let strength = 1.0 - (r2.sqrt() - 1.0) / 0.077;
                let facing = ((nx * scene.light.x + ny * scene.light.y) / nx.hypot(ny)).max(0.0);
                strength.powi(2) * (0.12 + 0.5 * facing)
            } else {
                continue;
            };
            if scene.inked(x, y, tone) {
                canvas.put(x, y, land);
            }
        }
    }
}

/// A point on the great circle from `a` to `b`, lifted in the middle.
fn arc_point(a: Vec3, b: Vec3, angle: f32, s: f32) -> Vec3 {
    let sin = if angle.sin() == 0.0 { 1.0 } else { angle.sin() };
    let (wa, wb) = (((1.0 - s) * angle).sin() / sin, (s * angle).sin() / sin);
    let lift = 1.0 + (std::f32::consts::PI * s).sin() * (0.06 + angle * 0.1);
    (a * wa + b * wb) * lift
}

fn paint_arcs(scene: &Scene, canvas: &mut Canvas) {
    for (index, arc) in scene.globe.arcs.iter().enumerate() {
        let (a, b) = (on_sphere(arc.from), on_sphere(arc.to));
        let angle = a.dot(b).min(1.0).acos();
        let color = pixel(arc.color.unwrap_or(scene.globe.accent));
        let steps = ((angle * scene.disc.radius * 1.3).ceil() as i32).max(12);
        // A bright stretch travels along each arc; the rest is dotted.
        let head = (scene.time / 3.2 + index as f32 * 0.41) % 1.5;
        for s in 0..=steps {
            let t = s as f32 / steps as f32;
            let point = scene.project(arc_point(a, b, angle, t));
            let lit = t <= head && t > head - 0.25;
            if point.visible && (lit || s % 3 == 0) {
                canvas.fill_cell(point.x, point.y, 1, color);
            }
        }
    }
}

fn paint_markers(scene: &Scene, canvas: &mut Canvas) {
    for (index, marker) in scene.globe.markers.iter().enumerate() {
        let point = scene.project(on_sphere(marker.at));
        if point.z <= 0.05 {
            continue;
        }
        let color = pixel(marker.color.unwrap_or(scene.globe.accent));
        // An expanding ring of dithered cells that fades as it grows.
        let phase = (scene.time / PULSE + index as f32 * 0.37) % 1.0;
        let ring = 2.0 + phase * 6.0_f32.max(scene.disc.radius * 0.1);
        let fade = (1.0 - phase).powf(1.5);
        let steps = (ring * 6.0).ceil() as i32;
        for s in 0..steps {
            let angle = s as f32 / steps as f32 * std::f32::consts::TAU;
            let x = round(point.x + angle.cos() * ring);
            // Flattened by depth, so rings near the limb lie on the surface.
            let y = round(point.y + angle.sin() * ring * point.z);
            if scene.inked(x, y, fade) {
                canvas.fill_cell(x as f32, y as f32, 1, color);
            }
        }
        canvas.fill_cell(point.x, point.y, 3, color);
    }
}

pub fn globe(columns: u32, rows: u32, globe: &Globe, view: &View) -> Image {
    let (columns, rows) = (columns.max(1), rows.max(1));
    let scene = Scene {
        disc: Disc {
            columns: columns as i32,
            rows: rows as i32,
            centre: (columns as f32 / 2.0, rows as f32 / 2.0),
            radius: columns.min(rows) as f32 * RADIUS_SHARE,
        },
        orientation: Orientation::new(view),
        light: light_vector(globe.light),
        globe,
        time: view.time,
    };
    let mut canvas = Canvas {
        columns: columns as i32,
        rows: rows as i32,
        pixels: vec![Rgba8Pixel::new(0, 0, 0, 0); (columns * rows) as usize],
    };
    paint_sphere(&scene, &mut canvas, pixel(globe.land));
    paint_arcs(&scene, &mut canvas);
    paint_markers(&scene, &mut canvas);
    paint(columns, rows, |x, y| canvas.pixels[(y * columns + x) as usize])
}

#[cfg(test)]
mod tests {
    use super::*;

    const WHITE: Color = Color::from_rgb_u8(255, 255, 255);
    const ORANGE: Color = Color::from_rgb_u8(255, 120, 0);

    fn sample<'a>(markers: &'a [Marker], arcs: &'a [Route]) -> Globe<'a> {
        Globe { land: WHITE, accent: ORANGE, markers, arcs, light: 300.0, ocean: 0.18, atmosphere: true, matrix: Matrix::Bayer4 }
    }

    const FRONT: View = View { spin: 10.0, tilt: 18.0, time: 1.0 };

    fn pixels(globe: &Globe, view: &View) -> Vec<Rgba8Pixel> {
        super::globe(120, 120, globe, view).to_rgba8().unwrap().as_slice().to_vec()
    }

    fn count(pixels: &[Rgba8Pixel], color: Color) -> usize {
        pixels.iter().filter(|p| **p == pixel(color)).count()
    }

    #[test]
    fn the_globe_is_a_round_disc_with_clear_corners() {
        let pixels = pixels(&sample(&[], &[]), &FRONT);
        assert!(count(&pixels, WHITE) > 500);
        assert!(pixels[..120].iter().all(|p| p.a == 0));
        assert!(pixels[120 * 119..].iter().all(|p| p.a == 0));
    }

    #[test]
    fn turning_it_shows_other_land() {
        let before = pixels(&sample(&[], &[]), &FRONT);
        let after = pixels(&sample(&[], &[]), &View { spin: 100.0, ..FRONT });
        assert_ne!(before, after);
    }

    #[test]
    fn a_marker_shows_on_the_near_side_and_hides_on_the_far_side() {
        let near = [Marker { at: Place { lat: 5.0, lon: 10.0 }, color: None }];
        let far = [Marker { at: Place { lat: 5.0, lon: 190.0 }, color: None }];
        assert!(count(&pixels(&sample(&near, &[]), &FRONT), ORANGE) >= 9);
        assert_eq!(count(&pixels(&sample(&far, &[]), &FRONT), ORANGE), 0);
    }

    #[test]
    fn a_marker_keeps_its_own_colour() {
        let blue = Color::from_rgb_u8(0, 0, 255);
        let markers = [Marker { at: Place { lat: 5.0, lon: 10.0 }, color: Some(blue) }];
        let pixels = pixels(&sample(&markers, &[]), &FRONT);
        assert!(count(&pixels, blue) >= 9);
        assert_eq!(count(&pixels, ORANGE), 0);
    }

    #[test]
    fn an_arc_is_drawn_between_two_places() {
        let arcs = [Route { from: Place { lat: 48.0, lon: 2.0 }, to: Place { lat: 40.0, lon: -74.0 }, color: None }];
        assert!(count(&pixels(&sample(&[], &arcs), &FRONT), ORANGE) > 10);
    }

    #[test]
    fn the_atmosphere_can_be_switched_off() {
        let on = count(&pixels(&sample(&[], &[]), &FRONT), WHITE);
        let off = count(&pixels(&Globe { atmosphere: false, ..sample(&[], &[]) }, &FRONT), WHITE);
        assert!(on > off);
    }

    #[test]
    fn the_markers_pulse_over_time() {
        let markers = [Marker { at: Place { lat: 5.0, lon: 10.0 }, color: None }];
        let at = |time: f32| pixels(&sample(&markers, &[]), &View { time, ..FRONT });
        assert_ne!(at(0.5), at(2.0));
    }
}
