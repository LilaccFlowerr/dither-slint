use slint::{Color, Image, Rgba8Pixel};

use crate::Matrix;
use crate::color::mix_oklab;
use crate::render::paint;

const PILL_RADIUS: f32 = 8.0;
const BAR_CELLS: f32 = 2.0;
const GLOW_SHARE: f32 = 0.32;
const GLOW_PEAK: f32 = 0.36;

const BASE: usize = 0;
const LIGHT: usize = 1;
const DARK: usize = 2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TabsVariant {
    Pill,
    Underline,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl BoxRect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn empty() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            w: 0.0,
            h: 0.0,
        }
    }

    pub fn centre(&self) -> (f32, f32) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }
}

pub struct Ghost {
    pub box_rect: BoxRect,
    pub tone: f32,
}

pub struct Scene {
    pub body: BoxRect,
    pub dir: (f32, f32),
    pub trail: f32,
    pub ghosts: Vec<Ghost>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    pub ink: usize,
    pub tone: f32,
}

const NONE: Sample = Sample {
    ink: BASE,
    tone: 0.0,
};

fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

fn ease_out(v: f32) -> f32 {
    1.0 - (1.0 - clamp01(v)).powi(4)
}

fn ease_in_out(v: f32) -> f32 {
    let x = clamp01(v);
    if x < 0.5 {
        8.0 * x.powi(4)
    } else {
        1.0 - (-2.0 * x + 2.0).powi(4) / 2.0
    }
}

fn smooth(edge0: f32, edge1: f32, v: f32) -> f32 {
    let x = clamp01((v - edge0) / (edge1 - edge0));
    x * x * (3.0 - 2.0 * x)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn edge_progress(facing: f32, progress: f32) -> f32 {
    if facing > 0.0 {
        ease_out(progress / 0.78)
    } else if facing < 0.0 {
        ease_out((progress - 0.14) / 0.86)
    } else {
        ease_out(progress)
    }
}

pub fn glide_scene(from: BoxRect, to: BoxRect, progress: f32) -> Scene {
    let (fx, fy) = from.centre();
    let (tx, ty) = to.centre();
    let length = (tx - fx).hypot(ty - fy);
    let dir = if length < 0.5 {
        (0.0, 0.0)
    } else {
        ((tx - fx) / length, (ty - fy) / length)
    };
    let left = lerp(from.x, to.x, edge_progress(-dir.0, progress));
    let right = lerp(from.x + from.w, to.x + to.w, edge_progress(dir.0, progress));
    let top = lerp(from.y, to.y, edge_progress(-dir.1, progress));
    let bottom = lerp(from.y + from.h, to.y + to.h, edge_progress(dir.1, progress));
    Scene {
        body: BoxRect {
            x: left,
            y: top,
            w: (right - left).max(0.0),
            h: (bottom - top).max(0.0),
        },
        dir,
        ghosts: vec![
            Ghost {
                box_rect: from,
                tone: 1.0 - ease_out(progress / 0.5),
            },
            Ghost {
                box_rect: to,
                tone: ease_in_out((progress - 0.3) / 0.7),
            },
        ],
        trail: 1.0 - 0.85 * (std::f32::consts::PI * clamp01(progress)).sin().powf(0.7),
    }
}

pub fn rest_scene(box_rect: BoxRect) -> Scene {
    Scene {
        body: box_rect,
        dir: (0.0, 0.0),
        ghosts: Vec::new(),
        trail: 1.0,
    }
}

fn rounded_coverage(b: &BoxRect, px: f32, py: f32, radius: f32, size: f32) -> f32 {
    let r = radius.min(b.w / 2.0).min(b.h / 2.0);
    let qx = (px - (b.x + b.w / 2.0)).abs() - (b.w / 2.0 - r);
    let qy = (py - (b.y + b.h / 2.0)).abs() - (b.h / 2.0 - r);
    let outside = qx.max(0.0).hypot(qy.max(0.0));
    let distance = outside + qx.max(qy).min(0.0) - r;
    clamp01(0.5 - distance / size)
}

pub fn pill_sample(
    b: &BoxRect,
    px: f32,
    py: f32,
    cx: i32,
    cy: i32,
    size: f32,
    matrix: Matrix,
) -> Sample {
    if b.w <= 0.0 || b.h <= 0.0 {
        return NONE;
    }
    let cover = rounded_coverage(b, px, py, PILL_RADIUS, size);
    if cover <= 0.0 {
        return NONE;
    }
    let from_top = (py - b.y) / b.h;
    if from_top > 0.34 && from_top < 0.66 {
        return Sample { ink: BASE, tone: cover };
    }
    let rows = b.h / size;
    let light_length = (rows * 0.24).round().max(2.0);
    let dark_length = (rows * 0.5).round().max(2.0);
    let light = 1.0 - (py - b.y) / size / light_length;
    if matrix.is_inked(cx, cy, light) {
        return Sample { ink: LIGHT, tone: cover };
    }
    let dark = 1.0 - (b.y + b.h - py) / size / dark_length;
    Sample {
        ink: if matrix.is_inked(cx, cy, dark) { DARK } else { BASE },
        tone: cover,
    }
}

pub fn underline_sample(
    b: &BoxRect,
    px: f32,
    py: f32,
    size: f32,
) -> Sample {
    if b.w <= 0.0 || b.h <= 0.0 {
        return NONE;
    }
    let along = (px - b.x) / b.w;
    let across = b.y + b.h - py;
    let depth = b.h;
    if along < 0.0 || along > 1.0 || across < 0.0 || across > depth {
        return NONE;
    }
    let ends = smooth(0.0, 0.62, along.min(1.0 - along) * 2.0);
    let bar = BAR_CELLS * size;
    if across <= bar {
        return Sample { ink: BASE, tone: ends };
    }
    let rise = (across - bar) / (depth * GLOW_SHARE);
    let tone = if rise >= 1.0 {
        0.0
    } else {
        GLOW_PEAK * (1.0 - rise).powf(1.3) * ends
    };
    Sample { ink: BASE, tone }
}

fn along_body(scene: &Scene, px: f32, py: f32) -> f32 {
    let (dx, dy) = scene.dir;
    let half = (dx.abs() * scene.body.w + dy.abs() * scene.body.h) / 2.0;
    if half <= 0.0 {
        return 1.0;
    }
    let (bx, by) = scene.body.centre();
    clamp01(((px - bx) * dx + (py - by) * dy) / (2.0 * half) + 0.5)
}

pub struct Indicator {
    pub columns: u32,
    pub rows: u32,
    pub cell: f32,
    pub variant: TabsVariant,
    pub matrix: Matrix,
    pub color: Color,
    pub from: BoxRect,
    pub to: BoxRect,
    pub progress: f32,
}

pub fn tabs_indicator(indicator: &Indicator) -> Image {
    let Indicator { variant, matrix, color, from, to, .. } = *indicator;
    let (columns, rows) = (indicator.columns.max(1), indicator.rows.max(1));
    let cell = indicator.cell.max(1.0);
    let progress = indicator.progress.clamp(0.0, 1.0);

    let same_box = from.x == to.x && from.y == to.y && from.w == to.w && from.h == to.h;
    let scene = if progress >= 1.0 || same_box || from.w <= 0.0 || from.h <= 0.0 {
        rest_scene(to)
    } else {
        glide_scene(from, to, progress)
    };

    let base_rgba = [
        color.red() as f32 / 255.0,
        color.green() as f32 / 255.0,
        color.blue() as f32 / 255.0,
        color.alpha() as f32 / 255.0,
    ];
    let white = [1.0, 1.0, 1.0, 1.0];
    let black = [0.0, 0.0, 0.0, 1.0];

    let light_rgba = mix_oklab(base_rgba, white, 0.42);
    let dark_rgba = mix_oklab(base_rgba, black, 0.48);

    let to_pixel = |rgba: [f32; 4]| {
        Rgba8Pixel::new(
            (rgba[0] * 255.0).round() as u8,
            (rgba[1] * 255.0).round() as u8,
            (rgba[2] * 255.0).round() as u8,
            (rgba[3] * 255.0).round() as u8,
        )
    };

    let inks = [to_pixel(base_rgba), to_pixel(light_rgba), to_pixel(dark_rgba)];
    let clear = Rgba8Pixel::new(0, 0, 0, 0);

    paint(columns, rows, |cx, cy| {
        let px = (cx as f32 + 0.5) * cell;
        let py = (cy as f32 + 0.5) * cell;

        let sample_fn = |b: &BoxRect| match variant {
            TabsVariant::Pill => pill_sample(b, px, py, cx as i32, cy as i32, cell, matrix),
            TabsVariant::Underline => underline_sample(b, px, py, cell),
        };

        let mut best = sample_fn(&scene.body);
        if best.tone > 0.0 && scene.trail < 1.0 {
            let s = along_body(&scene, px, py);
            best.tone *= lerp(scene.trail, 1.0, smooth(0.0, 0.9, s));
        }

        for ghost in &scene.ghosts {
            if ghost.tone > 0.0 {
                let hit = sample_fn(&ghost.box_rect);
                if hit.tone * ghost.tone > best.tone {
                    best = Sample {
                        ink: hit.ink,
                        tone: hit.tone * ghost.tone,
                    };
                }
            }
        }

        let on = best.tone > 0.0 && matrix.is_inked(cx as i32, cy as i32, best.tone);
        if on {
            inks[best.ink]
        } else {
            clear
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ease_curves() {
        assert_eq!(clamp01(-0.5), 0.0);
        assert_eq!(clamp01(1.5), 1.0);
        assert_eq!(ease_out(0.0), 0.0);
        assert_eq!(ease_out(1.0), 1.0);
        assert!(ease_out(0.5) > 0.5);
        assert_eq!(ease_in_out(0.0), 0.0);
        assert_eq!(ease_in_out(1.0), 1.0);
        assert_eq!(ease_in_out(0.5), 0.5);
    }

    #[test]
    fn glide_scene_stretches_in_motion_direction() {
        let from = BoxRect::new(10.0, 4.0, 60.0, 32.0);
        let to = BoxRect::new(120.0, 4.0, 60.0, 32.0);
        let scene = glide_scene(from, to, 0.35);

        assert_eq!(scene.dir.0, 1.0);
        assert_eq!(scene.dir.1, 0.0);
        // Leading edge moves ahead of trailing edge, stretching the body
        assert!(scene.body.w > 60.0);
        assert!(scene.trail < 1.0);
        assert_eq!(scene.ghosts.len(), 2);
    }

    #[test]
    fn glide_scene_ghosts_fade_appropriately() {
        let from = BoxRect::new(10.0, 4.0, 60.0, 32.0);
        let to = BoxRect::new(120.0, 4.0, 60.0, 32.0);

        let start_scene = glide_scene(from, to, 0.0);
        assert_eq!(start_scene.ghosts[0].tone, 1.0);
        assert_eq!(start_scene.ghosts[1].tone, 0.0);

        let mid_scene = glide_scene(from, to, 0.5);
        assert!(mid_scene.ghosts[0].tone < 0.2);
        assert!(mid_scene.ghosts[1].tone > 0.0);

        let end_scene = glide_scene(from, to, 1.0);
        assert_eq!(end_scene.ghosts[0].tone, 0.0);
        assert_eq!(end_scene.ghosts[1].tone, 1.0);
    }

    #[test]
    fn pill_sample_respects_label_band_and_radius() {
        let b = BoxRect::new(10.0, 10.0, 80.0, 40.0);
        let cell = 2.0;

        // Outside box
        let sample_outside = pill_sample(&b, 0.0, 0.0, 0, 0, cell, Matrix::Bayer4);
        assert_eq!(sample_outside.tone, 0.0);

        // Center label band (50% from top, py = 30.0) -> BASE ink, full coverage
        let sample_center = pill_sample(&b, 50.0, 30.0, 25, 15, cell, Matrix::Bayer4);
        assert_eq!(sample_center.ink, BASE);
        assert_eq!(sample_center.tone, 1.0);

        // Near top edge (py = 11.0) -> light gradient
        let sample_top = pill_sample(&b, 50.0, 11.0, 25, 5, cell, Matrix::Bayer4);
        assert!(sample_top.tone > 0.0);
    }

    #[test]
    fn underline_sample_bar_and_glow() {
        let b = BoxRect::new(10.0, 10.0, 80.0, 40.0);
        let cell = 2.0;

        // Bottom bar (across <= 4.0 px, e.g. py = 48.0)
        let sample_bar = underline_sample(&b, 50.0, 48.0, cell);
        assert_eq!(sample_bar.ink, BASE);
        assert!(sample_bar.tone > 0.9);

        // Far outside
        let sample_out = underline_sample(&b, 5.0, 48.0, cell);
        assert_eq!(sample_out.tone, 0.0);

        // Glow region (e.g. py = 42.0, across = 8.0)
        let sample_glow = underline_sample(&b, 50.0, 42.0, cell);
        assert!(sample_glow.tone > 0.0 && sample_glow.tone < 0.4);
    }

    #[test]
    fn tabs_indicator_renders_image() {
        let color = Color::from_rgb_u8(84, 113, 245);
        let from = BoxRect::new(4.0, 4.0, 64.0, 32.0);
        let to = BoxRect::new(80.0, 4.0, 64.0, 32.0);

        // Rest image
        let img_rest = tabs_indicator(&Indicator {
            columns: 80, rows: 20, cell: 2.0,
            variant: TabsVariant::Pill,
            matrix: Matrix::Bayer4,
            color,
            from,
            to,
            progress: 1.0,
        });
        let buffer = img_rest.to_rgba8().unwrap();
        let inked_count = buffer.as_slice().iter().filter(|p| p.a > 0).count();
        assert!(inked_count > 0);

        // Moving glide image
        let img_move = tabs_indicator(&Indicator {
            columns: 80, rows: 20, cell: 2.0,
            variant: TabsVariant::Pill,
            matrix: Matrix::Bayer4,
            color,
            from,
            to,
            progress: 0.5,
        });
        let buffer_move = img_move.to_rgba8().unwrap();
        let inked_move_count = buffer_move.as_slice().iter().filter(|p| p.a > 0).count();
        assert!(inked_move_count > 0);

        // Underline variant
        let img_underline = tabs_indicator(&Indicator {
            columns: 80, rows: 20, cell: 2.0,
            variant: TabsVariant::Underline,
            matrix: Matrix::Bayer4,
            color,
            from,
            to,
            progress: 0.5,
        });
        let buffer_u = img_underline.to_rgba8().unwrap();
        assert!(buffer_u.as_slice().iter().any(|p| p.a > 0));
    }
}
