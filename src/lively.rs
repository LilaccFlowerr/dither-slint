use slint::{Color, Image, Rgba8Pixel};

use crate::Matrix;
use crate::render::{paint, pixel};

fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn draw(layer: &Layer, tone: impl Fn(f32, f32) -> f32) -> Image {
    let (columns, rows) = (layer.columns.max(1), layer.rows.max(1));
    let (ink, clear) = (pixel(layer.color), Rgba8Pixel::new(0, 0, 0, 0));
    paint(columns, rows, |x, y| {
        let inked = layer.matrix.is_inked(x as i32, y as i32, tone(x as f32, y as f32));
        if inked { ink } else { clear }
    })
}

pub fn sweep(layer: &Layer, t: f32) -> Image {
    let Layer { columns, rows, strength, .. } = *layer;
    let t = t.clamp(0.0, 1.0);
    let width = (columns as f32 * 0.18).max(6.0);
    let front = -width + t * (columns as f32 + rows as f32 + width * 2.0);
    let fade = (std::f32::consts::PI * t).sin();
    draw(layer, |x, y| {
        let d = (x + y * 0.6 - front).abs() / width;
        if d < 1.0 { (1.0 - d * d) * strength * fade } else { 0.0 }
    })
}

pub fn ripple(layer: &Layer, (cx, cy): (f32, f32), t: f32) -> Image {
    let Layer { columns, rows, strength, .. } = *layer;
    let t = t.clamp(0.0, 1.0);
    let far = cx.max(columns as f32 - cx).hypot(cy.max(rows as f32 - cy));
    let radius = far * ease_out(t);
    let band = 2.0 + 4.0 * (1.0 - t);
    let peak = strength * (1.0 - t);
    draw(layer, |x, y| {
        let d = ((x - cx).hypot(y - cy) - radius).abs() / band;
        if d < 1.0 { (1.0 - d) * peak } else { 0.0 }
    })
}

fn hash(x: i32, y: i32, seed: i32) -> f32 {
    let mut h = (x as u32)
        .wrapping_mul(374761393)
        .wrapping_add((y as u32).wrapping_mul(668265263))
        .wrapping_add((seed as u32).wrapping_mul(1442695041));
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    ((h ^ (h >> 16)) as f64 / 4294967296.0) as f32
}

pub fn scan(layer: &Layer, t: f32) -> Image {
    let Layer { rows, strength, .. } = *layer;
    let t = t.clamp(0.0, 1.0);
    let front = -2.0 + ease_out(t) * (rows as f32 + 4.0);
    let trail = (rows as f32 * 0.6).max(4.0);
    let fade = 1.0 - t.powi(4);
    draw(layer, |_, y| {
        let d = front - y;
        if d < 0.0 || d > trail {
            return 0.0;
        }
        (if d < 1.5 { 1.0 } else { 0.6 * (1.0 - d / trail) }) * strength * fade
    })
}

pub fn trace(layer: &Layer, t: f32) -> Image {
    let Layer { columns, rows, strength, .. } = *layer;
    let t = t.clamp(0.0, 1.0);
    let (w, h) = (columns as i32, rows as i32);
    let perimeter = 2.0 * (columns + rows) as f32;
    let tail = perimeter * 0.3;
    let head = t * (perimeter + tail);
    draw(layer, |x, y| {
        let (x, y) = (x as i32, y as i32);
        let (top, right, bottom, left) = (y, w - 1 - x, h - 1 - y, x);
        let near = top.min(right).min(bottom).min(left);
        if near >= 3 {
            return 0.0;
        }
        let along = if near == top {
            x
        } else if near == right {
            w + y
        } else if near == bottom {
            w + h + right
        } else {
            2 * w + h + bottom
        };
        let behind = head - along as f32;
        if behind < 0.0 || behind > tail {
            return 0.0;
        }
        let weight = match near {
            0 => 1.2,
            1 => 0.75,
            _ => 0.35,
        };
        strength * (1.0 - behind / tail) * weight
    })
}

pub fn noise(layer: &Layer, t: f32) -> Image {
    let Layer { columns, rows, strength, color, .. } = *layer;
    let t = t.clamp(0.0, 1.0);
    let frame = (t * 480.0 / 45.0).floor() as i32;
    let density = (std::f32::consts::PI * t).sin() * strength * 0.22;
    let (ink, clear) = (pixel(color), Rgba8Pixel::new(0, 0, 0, 0));
    paint(columns.max(1), rows.max(1), |x, y| {
        if hash(x as i32, y as i32, frame) < density { ink } else { clear }
    })
}

pub fn spotlight(layer: &Layer, (x, y): (f32, f32), level: f32) -> Image {
    let Layer { columns, rows, strength, .. } = *layer;
    let radius = ((columns as f32).hypot(rows as f32) * 0.3).clamp(8.0, 60.0);
    draw(layer, |cx, cy| {
        let d = (cx - x).hypot(cy - y) / radius;
        if d < 1.0 { level * strength * 0.8 * (1.0 - d) * (1.0 + d) } else { 0.0 }
    })
}

pub fn rise(layer: &Layer, level: f32) -> Image {
    let Layer { rows, strength, .. } = *layer;
    draw(layer, |_, y| {
        let u = (y + 1.0) / rows as f32;
        level * strength * 0.6 * u * u
    })
}

pub fn flash(layer: &Layer, (cx, cy): (f32, f32), t: f32) -> Image {
    let Layer { columns, rows, strength, .. } = *layer;
    let t = t.clamp(0.0, 1.0);
    let peak = strength * 0.85 * (1.0 - t).powi(2);
    let reach = (columns as f32).hypot(rows as f32) * 0.6;
    draw(layer, |x, y| {
        let near = (1.0 - (x - cx).hypot(y - cy) / reach).max(0.0);
        peak * (0.45 + 0.55 * near)
    })
}

pub fn pulse(layer: &Layer, (cx, cy): (f32, f32), t: f32) -> Image {
    let Layer { columns, rows, strength, .. } = *layer;
    let t = t.clamp(0.0, 1.0);
    let reach = cx.max(columns as f32 - cx).max(cy).max(rows as f32 - cy);
    draw(layer, |x, y| {
        let distance = (x - cx).abs().max((y - cy).abs());
        (0..2)
            .map(|ring| {
                let u = t * 1.3 - ring as f32 * 0.3;
                if u <= 0.0 || u >= 1.0 {
                    return 0.0;
                }
                let d = (distance - reach * ease_out(u)).abs() / 1.6;
                if d < 1.0 { (1.0 - d) * strength * (1.0 - u) } else { 0.0 }
            })
            .fold(0.0, f32::max)
    })
}

pub fn burst(layer: &Layer, (cx, cy): (f32, f32), t: f32, seed: i32) -> Image {
    let Layer { columns, rows, strength, color, .. } = *layer;
    let t = t.clamp(0.0, 1.0);
    let reach = (columns.min(rows) as f32 * 0.8 + 6.0).clamp(10.0, 48.0);
    let blink = (t * 640.0 / 40.0).floor() as i32;
    let mut lit = vec![false; (columns.max(1) * rows.max(1)) as usize];
    let mut set = |x: i32, y: i32| {
        if x >= 0 && y >= 0 && (x as u32) < columns && (y as u32) < rows {
            lit[y as usize * columns as usize + x as usize] = true;
        }
    };
    for i in 0..12 {
        let angle = i as f32 / 12.0 * std::f32::consts::TAU + (hash(i, 0, seed) - 0.5) * 0.5;
        let r = reach * (0.55 + hash(i, 1, seed) * 0.6) * ease_out(t);
        let (x, y) = ((cx + angle.cos() * r).round() as i32, (cy + angle.sin() * r).round() as i32);
        if hash(i, blink, seed) > (1.0 - t) * 1.4 {
            continue;
        }
        set(x, y);
        if t < 0.45 {
            set(x + 1, y);
            set(x, y + 1);
            set(x + 1, y + 1);
        }
        let (tx, ty) = ((cx + angle.cos() * r * 0.7).round() as i32, (cy + angle.sin() * r * 0.7).round() as i32);
        if t < 0.7 && hash(i, blink + 1, seed) < strength {
            set(tx, ty);
        }
    }
    let (ink, clear) = (pixel(color), Rgba8Pixel::new(0, 0, 0, 0));
    paint(columns.max(1), rows.max(1), |x, y| {
        if lit[(y * columns.max(1) + x) as usize] { ink } else { clear }
    })
}

pub fn glitch(layer: &Layer, t: f32, seed: i32) -> Image {
    let Layer { columns, rows, strength, .. } = *layer;
    let t = t.clamp(0.0, 1.0);
    let tick = (t * 380.0 / 40.0).floor() as i32;
    let tone = strength * (1.1 - t * 0.7);
    let bars: Vec<(u32, u32, u32, u32)> = (0..5)
        .filter(|&bar| hash(bar, tick, seed) >= 0.3)
        .map(|bar| {
            let y0 = (hash(bar, tick + 7, seed) * rows as f32).floor() as u32;
            let height = 1 + (hash(bar, tick + 3, seed) * 3.0).floor() as u32;
            let x0 = (hash(bar, tick + 11, seed) * columns as f32 * 0.7).floor() as u32;
            let length = (columns as f32 * (0.2 + hash(bar, tick + 5, seed) * 0.6)).ceil() as u32;
            (x0, y0, length, height)
        })
        .collect();
    draw(layer, |x, y| {
        let (x, y) = (x as u32, y as u32);
        let inside = bars.iter().any(|&(x0, y0, length, height)| x >= x0 && x < x0 + length && y >= y0 && y < y0 + height);
        if inside { tone } else { 0.0 }
    })
}

pub fn breathe(layer: &Layer, phase: f32) -> Image {
    let Layer { columns, rows, strength, .. } = *layer;
    let b = 0.5 - 0.5 * (std::f32::consts::TAU * phase).cos();
    let depth = (columns.min(rows) as f32 * 0.18).clamp(3.0, 8.0);
    draw(layer, |x, y| {
        let edge = x.min(y).min(columns as f32 - 1.0 - x).min(rows as f32 - 1.0 - y);
        if edge < depth { strength * 0.6 * b * (1.0 - edge / depth) } else { 0.0 }
    })
}

pub struct Star {
    pub x: f32,
    pub y: f32,
    pub phase: f32,
}

pub struct Around {
    pub x: f32,
    pub y: f32,
    pub reach: f32,
}

pub fn scatter(count: usize, width: f32, height: f32, around: Option<&Around>) -> Vec<Star> {
    (0..count as i64)
        .map(|i| {
            let seed = i * 67 + 13;
            let u = ((seed * 53 + 7) % 997) as f32 / 997.0;
            let v = ((seed * 31 + 3) % 991) as f32 / 991.0;
            let phase = ((seed * 41) % 360) as f32;
            let Some(around) = around else {
                return Star { x: u * width, y: v * height, phase };
            };
            let angle = u * std::f32::consts::TAU;
            let distance = v * v * around.reach;
            Star {
                x: (around.x * width + angle.cos() * distance).clamp(0.0, width - 1.0),
                y: (around.y * height + angle.sin() * distance).clamp(0.0, height - 1.0),
                phase,
            }
        })
        .collect()
}

pub fn twinkle_count(columns: u32, rows: u32, cell: f32) -> usize {
    let area = columns as f32 * rows as f32 * cell * cell;
    (area / 1400.0).round().clamp(3.0, 40.0) as usize
}

pub fn sparkles(layer: &Layer, cell: f32, stars: &[Star], tick: f32, excited: bool) -> Image {
    let Layer { columns, rows, color, .. } = *layer;
    let (columns, rows) = (columns.max(1), rows.max(1));
    let ink = pixel(color);
    let with_alpha = |alpha: f32| Rgba8Pixel::new(ink.r, ink.g, ink.b, (alpha * 255.0).round().clamp(0.0, 255.0) as u8);
    let mut cells = vec![Rgba8Pixel::new(0, 0, 0, 0); (columns * rows) as usize];
    let mut put = |x: i32, y: i32, value: Rgba8Pixel| {
        if x >= 0 && y >= 0 && (x as u32) < columns && (y as u32) < rows {
            cells[y as usize * columns as usize + x as usize] = value;
        }
    };
    let intensity = if excited { 1.0 } else { 0.0 };
    for star in stars {
        let wink = ((tick + star.phase) * 0.35).sin().mul_add(0.5, 0.5);
        let lift = wink * (0.7 + 0.3 * intensity);
        if lift < 0.55 {
            continue;
        }
        let (x, y) = ((star.x / cell).floor() as i32, (star.y / cell).floor() as i32);
        put(x, y, with_alpha(lift));
        if wink > 0.9 {
            let glint = with_alpha(lift * 0.6 * (wink - 0.9) * 10.0);
            put(x - 1, y, glint);
            put(x + 1, y, glint);
            put(x, y - 1, glint);
            put(x, y + 1, glint);
        }
    }
    paint(columns, rows, |x, y| cells[(y * columns + x) as usize])
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Hover {
    Sweep,
    Spotlight,
    Scan,
    Trace,
    Rise,
    Noise,
    None,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Press {
    Ripple,
    Burst,
    Flash,
    Pulse,
    Glitch,
    None,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Idle {
    None,
    Shimmer,
    Breathe,
    Twinkle,
}

#[derive(Clone, Copy)]
pub struct Layer {
    pub columns: u32,
    pub rows: u32,
    pub strength: f32,
    pub color: Color,
    pub matrix: Matrix,
}

impl Layer {
    pub fn new(columns: i32, rows: i32, strength: f32, color: Color, matrix: Matrix) -> Self {
        Self { columns: columns.max(0) as u32, rows: rows.max(0) as u32, strength, color, matrix }
    }
}

pub fn hover(kind: Hover, layer: &Layer, t: f32, at: (f32, f32), level: f32) -> Image {
    match kind {
        Hover::Sweep => sweep(layer, t),
        Hover::Spotlight => spotlight(layer, at, level),
        Hover::Scan => scan(layer, t),
        Hover::Trace => trace(layer, t),
        Hover::Rise => rise(layer, level),
        Hover::Noise => noise(layer, t),
        Hover::None => draw(layer, |_, _| 0.0),
    }
}

pub fn press(kind: Press, layer: &Layer, t: f32, at: (f32, f32), seed: i32) -> Image {
    match kind {
        Press::Ripple => ripple(layer, at, t),
        Press::Burst => burst(layer, at, t, seed),
        Press::Flash => flash(layer, at, t),
        Press::Pulse => pulse(layer, at, t),
        Press::Glitch => glitch(layer, t, seed),
        Press::None => draw(layer, |_, _| 0.0),
    }
}

pub fn idle(kind: Idle, layer: &Layer, t: f32, cell: f32) -> Image {
    match kind {
        Idle::Shimmer => sweep(&Layer { strength: layer.strength * 0.55, ..*layer }, t),
        Idle::Breathe => breathe(layer, t),
        Idle::Twinkle => {
            let stars = scatter(twinkle_count(layer.columns, layer.rows, cell), layer.columns as f32 * cell, layer.rows as f32 * cell, None);
            sparkles(layer, cell, &stars, t, false)
        }
        Idle::None => draw(layer, |_, _| 0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layer(columns: u32, rows: u32, strength: f32) -> Layer {
        Layer { columns, rows, strength, color: Color::from_rgb_u8(255, 255, 255), matrix: Matrix::Bayer4 }
    }

    fn inked(image: &Image) -> usize {
        image.to_rgba8().unwrap().as_slice().iter().filter(|p| p.a > 0).count()
    }

    fn pixels(image: &Image) -> Vec<bool> {
        image.to_rgba8().unwrap().as_slice().iter().map(|p| p.a > 0).collect()
    }

    #[test]
    fn sweep_is_empty_at_both_ends() {
        let layer = layer(40, 18, 0.85);
        assert_eq!(inked(&sweep(&layer, 0.0)), 0);
        assert_eq!(inked(&sweep(&layer, 1.0)), 0);
        assert!(inked(&sweep(&layer, 0.5)) > 0);
    }

    #[test]
    fn sweep_moves_left_to_right() {
        let layer = layer(60, 10, 1.0);
        let centre = |t: f32| {
            let lit: Vec<usize> = pixels(&sweep(&layer, t)).iter().enumerate().filter(|(_, on)| **on).map(|(i, _)| i % 60).collect();
            lit.iter().sum::<usize>() as f32 / lit.len() as f32
        };
        assert!(centre(0.3) < centre(0.6));
    }

    #[test]
    fn ripple_fades_out() {
        let layer = layer(40, 18, 0.85);
        assert!(inked(&ripple(&layer, (20.0, 9.0), 0.2)) > 0);
        assert_eq!(inked(&ripple(&layer, (20.0, 9.0), 1.0)), 0);
    }

    #[test]
    fn scan_is_empty_at_both_ends() {
        let layer = layer(40, 18, 0.85);
        assert_eq!(inked(&scan(&layer, 0.0)), 0);
        assert_eq!(inked(&scan(&layer, 1.0)), 0);
        assert!(inked(&scan(&layer, 0.4)) > 0);
    }

    #[test]
    fn trace_stays_on_the_edge() {
        let lit = pixels(&trace(&layer(40, 18, 1.0), 0.5));
        assert!(lit.iter().any(|on| *on));
        for (i, _) in lit.iter().enumerate().filter(|(_, on)| **on) {
            let (x, y) = (i % 40, i / 40);
            assert!(x.min(39 - x).min(y).min(17 - y) < 3);
        }
    }

    #[test]
    fn noise_flickers_in_the_middle() {
        let layer = layer(40, 18, 0.85);
        assert_eq!(inked(&noise(&layer, 0.0)), 0);
        assert!(inked(&noise(&layer, 0.5)) > 0);
    }

    #[test]
    fn spotlight_needs_the_pointer_to_have_arrived() {
        let layer = layer(40, 18, 0.85);
        assert_eq!(inked(&spotlight(&layer, (20.0, 9.0), 0.0)), 0);
        assert!(inked(&spotlight(&layer, (20.0, 9.0), 1.0)) > 0);
    }

    #[test]
    fn rise_is_denser_at_the_bottom() {
        let lit = pixels(&rise(&layer(40, 18, 1.0), 1.0));
        let row = |y: usize| lit[y * 40..(y + 1) * 40].iter().filter(|on| **on).count();
        assert!(row(17) > row(2));
    }

    #[test]
    fn press_effects_fade_out() {
        let layer = layer(40, 18, 0.85);
        assert!(inked(&flash(&layer, (20.0, 9.0), 0.2)) > 0);
        assert_eq!(inked(&flash(&layer, (20.0, 9.0), 1.0)), 0);
        assert!(inked(&pulse(&layer, (20.0, 9.0), 0.3)) > 0);
        assert_eq!(inked(&pulse(&layer, (20.0, 9.0), 1.0)), 0);
    }

    #[test]
    fn burst_sparks_start_at_the_press_point() {
        let layer = layer(40, 18, 0.85);
        assert_eq!(inked(&burst(&layer, (20.0, 9.0), 1.0, 1)), 0);
        assert!(inked(&burst(&layer, (20.0, 9.0), 0.1, 1)) > 0);
    }

    #[test]
    fn glitch_is_stable_per_seed() {
        let layer = layer(40, 18, 0.85);
        assert_eq!(pixels(&glitch(&layer, 0.3, 5)), pixels(&glitch(&layer, 0.3, 5)));
    }

    #[test]
    fn breathe_glows_only_at_the_edge() {
        let layer = layer(40, 18, 1.0);
        assert_eq!(inked(&breathe(&layer, 0.0)), 0);
        let lit = pixels(&breathe(&layer, 0.5));
        assert!(lit.iter().any(|on| *on));
        assert!(!lit[9 * 40 + 20]);
    }

    #[test]
    fn scatter_is_stable_and_inside_the_box() {
        let stars = scatter(24, 100.0, 50.0, None);
        assert_eq!(stars.len(), 24);
        assert!(stars.iter().all(|s| (0.0..100.0).contains(&s.x) && (0.0..50.0).contains(&s.y)));
        let again = scatter(24, 100.0, 50.0, None);
        assert!(stars.iter().zip(&again).all(|(a, b)| a.x == b.x && a.y == b.y && a.phase == b.phase));
    }

    #[test]
    fn twinkle_count_is_between_three_and_forty() {
        assert_eq!(twinkle_count(10, 10, 2.0), 3);
        assert_eq!(twinkle_count(500, 500, 2.0), 40);
    }

    #[test]
    fn stars_wink_in_and_out() {
        let layer = layer(40, 18, 0.7);
        let stars = scatter(24, 80.0, 36.0, None);
        let shown: Vec<usize> = (0..40).map(|tick| inked(&sparkles(&layer, 2.0, &stars, tick as f32, false))).collect();
        assert!(shown.iter().max() > Some(&0));
        assert!(shown.iter().min() < shown.iter().max());
    }

    #[test]
    fn none_draws_nothing() {
        let layer = layer(40, 18, 0.7);
        assert_eq!(inked(&hover(Hover::None, &layer, 0.5, (0.0, 0.0), 1.0)), 0);
        assert_eq!(inked(&press(Press::None, &layer, 0.5, (0.0, 0.0), 1)), 0);
        assert_eq!(inked(&idle(Idle::None, &layer, 0.5, 2.0)), 0);
    }

    #[test]
    fn every_effect_draws_something() {
        let layer = layer(60, 24, 0.9);
        for kind in [Hover::Sweep, Hover::Spotlight, Hover::Scan, Hover::Trace, Hover::Rise, Hover::Noise] {
            assert!(inked(&hover(kind, &layer, 0.4, (30.0, 12.0), 1.0)) > 0, "{kind:?}");
        }
        for kind in [Press::Ripple, Press::Burst, Press::Flash, Press::Pulse, Press::Glitch] {
            assert!(inked(&press(kind, &layer, 0.3, (30.0, 12.0), 3)) > 0, "{kind:?}");
        }
        for (kind, t) in [(Idle::Shimmer, 0.5), (Idle::Breathe, 0.5), (Idle::Twinkle, 9.0)] {
            assert!(inked(&idle(kind, &layer, t, 2.0)) > 0, "{kind:?}");
        }
    }

    #[test]
    fn the_matrix_changes_the_grain() {
        let bayer = layer(40, 18, 0.85);
        let noise = Layer { matrix: Matrix::BlueNoise, ..bayer };
        assert_ne!(pixels(&sweep(&bayer, 0.5)), pixels(&sweep(&noise, 0.5)));
    }
}
