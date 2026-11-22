use crate::stat::format_decimals;

const STAGGER_START: f32 = 0.05;
const STAGGER_SPREAD: f32 = 0.4;
const DISCOVERY_SPAN: f32 = 0.45;
const DISCOVERY_END: f32 = 0.98;
const THOUSAND: f32 = 1000.0;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub struct Tile {
    pub rect: Rect,
    pub value: f32,
}

pub struct Layout {
    pub tiles: Vec<Tile>,
    pub used: f32,
}

pub fn format_value(value: f32) -> String {
    format_decimals(if value >= THOUSAND { value / THOUSAND } else { value }, 1)
}

fn ease_out(x: f32) -> f32 {
    1.0 - (1.0 - x) * (1.0 - x)
}

pub fn squarify(values: &[(usize, f32)], bounds: Rect) -> Vec<(usize, Rect)> {
    let total: f32 = values.iter().map(|(_, value)| value.max(0.0)).sum();
    if total <= 0.0 {
        return values.iter().map(|&(index, _)| (index, Rect { x: bounds.x, y: bounds.y, w: 0.0, h: 0.0 })).collect();
    }
    let mut remaining: Vec<(usize, f32)> = values.iter().map(|&(index, value)| (index, value.max(0.0))).collect();
    remaining.sort_by(|a, b| b.1.total_cmp(&a.1));
    let (mut x, mut y, mut w, mut h) = (bounds.x, bounds.y, bounds.w, bounds.h);
    let mut left = total;
    let mut rects = Vec::with_capacity(values.len());

    while !remaining.is_empty() {
        let horizontal = w >= h;
        let side = if horizontal { h } else { w };
        let run = if horizontal { w } else { h };
        let worst = |row: &[(usize, f32)], sum: f32| {
            let thickness = sum / left * run;
            if thickness <= 0.0 {
                return f32::INFINITY;
            }
            row.iter()
                .filter_map(|&(_, value)| {
                    let length = value / sum * side;
                    (length > 0.0).then(|| (thickness / length).max(length / thickness))
                })
                .fold(1.0, f32::max)
        };

        let mut count = 1;
        let mut sum = remaining[0].1;
        let mut current = worst(&remaining[..1], sum);
        while count < remaining.len() {
            let next_sum = sum + remaining[count].1;
            let next = worst(&remaining[..=count], next_sum);
            if next > current {
                break;
            }
            sum = next_sum;
            current = next;
            count += 1;
        }

        let thickness = sum / left * run;
        let mut offset = 0.0;
        for &(index, value) in &remaining[..count] {
            let length = if sum > 0.0 { value / sum * side } else { 0.0 };
            let rect = if horizontal {
                Rect { x, y: y + offset, w: thickness, h: length }
            } else {
                Rect { x: x + offset, y, w: length, h: thickness }
            };
            rects.push((index, rect));
            offset += length;
        }
        if horizontal {
            x += thickness;
            w = (w - thickness).max(0.0);
        } else {
            y += thickness;
            h = (h - thickness).max(0.0);
        }
        left = (left - sum).max(0.0);
        remaining.drain(..count);
    }
    rects
}

fn live_values(values: &[f32], remainder: Option<usize>, capacity: f32, progress: f32) -> Vec<f32> {
    if progress >= 1.0 {
        return values.to_vec();
    }
    let count = values.len();
    let mut live: Vec<f32> = values
        .iter()
        .enumerate()
        .map(|(index, &value)| {
            let start = STAGGER_START + index as f32 / count.saturating_sub(1).max(1) as f32 * STAGGER_SPREAD;
            let end = (start + DISCOVERY_SPAN).min(DISCOVERY_END);
            value * ease_out(((progress - start) / (end - start)).clamp(0.0, 1.0))
        })
        .collect();
    if let Some(remainder) = remainder {
        let used: f32 = live.iter().enumerate().filter(|&(index, _)| index != remainder).map(|(_, value)| value).sum();
        live[remainder] = (capacity - used).max(0.0);
    }
    live
}

pub fn layout(values: &[f32], remainder: Option<usize>, capacity: f32, progress: f32) -> Layout {
    let live = live_values(values, remainder, capacity, progress.clamp(0.0, 1.0));
    let used: f32 = live.iter().enumerate().filter(|&(index, _)| Some(index) != remainder).map(|(_, value)| value).sum();
    let whole = Rect { x: 0.0, y: 0.0, w: 100.0, h: 100.0 };

    let mut rects = vec![Rect { x: 0.0, y: 0.0, w: 0.0, h: 0.0 }; values.len()];
    let others: Vec<(usize, f32)> = live.iter().copied().enumerate().filter(|&(index, _)| Some(index) != remainder).collect();
    if let Some(remainder) = remainder {
        let share = if capacity > 0.0 { (used / capacity).clamp(0.0, 1.0) } else { 0.5 };
        let used_width = share * 100.0;
        for (index, rect) in squarify(&others, Rect { w: used_width, ..whole }) {
            rects[index] = rect;
        }
        rects[remainder] = Rect { x: used_width, y: 0.0, w: (100.0 - used_width).max(0.0), h: 100.0 };
    } else {
        for (index, rect) in squarify(&others, whole) {
            rects[index] = rect;
        }
    }
    Layout { tiles: rects.into_iter().zip(live).map(|(rect, value)| Tile { rect, value }).collect(), used }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(rect: &Rect) -> f32 {
        rect.w * rect.h
    }

    #[test]
    fn tiles_fill_the_bounds_in_proportion_to_their_values() {
        let rects = squarify(&[(0, 6.0), (1, 3.0), (2, 1.0)], Rect { x: 0.0, y: 0.0, w: 100.0, h: 100.0 });
        let total: f32 = rects.iter().map(|(_, rect)| area(rect)).sum();
        assert!((total - 10_000.0).abs() < 1.0);
        let of = |index: usize| area(&rects.iter().find(|(i, _)| *i == index).unwrap().1);
        assert!((of(0) / of(2) - 6.0).abs() < 0.01);
    }

    #[test]
    fn nothing_to_lay_out_gives_empty_tiles() {
        let rects = squarify(&[(0, 0.0), (1, 0.0)], Rect { x: 5.0, y: 5.0, w: 50.0, h: 50.0 });
        assert!(rects.iter().all(|(_, rect)| area(rect) == 0.0));
    }

    #[test]
    fn the_remainder_takes_the_right_side_in_proportion_to_capacity() {
        let finished = layout(&[50.0, 25.0, 75.0], Some(2), 200.0, 1.0);
        let remainder = &finished.tiles[2].rect;
        assert!((remainder.x - 37.5).abs() < 0.01);
        assert_eq!((remainder.y, remainder.h), (0.0, 100.0));
        assert_eq!(finished.used, 75.0);
    }

    #[test]
    fn values_grow_in_over_time_and_the_remainder_shrinks() {
        let values = [50.0, 25.0, 125.0];
        let begin = layout(&values, Some(2), 200.0, 0.0);
        let middle = layout(&values, Some(2), 200.0, 0.5);
        assert_eq!(begin.used, 0.0);
        assert_eq!(begin.tiles[2].value, 200.0);
        assert!(middle.used > 0.0 && middle.used < 75.0);
        assert!(middle.tiles[2].value < begin.tiles[2].value);
    }

    #[test]
    fn values_over_a_thousand_are_shown_in_thousands() {
        assert_eq!(format_value(142.0), "142");
        assert_eq!(format_value(18.25), "18.2");
        assert_eq!(format_value(14_800.0), "14.8");
        assert_eq!(format_value(157_100.0), "157.1");
    }
}
