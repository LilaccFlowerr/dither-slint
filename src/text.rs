use std::f32::consts::PI;

use slint::{Color, Image, Rgba8Pixel};

use crate::Matrix;
use crate::render::paint;
use crate::style::{FillVariant, texture_inked};

const NOISE_FREQUENCY: f32 = 0.09;
const DRIFT_FREQUENCY: f32 = 1.6;
const DRIFT_SPEED: f32 = 0.07;
const DRIFT_SHEAR: f32 = 0.04;
const DRIFT_STRENGTH: f32 = 0.28;
const NOISE_LOW: f32 = 0.22;
const NOISE_SPAN: f32 = 0.56;
const TEXTURE_DRIFT: f32 = 2.0;
const DEFAULT_GHOST: f32 = 0.2;
const OCTAVES: [(f32, f32, f32, f32); 4] = [(1.0, 0.0, 0.0, 0.5333), (2.03, 17.1, 9.2, 0.2667), (4.01, 3.7, 31.4, 0.1333), (8.05, 41.3, 5.9, 0.0667)];

pub const DISSOLVE_STEPS: usize = 16;
const BAYER2_STEPS: usize = 4;
const DISSOLVE_JITTER: f32 = 78.233;
const SCRAMBLE_JITTER: f32 = 91.17;
const SCRAMBLE_JITTER_OFFSET: f32 = 7.31;
const SCRAMBLE_SPREAD: f32 = 1.4;
const BLOCK_SHARE: f32 = 0.6;
const BLOCK_LEVELS: f32 = 12.0;
const BLOCKS_ONLY_LEVELS: f32 = 13.0;
const LEVELS: i32 = 16;
const SHIMMER_FLOOR: i32 = 1;
const SHIMMER_CEIL: i32 = 14;
const MIN_POOL: usize = 4;
const DIGITS: &str = "0123456789";
const SEED_TICK: f32 = 12.9898;
const SEED_INDEX: f32 = 78.233;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InkFill {
    Gradient,
    Noise,
    Dotted,
    Hatched,
    Solid,
}

#[derive(Clone, Copy, Debug)]
pub struct Ink {
    pub fill: InkFill,
    pub direction: f32,
    pub scale: f32,
    pub from: f32,
    pub to: f32,
    pub ghost: Option<f32>,
    pub matrix: Matrix,
}

fn hash(x: f32, y: f32) -> f32 {
    let value = (x * 127.1 + y * 311.7).sin() * 43_758.547;
    value - value.floor()
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn value_noise(x: f32, y: f32) -> f32 {
    let (ix, iy) = (x.floor(), y.floor());
    let (fx, fy) = (smooth(x - ix), smooth(y - iy));
    let (a, b, c, d) = (hash(ix, iy), hash(ix + 1.0, iy), hash(ix, iy + 1.0), hash(ix + 1.0, iy + 1.0));
    a + (b - a) * fx + (c - a) * fy + (a - b - c + d) * fx * fy
}

fn fbm(x: f32, y: f32) -> f32 {
    OCTAVES.iter().map(|&(scale, offset_x, offset_y, weight)| value_noise(x * scale + offset_x, y * scale + offset_y) * weight).sum()
}

fn direction_vector(degrees: f32) -> (f32, f32) {
    let radians = degrees * PI / 180.0;
    (radians.sin(), -radians.cos())
}

fn tone_at(ink: &Ink, u: f32, v: f32, aspect: f32, time: f32) -> f32 {
    let (dx, dy) = direction_vector(ink.direction);
    let (px, py) = ((u - 0.5) * aspect, v - 0.5);
    if ink.fill == InkFill::Noise {
        let n = fbm(px * ink.scale - time * NOISE_FREQUENCY * dx, py * ink.scale - time * NOISE_FREQUENCY * dy);
        return ink.from + (ink.to - ink.from) * ((n - NOISE_LOW) / NOISE_SPAN).clamp(0.0, 1.0);
    }
    let extent = dx.abs() * aspect * 0.5 + dy.abs() * 0.5;
    let drift = fbm(px * DRIFT_FREQUENCY + time * DRIFT_SPEED, py * DRIFT_FREQUENCY - time * DRIFT_SHEAR) - 0.5;
    let along = ((px * dx + py * dy) / (2.0 * extent) + 0.5 + drift * DRIFT_STRENGTH).clamp(0.0, 1.0);
    ink.from + (ink.to - ink.from) * along
}

pub fn ghost(ink: &Ink) -> f32 {
    ink.ghost.unwrap_or(if matches!(ink.fill, InkFill::Dotted | InkFill::Hatched) { DEFAULT_GHOST } else { 0.0 })
}

pub fn cell_for(font_size: f32) -> f32 {
    2.0f32.max((font_size / 24.0).round())
}

pub fn ink_overlay(columns: u32, rows: u32, ink: &Ink, time: f32, backdrop: Color) -> Image {
    let aspect = columns as f32 / rows.max(1) as f32;
    let (dx, dy) = direction_vector(ink.direction);
    let (shift_x, shift_y) = ((-dx * time * TEXTURE_DRIFT).round() as i32, (-dy * time * TEXTURE_DRIFT).round() as i32);
    let hidden = Rgba8Pixel::new(backdrop.red(), backdrop.green(), backdrop.blue(), ((1.0 - ghost(ink)) * 255.0).round() as u8);
    let clear = Rgba8Pixel::new(0, 0, 0, 0);
    paint(columns, rows, |x, y| {
        let inked = match ink.fill {
            InkFill::Solid => true,
            InkFill::Dotted => texture_inked(FillVariant::Dotted, (x as i32 + shift_x).rem_euclid(4) as usize, (y as i32 + shift_y).rem_euclid(4) as usize),
            InkFill::Hatched => texture_inked(FillVariant::Hatched, (x as i32 + shift_x).rem_euclid(4) as usize, (y as i32 + shift_y).rem_euclid(4) as usize),
            InkFill::Gradient | InkFill::Noise => {
                let tone = tone_at(ink, (x as f32 + 0.5) / columns as f32, (y as f32 + 0.5) / rows as f32, aspect, time);
                ink.matrix.is_inked(x as i32, y as i32, tone.clamp(0.0, 1.0))
            }
        };
        if inked { clear } else { hidden }
    })
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DissolveOrder {
    Start,
    End,
    Center,
    Edges,
    Random,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DissolveFill {
    Solid,
    Gradient,
    Dotted,
    Hatched,
}

fn jitter(index: usize, factor: f32, offset: f32) -> f32 {
    let value = (index as f32 * factor + offset).sin() * 43_758.547;
    value - value.floor()
}

pub fn ranks(count: usize, order: DissolveOrder) -> Vec<f32> {
    let middle = count.saturating_sub(1) as f32 / 2.0;
    let indices = 0..count;
    match order {
        DissolveOrder::Start => indices.map(|index| index as f32).collect(),
        DissolveOrder::End => indices.map(|index| (count - 1 - index) as f32).collect(),
        DissolveOrder::Center => indices.map(|index| (index as f32 - middle).abs()).collect(),
        DissolveOrder::Edges => indices.map(|index| middle - (index as f32 - middle).abs()).collect(),
        DissolveOrder::Random => {
            let mut shuffled: Vec<usize> = indices.collect();
            shuffled.sort_by(|&a, &b| jitter(a, DISSOLVE_JITTER, 12.9898).total_cmp(&jitter(b, DISSOLVE_JITTER, 12.9898)));
            let mut rank = vec![0.0; count];
            for (place, index) in shuffled.into_iter().enumerate() {
                rank[index] = place as f32;
            }
            rank
        }
    }
}

pub fn units(text: &str, word: bool) -> Vec<(String, Option<usize>)> {
    let mut index = 0;
    let mut found = Vec::new();
    let mut run = String::new();
    let flush = |run: &mut String, found: &mut Vec<(String, Option<usize>)>, index: &mut usize| {
        if run.is_empty() {
            return;
        }
        if word {
            found.push((std::mem::take(run), Some(*index)));
            *index += 1;
        } else {
            for character in run.chars() {
                found.push((character.to_string(), Some(*index)));
                *index += 1;
            }
            run.clear();
        }
    };
    for character in text.chars() {
        if character.is_whitespace() {
            flush(&mut run, &mut found, &mut index);
            found.push((character.to_string(), None));
        } else {
            run.push(character);
        }
    }
    flush(&mut run, &mut found, &mut index);
    found
}

pub fn dissolve_steps(matrix: Matrix) -> usize {
    if matrix == Matrix::Bayer2 { BAYER2_STEPS } else { DISSOLVE_STEPS }
}

pub struct Dissolve {
    pub duration: f32,
    pub stagger: f32,
    pub delay: f32,
    pub matrix: Matrix,
    pub fill: DissolveFill,
    pub level: f32,
    pub showing: bool,
}

pub fn dissolve_step(elapsed: f32, rank: f32, options: &Dissolve) -> usize {
    let steps = dissolve_steps(options.matrix);
    let start = options.delay + rank * options.stagger;
    let progress = ((elapsed - start) / options.duration.max(1.0)).clamp(0.0, 1.0);
    let eased = 1.0 - (1.0 - progress).powi(2);
    let reached = ((eased * steps as f32 + 1e-4).floor() as usize).min(steps);
    if options.showing { reached } else { steps - reached }
}

pub fn dissolve_overlay(columns: u32, rows: u32, step: usize, options: &Dissolve, backdrop: Color) -> Image {
    let steps = dissolve_steps(options.matrix);
    let last = ((options.level * steps as f32).round() as usize).clamp(1, steps);
    let hidden = Rgba8Pixel::new(backdrop.red(), backdrop.green(), backdrop.blue(), 255);
    let clear = Rgba8Pixel::new(0, 0, 0, 0);
    paint(columns, rows, |x, y| {
        let (cx, cy) = (x as i32, y as i32);
        let textured = |variant: FillVariant| texture_inked(variant, x as usize, y as usize);
        let visible = match options.fill {
            DissolveFill::Solid => step >= steps || options.matrix.is_inked(cx, cy, step as f32 / steps as f32),
            DissolveFill::Gradient => options.matrix.is_inked(cx, cy, step.min(last) as f32 / steps as f32),
            DissolveFill::Dotted => (step >= steps || options.matrix.is_inked(cx, cy, step as f32 / steps as f32)) && textured(FillVariant::Dotted),
            DissolveFill::Hatched => (step >= steps || options.matrix.is_inked(cx, cy, step as f32 / steps as f32)) && textured(FillVariant::Hatched),
        };
        if visible { clear } else { hidden }
    })
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScrambleMode {
    Dither,
    Blocks,
    Glyphs,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Wait,
    Old,
    Block,
    Glyph,
    Done,
}

pub struct Scramble {
    pub mode: ScrambleMode,
    pub duration: f32,
    pub stagger: f32,
    pub delay: f32,
    pub rate: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Slot {
    pub character: String,
    pub phase: Phase,
    pub level: i32,
    pub glyph: String,
}

pub fn glyph_pool(text: &str, characters: &str) -> Vec<char> {
    let source: Vec<char> = if characters.is_empty() { text.chars().filter(|character| !character.is_whitespace()).collect() } else { characters.chars().collect() };
    let mut pool: Vec<char> = Vec::new();
    for character in source {
        if !pool.contains(&character) {
            pool.push(character);
        }
    }
    if pool.len() < MIN_POOL {
        pool.extend(DIGITS.chars());
    }
    pool
}

fn pick(pool: &[char], index: usize, tick: u32) -> char {
    let seed = (tick as f32 * SEED_TICK + index as f32 * SEED_INDEX).sin() * 43_758.547;
    pool[((seed - seed.floor()) * pool.len() as f32) as usize % pool.len()]
}

fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn frame(order: usize, elapsed: f32, options: &Scramble, before: Phase) -> (Phase, i32) {
    let offset = (jitter(order, SCRAMBLE_JITTER, SCRAMBLE_JITTER_OFFSET) - 0.5) * options.stagger * SCRAMBLE_SPREAD;
    let start = (options.delay + order as f32 * options.stagger + offset).max(0.0);
    let progress = (elapsed - start) / options.duration.max(1.0);
    if progress < 0.0 {
        return (before, 0);
    }
    if progress >= 1.0 {
        return (Phase::Done, LEVELS);
    }
    match options.mode {
        ScrambleMode::Glyphs => (Phase::Glyph, 0),
        ScrambleMode::Blocks => (Phase::Block, ((ease_out(progress) * BLOCKS_ONLY_LEVELS).round() as i32).max(1)),
        ScrambleMode::Dither if progress < BLOCK_SHARE => (Phase::Block, ((ease_out(progress / BLOCK_SHARE) * BLOCK_LEVELS).round() as i32).max(1)),
        ScrambleMode::Dither => (Phase::Glyph, 0),
    }
}

pub fn slots(text: &str, from: &str, keep_same: bool, elapsed: f32, options: &Scramble, pool: &[char]) -> Vec<Slot> {
    let old: Vec<char> = from.chars().collect();
    let tick = (elapsed * options.rate.max(1.0) / 1000.0) as u32;
    let mut order = 0;
    text.chars()
        .enumerate()
        .map(|(index, character)| {
            let previous = old.get(index).copied();
            if character.is_whitespace() || (keep_same && previous == Some(character)) {
                return Slot { character: character.to_string(), phase: Phase::Done, level: LEVELS, glyph: String::new() };
            }
            let before = if previous.is_none_or(char::is_whitespace) { Phase::Wait } else { Phase::Old };
            let (phase, level) = frame(order, elapsed, options, before);
            order += 1;
            let glyph = match phase {
                Phase::Old => previous.map(String::from).unwrap_or_default(),
                Phase::Glyph => pick(pool, index, tick).to_string(),
                _ => String::new(),
            };
            let shimmer = ((jitter(index + tick as usize * 7, SEED_INDEX, SEED_TICK) * 3.0).floor() as i32) - 1;
            Slot { character: character.to_string(), phase, level: if phase == Phase::Block { (level + shimmer).clamp(SHIMMER_FLOOR, SHIMMER_CEIL) } else { level }, glyph }
        })
        .collect()
}

pub fn settled(text: &str, from: &str, keep_same: bool, elapsed: f32, options: &Scramble) -> bool {
    let pool = glyph_pool(text, "");
    slots(text, from, keep_same, elapsed, options, &pool).iter().all(|slot| slot.phase == Phase::Done)
}

pub fn block_overlay(columns: u32, rows: u32, level: i32, color: Color) -> Image {
    let ink = Rgba8Pixel::new(color.red(), color.green(), color.blue(), 255);
    let clear = Rgba8Pixel::new(0, 0, 0, 0);
    paint(columns, rows, |x, y| if Matrix::Bayer4.is_inked(x as i32, y as i32, level as f32 / LEVELS as f32) { ink } else { clear })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ink(fill: InkFill) -> Ink {
        Ink { fill, direction: 180.0, scale: 3.0, from: 1.0, to: 0.4, ghost: None, matrix: Matrix::Bayer4 }
    }

    fn alphas(image: &Image) -> Vec<u8> {
        image.to_rgba8().unwrap().as_slice().iter().map(|pixel| pixel.a).collect()
    }

    fn hidden(image: &Image) -> usize {
        alphas(image).iter().filter(|&&alpha| alpha > 0).count()
    }

    #[test]
    fn the_cell_follows_the_font_size_but_never_gets_finer_than_two() {
        assert_eq!(cell_for(16.0), 2.0);
        assert_eq!(cell_for(72.0), 3.0);
        assert_eq!(cell_for(120.0), 5.0);
    }

    #[test]
    fn a_solid_fill_leaves_the_text_alone() {
        assert_eq!(hidden(&ink_overlay(40, 20, &ink(InkFill::Solid), 0.0, Color::from_rgb_u8(0, 0, 0))), 0);
    }

    #[test]
    fn a_gradient_covers_more_of_the_sparse_end() {
        let image = ink_overlay(48, 48, &ink(InkFill::Gradient), 0.0, Color::from_rgb_u8(0, 0, 0));
        let cells = alphas(&image);
        let rows = |from: usize, to: usize| cells[from * 48..to * 48].iter().filter(|&&a| a > 0).count();
        assert!(rows(0, 12) < rows(36, 48), "dense at the top, thinning towards the bottom when it points down");
    }

    #[test]
    fn the_direction_turns_the_gradient() {
        let up = Ink { direction: 0.0, ..ink(InkFill::Gradient) };
        let cells = alphas(&ink_overlay(48, 48, &up, 0.0, Color::from_rgb_u8(0, 0, 0)));
        let rows = |from: usize, to: usize| cells[from * 48..to * 48].iter().filter(|&&a| a > 0).count();
        assert!(rows(0, 12) > rows(36, 48));
    }

    #[test]
    fn dotted_and_hatched_keep_a_ghost_of_the_letters() {
        assert_eq!(ghost(&ink(InkFill::Dotted)), DEFAULT_GHOST);
        assert_eq!(ghost(&ink(InkFill::Gradient)), 0.0);
        let image = ink_overlay(16, 16, &ink(InkFill::Dotted), 0.0, Color::from_rgb_u8(0, 0, 0));
        assert!(alphas(&image).iter().all(|&a| a == 0 || a == (0.8f32 * 255.0).round() as u8));
        let open = Ink { ghost: Some(0.0), ..ink(InkFill::Hatched) };
        assert!(alphas(&ink_overlay(16, 16, &open, 0.0, Color::from_rgb_u8(0, 0, 0))).contains(&255));
    }

    #[test]
    fn noise_and_textures_drift_with_time() {
        let noise = |time: f32| alphas(&ink_overlay(40, 40, &ink(InkFill::Noise), time, Color::from_rgb_u8(0, 0, 0)));
        assert_ne!(noise(0.0), noise(30.0));
        let hatched = |time: f32| alphas(&ink_overlay(16, 16, &ink(InkFill::Hatched), time, Color::from_rgb_u8(0, 0, 0)));
        assert_ne!(hatched(0.0), hatched(1.0));
    }

    #[test]
    fn text_splits_into_characters_or_words_numbered_in_reading_order() {
        let characters = units("hi you", false);
        assert_eq!(characters.iter().filter_map(|(_, index)| *index).collect::<Vec<_>>(), vec![0, 1, 2, 3, 4]);
        assert_eq!(characters[2], (" ".to_string(), None), "a space is not a unit");
        let words = units("hi  you", true);
        assert_eq!(words.iter().filter_map(|(text, index)| index.map(|index| (text.as_str(), index))).collect::<Vec<_>>(), vec![("hi", 0), ("you", 1)]);
        assert_eq!(words.len(), 4, "both spaces stay between the words");
        assert!(units("", true).is_empty());
    }

    #[test]
    fn units_dissolve_in_the_order_asked_for() {
        assert_eq!(ranks(4, DissolveOrder::Start), vec![0.0, 1.0, 2.0, 3.0]);
        assert_eq!(ranks(4, DissolveOrder::End), vec![3.0, 2.0, 1.0, 0.0]);
        assert_eq!(ranks(5, DissolveOrder::Center), vec![2.0, 1.0, 0.0, 1.0, 2.0]);
        assert_eq!(ranks(5, DissolveOrder::Edges), vec![0.0, 1.0, 2.0, 1.0, 0.0]);
        let random = ranks(8, DissolveOrder::Random);
        let mut sorted = random.clone();
        sorted.sort_by(f32::total_cmp);
        assert_eq!(sorted, (0..8).map(|place| place as f32).collect::<Vec<_>>(), "a shuffle uses every place once");
        assert_eq!(random, ranks(8, DissolveOrder::Random), "and is the same every time");
    }

    fn dissolve(showing: bool) -> Dissolve {
        Dissolve { duration: 600.0, stagger: 28.0, delay: 0.0, matrix: Matrix::Bayer4, fill: DissolveFill::Solid, level: 0.625, showing }
    }

    #[test]
    fn a_unit_steps_through_the_levels_and_lands() {
        let options = dissolve(true);
        assert_eq!(dissolve_step(0.0, 0.0, &options), 0);
        let middle = dissolve_step(150.0, 0.0, &options);
        assert!(middle > 0 && middle < DISSOLVE_STEPS);
        assert_eq!(dissolve_step(900.0, 0.0, &options), DISSOLVE_STEPS);
        assert_eq!(dissolve_step(100.0, 10.0, &options), 0, "a later unit has not started");
    }

    #[test]
    fn dissolving_out_runs_the_levels_backwards() {
        assert_eq!(dissolve_step(0.0, 0.0, &dissolve(false)), DISSOLVE_STEPS);
        assert_eq!(dissolve_step(900.0, 0.0, &dissolve(false)), 0);
    }

    #[test]
    fn the_mask_reveals_more_cells_at_higher_steps() {
        let options = dissolve(true);
        let backdrop = Color::from_rgb_u8(0, 0, 0);
        let cover = |step: usize| hidden(&dissolve_overlay(16, 16, step, &options, backdrop));
        assert_eq!(cover(0), 256, "nothing shows at the start");
        assert!(cover(8) < cover(2));
        assert_eq!(cover(DISSOLVE_STEPS), 0, "plain text at the end");
        let dotted = Dissolve { fill: DissolveFill::Dotted, ..dissolve(true) };
        assert!(hidden(&dissolve_overlay(16, 16, DISSOLVE_STEPS, &dotted, backdrop)) > 0, "a texture stays");
        let gradient = Dissolve { fill: DissolveFill::Gradient, ..dissolve(true) };
        assert!(hidden(&dissolve_overlay(16, 16, DISSOLVE_STEPS, &gradient, backdrop)) > 0, "a gradient settles below solid");
    }

    fn scramble(mode: ScrambleMode) -> Scramble {
        Scramble { mode, duration: 520.0, stagger: 34.0, delay: 0.0, rate: 20.0 }
    }

    #[test]
    fn the_glyph_pool_is_the_text_itself_with_digits_when_it_is_short() {
        assert_eq!(glyph_pool("hello world", "").len(), 7);
        assert!(glyph_pool("aa", "").len() >= 4);
        assert_eq!(glyph_pool("x", "ab"), vec!['a', 'b', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9']);
    }

    #[test]
    fn every_character_waits_then_blocks_then_glyphs_then_settles() {
        let options = scramble(ScrambleMode::Dither);
        let pool = glyph_pool("decode", "");
        let first = |elapsed: f32| slots("decode", "", false, elapsed, &options, &pool);
        assert!(first(0.0).iter().all(|slot| slot.phase == Phase::Wait || slot.phase == Phase::Block));
        assert_eq!(first(100.0)[0].phase, Phase::Block);
        assert!(first(1500.0).iter().all(|slot| slot.phase == Phase::Done && slot.level == LEVELS));
        assert!(settled("decode", "", false, 1500.0, &options));
        assert!(!settled("decode", "", false, 50.0, &options));
    }

    #[test]
    fn the_modes_show_blocks_glyphs_or_both() {
        let pool = glyph_pool("decode", "");
        let phases = |mode: ScrambleMode, elapsed: f32| slots("decode", "", false, elapsed, &scramble(mode), &pool).iter().map(|slot| slot.phase).collect::<Vec<_>>();
        assert!(phases(ScrambleMode::Glyphs, 200.0).contains(&Phase::Glyph));
        assert!(!phases(ScrambleMode::Glyphs, 200.0).contains(&Phase::Block));
        assert!(phases(ScrambleMode::Blocks, 200.0).contains(&Phase::Block));
        assert!(!phases(ScrambleMode::Blocks, 200.0).contains(&Phase::Glyph));
        assert!(phases(ScrambleMode::Dither, 450.0).contains(&Phase::Glyph), "late in a character's run the block gives way to glyphs");
    }

    #[test]
    fn a_changed_text_only_scrambles_the_characters_that_differ() {
        let options = scramble(ScrambleMode::Dither);
        let pool = glyph_pool("cat", "");
        let result = slots("cut", "cat", true, 100.0, &options, &pool);
        assert_eq!(result[0].phase, Phase::Done);
        assert_ne!(result[1].phase, Phase::Done);
        assert_eq!(result[2].phase, Phase::Done);
        let old = slots("cut", "cat", true, 0.0, &Scramble { delay: 10_000.0, ..scramble(ScrambleMode::Dither) }, &pool);
        assert_eq!((old[1].phase, old[1].glyph.as_str()), (Phase::Old, "a"), "the old letter shows until it is its turn");
    }

    #[test]
    fn glyphs_change_over_time_but_not_within_a_tick() {
        let options = scramble(ScrambleMode::Glyphs);
        let pool = glyph_pool("abcdefghij", "");
        let first = |elapsed: f32| slots("abcdefghij", "", false, elapsed, &options, &pool)[0].glyph.clone();
        assert_eq!(first(100.0), first(110.0));
        assert!((0..20).any(|tick| first(100.0 + tick as f32 * 50.0) != first(100.0)), "the glyph keeps changing");
    }

    #[test]
    fn spaces_are_left_alone() {
        let pool = glyph_pool("a b", "");
        let result = slots("a b", "", false, 10.0, &scramble(ScrambleMode::Dither), &pool);
        assert_eq!(result[1].phase, Phase::Done);
        assert_eq!(result[1].character, " ");
    }

    #[test]
    fn a_block_thickens_with_its_level() {
        let color = Color::from_rgb_u8(255, 0, 0);
        let inked = |level: i32| alphas(&block_overlay(8, 8, level, color)).iter().filter(|&&alpha| alpha > 0).count();
        assert!(inked(3) < inked(8) && inked(8) < inked(13));
    }
}
