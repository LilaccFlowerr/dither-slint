use glam::Vec2;

use super::{param, smoothstep};
use crate::field::{Cell, Field, Frame};
use crate::noise::snoise;

pub struct Ridgelines;

const TOP: f32 = 0.86;
const BOTTOM: f32 = 0.1;

struct Settings {
    lines: f32,
    amplitude: f32,
    spread: f32,
    shade: f32,
    time: f32,
}

impl Settings {
    // params: lines, amplitude, spread, shade
    fn new(params: &[f32], time: f32) -> Self {
        Self {
            lines: param(params, 0).round().clamp(6.0, 64.0),
            amplitude: param(params, 1).max(0.0),
            spread: param(params, 2).clamp(0.1, 1.0),
            shade: param(params, 3).clamp(0.0, 0.5),
            time,
        }
    }

    /// How many lines fit, at least 4.5 cells apart, so a short box draws fewer.
    fn count(&self, resolution: Vec2) -> usize {
        self.lines.min(((TOP - BOTTOM) * resolution.y / 4.5).floor() + 1.0) as usize
    }

    /// Peaks keep their shape in narrow boxes instead of crowding together.
    fn frequency(resolution: Vec2) -> f32 {
        (resolution.x / resolution.y / 1.3).clamp(0.45, 1.4)
    }

    /// Height of line `i` at `x`, in line spacings: flat at the ends, peaked in the middle.
    fn height(&self, x: f32, i: f32, frequency: f32) -> f32 {
        let t = self.time;
        let centred = (x - 0.5) / self.spread;
        let envelope = (-centred * centred * 5.5).exp();
        let n1 = snoise(Vec2::new(x * 7.0 * frequency + i * 0.37, i * 0.61 + t * 0.05));
        let n2 = snoise(Vec2::new(x * 17.0 * frequency - i * 0.53, i * 1.3 - t * 0.07));
        let peaks = (n1 * 0.8 + n2 * 0.25 + 0.1).max(0.0);
        let ripple = 0.1 * snoise(Vec2::new(x * 36.0 * frequency, i * 2.1 + t * 0.03));
        envelope * (peaks * peaks * 4.5 + ripple + 0.25) * self.amplitude
    }
}

/// Back to front, each line hides the ones behind it under its own paper fill.
/// `height(i, next)` gives line `i` at this cell's column, or the next column.
fn shade(settings: &Settings, uv: Vec2, resolution: Vec2, height: impl Fn(usize, bool) -> f32) -> f32 {
    let count = settings.count(resolution);
    let spacing = (TOP - BOTTOM) / (count as f32 - 1.0).max(1.0);
    let reach = 6.3 * settings.amplitude * spacing + 1.0 / resolution.y;
    let mut tone = 0.0;
    for index in 0..count {
        let base = TOP - index as f32 * spacing;
        if uv.y > base + reach {
            continue;
        }
        if base - uv.y > 6.0 * spacing {
            tone = 0.0;
            continue;
        }
        let h = height(index, false);
        let y = base + h * spacing;
        let under = (y - uv.y) / spacing;
        let fill = settings.shade * smoothstep(0.6, 3.0, h) * (-under * 0.7).exp();
        if under > 1.0 {
            tone = fill;
            continue;
        }
        let slope = (height(index, true) - h) * spacing * resolution.y;
        let d = (uv.y - y).abs() * resolution.y / (1.0 + slope * slope).sqrt();
        if uv.y < y {
            tone = fill;
        }
        if d <= 0.5 {
            tone = 1.0;
        }
    }
    tone
}

impl Field for Ridgelines {
    fn tone(&self, cell: &Cell) -> f32 {
        let settings = Settings::new(cell.params, cell.time);
        let frequency = Settings::frequency(cell.resolution);
        let x = cell.uv.x;
        let step = 1.0 / cell.resolution.x;
        shade(&settings, cell.uv, cell.resolution, |index, next| settings.height(if next { x + step } else { x }, index as f32, frequency))
    }

    fn prepare(&self, frame: &Frame) -> Option<Box<dyn Field + '_>> {
        Some(Box::new(Plot::new(frame)))
    }
}

/// One frame of the ridges, with every line's height worked out once per column.
struct Plot {
    settings: Settings,
    columns: usize,
    heights: Vec<f32>,
}

impl Plot {
    fn new(frame: &Frame) -> Self {
        let settings = Settings::new(frame.params, frame.time);
        let frequency = Settings::frequency(frame.resolution);
        let count = settings.count(frame.resolution);
        let columns = frame.resolution.x as usize + 1;
        let heights = (0..count * columns)
            .map(|index| {
                let (line, column) = (index / columns, index % columns);
                settings.height((column as f32 + 0.5) / frame.resolution.x, line as f32, frequency)
            })
            .collect();
        Self { settings, columns, heights }
    }
}

impl Field for Plot {
    fn tone(&self, cell: &Cell) -> f32 {
        let column = ((cell.uv.x * cell.resolution.x - 0.5).round().max(0.0) as usize).min(self.columns - 2);
        shade(&self.settings, cell.uv, cell.resolution, |index, next| self.heights[index * self.columns + column + next as usize])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Palette;

    const RESOLUTION: Vec2 = Vec2::new(160.0, 120.0);
    const DEFAULTS: [f32; 4] = [32.0, 1.0, 0.5, 0.25];

    fn image(field: &dyn Field, time: f32) -> Vec<f32> {
        let palette = Palette::new(&[]);
        (0..(RESOLUTION.x * RESOLUTION.y) as usize)
            .map(|index| {
                let uv = Vec2::new(((index % 160) as f32 + 0.5) / RESOLUTION.x, ((index / 160) as f32 + 0.5) / RESOLUTION.y);
                field.tone(&Cell { uv, resolution: RESOLUTION, time, params: &DEFAULTS, palette: &palette })
            })
            .collect()
    }

    #[test]
    fn a_prepared_frame_draws_the_same_ridges() {
        let plot = Ridgelines.prepare(&Frame { resolution: RESOLUTION, time: 8.0, params: &DEFAULTS }).unwrap();
        assert_eq!(image(plot.as_ref(), 8.0), image(&Ridgelines, 8.0));
    }

    #[test]
    fn the_peaks_rise_in_the_middle() {
        let tones = image(&Ridgelines, 8.0);
        let lit = |from: usize, to: usize| (0..120).flat_map(|y| (from..to).map(move |x| y * 160 + x)).filter(|&index| tones[index] > 0.0).count();
        assert!(lit(60, 100) > lit(0, 40));
    }

    #[test]
    fn short_boxes_draw_fewer_lines() {
        let settings = Settings::new(&DEFAULTS, 0.0);
        assert_eq!(settings.count(Vec2::new(200.0, 400.0)), 32);
        assert!(settings.count(Vec2::new(200.0, 40.0)) < 10);
    }
}
