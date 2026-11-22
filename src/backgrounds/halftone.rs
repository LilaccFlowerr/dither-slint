use glam::Vec2;

use super::{param, smoothstep};
use crate::field::{Cell, Field, Frame};
use crate::noise::{fbm, snoise};

pub struct Halftone;

const MOST_CHANNELS: usize = 3;

struct Settings {
    mode: f32,
    centre: Vec2,
    radius: f32,
    direction: f32,
    scale: f32,
    intensity: f32,
    drift: f32,
    angle: f32,
    pitch: f32,
    channels: usize,
}

impl Settings {
    // params: source (0 noise, 1 radial, 2 linear), centre x, centre y, radius, direction in
    // degrees, scale, intensity, drift, angle in degrees, pitch in rendered pixels, channels
    fn new(params: &[f32]) -> Self {
        let [mode, centre_x, centre_y, radius, direction, scale, intensity, drift, angle, pitch, channels] = std::array::from_fn(|index| param(params, index));
        Self {
            mode,
            centre: Vec2::new(centre_x, centre_y),
            radius: radius.max(0.01),
            direction: direction.to_radians(),
            scale: scale.max(0.05),
            intensity: intensity.clamp(0.0, 1.0),
            drift,
            angle,
            pitch: pitch.max(1.0),
            channels: channels.round().clamp(1.0, MOST_CHANNELS as f32) as usize,
        }
    }

    /// How large the dots of `channel` are around `uv`, 0 to 1.
    fn source(&self, uv: Vec2, aspect: f32, t: f32, channel: f32) -> f32 {
        let p = Vec2::new((uv.x - 0.5) * aspect, uv.y - 0.5);
        let phase = Vec2::new(3.7, 1.9) * channel;
        if self.mode < 0.5 {
            let q = p * 1.1 * self.scale + phase;
            let n = snoise(q + Vec2::new(t * 0.05, -t * 0.03)) * 0.75 + snoise(q * 2.1 + Vec2::new(-t * 0.04, t * 0.06) + 7.3) * 0.25;
            return smoothstep(-0.25, 0.85, n);
        }
        let drift = (fbm(p * 1.4 + phase + Vec2::new(t * 0.04, -t * 0.03)) - 0.5) * 0.22;
        if self.mode < 1.5 {
            let c = Vec2::new((self.centre.x - 0.5) * aspect, 0.5 - self.centre.y);
            let corner = Vec2::new(aspect * 0.5 + c.x.abs(), 0.5 + c.y.abs());
            return (1.0 - p.distance(c) / (corner.length() * self.radius) + drift).clamp(0.0, 1.0);
        }
        let dir = Vec2::new(self.direction.sin(), self.direction.cos());
        let extent = dir.x.abs() * aspect * 0.5 + dir.y.abs() * 0.5;
        (p.dot(dir) / (2.0 * extent) + 0.5 + drift).clamp(0.0, 1.0)
    }
}

/// One channel's screen of dots: turned by its angle and sliding along it.
struct Screen {
    sin: f32,
    cos: f32,
    slide: Vec2,
    pitch: f32,
}

impl Screen {
    fn new(settings: &Settings, t: f32, channel: usize) -> Self {
        let (sin, cos) = (settings.angle + 30.0 * channel as f32).to_radians().sin_cos();
        let travel = t * settings.drift * 4.0;
        Self { sin, cos, slide: Vec2::new(cos * travel, sin * travel), pitch: settings.pitch }
    }

    /// A pixel's place on the screen, in dots.
    fn grid(&self, px: Vec2) -> Vec2 {
        let v = px + self.slide;
        Vec2::new(self.cos * v.x + self.sin * v.y, -self.sin * v.x + self.cos * v.y) / self.pitch
    }

    /// The centre of the dot `id`, in the box's own 0 to 1 coordinates.
    fn centre(&self, id: Vec2, resolution: Vec2) -> Vec2 {
        let v = id * self.pitch;
        (Vec2::new(self.cos * v.x - self.sin * v.y, self.sin * v.x + self.cos * v.y) - self.slide) / resolution
    }
}

fn bayer2(a: Vec2) -> f32 {
    let a = a.floor();
    let v = a.x / 2.0 + a.y * a.y * 0.75;
    v - v.floor()
}

/// A 4 by 4 ordered threshold, so dot edges dither instead of stair-stepping.
fn bayer(a: Vec2) -> f32 {
    bayer2(0.5 * a) * 0.25 + bayer2(a) + 0.031_25
}

/// How far a dot of source value `value` reaches from its centre, in dots.
fn dot_radius(settings: &Settings, value: f32) -> f32 {
    0.5 * value.sqrt() * settings.intensity
}

/// Whether the dot under this pixel, of the radius `radius` gives for its id, inks it.
fn inked(settings: &Settings, screen: &Screen, px: Vec2, channel: usize, radius: impl FnOnce(Vec2) -> f32) -> bool {
    let q = screen.grid(px);
    let id = q.floor() + 0.5;
    let radius = radius(id);
    let cover = ((radius - (q - id).length()) * settings.pitch + 0.5).clamp(0.0, 1.0);
    cover >= bayer(px + Vec2::new(2.0, 1.0) * channel as f32)
}

/// Every ink that covers a pixel, one bit each, as a tone that picks its palette entry exactly.
fn tone(settings: &Settings, uv: Vec2, resolution: Vec2, mut inked_by: impl FnMut(usize, Vec2) -> bool) -> f32 {
    let px = uv * resolution - 0.5;
    let index: usize = (0..settings.channels).filter(|&channel| inked_by(channel, px)).map(|channel| 1 << channel).sum();
    index as f32 / ((1 << settings.channels) - 1) as f32
}

impl Field for Halftone {
    fn tone(&self, cell: &Cell) -> f32 {
        let settings = Settings::new(cell.params);
        let aspect = cell.resolution.x / cell.resolution.y;
        tone(&settings, cell.uv, cell.resolution, |channel, px| {
            let screen = Screen::new(&settings, cell.time, channel);
            inked(&settings, &screen, px, channel, |id| dot_radius(&settings, settings.source(screen.centre(id, cell.resolution), aspect, cell.time, channel as f32)))
        })
    }

    fn prepare(&self, frame: &Frame) -> Option<Box<dyn Field + '_>> {
        Some(Box::new(Print::new(frame)))
    }
}

/// One channel's dots for a frame: the radius of every dot that touches the box.
struct Dots {
    screen: Screen,
    first: (i32, i32),
    width: usize,
    radii: Vec<f32>,
}

impl Dots {
    fn new(settings: &Settings, frame: &Frame, channel: usize) -> Self {
        let screen = Screen::new(settings, frame.time, channel);
        let resolution = frame.resolution;
        let corners = [Vec2::splat(-0.5), Vec2::new(resolution.x - 0.5, -0.5), Vec2::new(-0.5, resolution.y - 0.5), resolution - 0.5];
        let grid = corners.map(|corner| screen.grid(corner).floor());
        let low = grid.iter().copied().fold(Vec2::splat(f32::INFINITY), Vec2::min);
        let high = grid.iter().copied().fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max);
        let (width, height) = ((high.x - low.x) as usize + 1, (high.y - low.y) as usize + 1);
        let aspect = resolution.x / resolution.y;
        let radii = (0..width * height)
            .map(|index| {
                let id = low + Vec2::new((index % width) as f32, (index / width) as f32) + 0.5;
                dot_radius(settings, settings.source(screen.centre(id, resolution), aspect, frame.time, channel as f32))
            })
            .collect();
        Self { screen, first: (low.x as i32, low.y as i32), width, radii }
    }

    fn radius(&self, id: Vec2) -> f32 {
        let (x, y) = ((id.x.floor() as i32 - self.first.0) as usize, (id.y.floor() as i32 - self.first.1) as usize);
        self.radii.get(y * self.width + x).copied().unwrap_or(0.0)
    }
}

/// A frame of the halftone with every dot's size worked out once, instead of in each of its pixels.
struct Print {
    settings: Settings,
    channels: Vec<Dots>,
}

impl Print {
    fn new(frame: &Frame) -> Self {
        let settings = Settings::new(frame.params);
        let channels = (0..settings.channels).map(|channel| Dots::new(&settings, frame, channel)).collect();
        Self { settings, channels }
    }
}

impl Field for Print {
    fn tone(&self, cell: &Cell) -> f32 {
        tone(&self.settings, cell.uv, cell.resolution, |channel, px| {
            let dots = &self.channels[channel];
            inked(&self.settings, &dots.screen, px, channel, |id| dots.radius(id))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Palette;

    const RESOLUTION: Vec2 = Vec2::new(160.0, 100.0);

    fn params(mode: f32, channels: f32) -> [f32; 11] {
        [mode, 0.5, 0.5, 1.0, 0.0, 1.0, 0.9, 1.0, 45.0, 10.0, channels]
    }

    fn image(field: &dyn Field, params: &[f32]) -> Vec<f32> {
        let palette = Palette::new(&[]);
        (0..(RESOLUTION.x * RESOLUTION.y) as usize)
            .map(|index| {
                let uv = Vec2::new(((index % RESOLUTION.x as usize) as f32 + 0.5) / RESOLUTION.x, ((index / RESOLUTION.x as usize) as f32 + 0.5) / RESOLUTION.y);
                field.tone(&Cell { uv, resolution: RESOLUTION, time: 3.0, params, palette: &palette })
            })
            .collect()
    }

    fn prepared(params: &[f32]) -> Vec<f32> {
        let print = Halftone.prepare(&Frame { resolution: RESOLUTION, time: 3.0, params }).unwrap();
        image(print.as_ref(), params)
    }

    #[test]
    fn a_prepared_frame_prints_the_same_dots() {
        for params in [params(0.0, 1.0), params(1.0, 2.0), params(2.0, 3.0)] {
            assert_eq!(prepared(&params), image(&Halftone, &params));
        }
    }

    #[test]
    fn radial_dots_are_largest_in_the_middle() {
        let tones = prepared(&params(1.0, 1.0));
        let inked = |from: usize, to: usize| (from..to).flat_map(|x| (0..100).map(move |y| y * 160 + x)).filter(|&index| tones[index] > 0.0).count();
        assert!(inked(60, 100) > inked(0, 40));
    }

    #[test]
    fn each_overlap_picks_its_own_palette_entry() {
        let tones = prepared(&params(0.0, 2.0));
        assert!(tones.iter().all(|t| [0.0, 1.0 / 3.0, 2.0 / 3.0, 1.0].iter().any(|entry| (t - entry).abs() < 1e-6)));
        assert!(tones.iter().any(|&t| (t - 1.0).abs() < 1e-6), "the screens overlap somewhere");
    }
}
