use std::collections::BTreeMap;

use slint::{Color, Image, Rgba8Pixel};

use crate::render::{paint, pixel};
use crate::stat::format_decimals;

pub const SQUARE: u32 = 4;
pub const PITCH: u32 = 5;
pub const TOP: f32 = 20.0;
const MIN_CELL: f32 = 2.0;
const DIMMED: f32 = 0.18;
const AXIS_CHAR: f32 = 6.1;
const LABEL_GUTTER: f32 = 8.0;
const LABEL_GAP: f32 = 6.0;
const MIN_STEPS: usize = 2;
const MAX_STEPS: usize = 6;
const DAYS_PER_WEEK: usize = 7;
const MONTH_LABEL_SPACING: usize = 3;
const BACKING_ALPHA: f32 = 0.07;
const THURSDAY: i64 = 4;

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const LABELLED_WEEKDAYS: [usize; 3] = [1, 3, 5];

const MASKS: [(&str, [&str; 4]); 6] = [
    ("checker", ["#.#.", ".#.#", "#.#.", ".#.#"]),
    ("dots", ["#.#.", "....", "#.#.", "...."]),
    ("frame", ["####", "#..#", "#..#", "####"]),
    ("pinhole", ["####", "#.##", "##.#", "####"]),
    ("solid", ["####", "####", "####", "####"]),
    ("sparse", ["....", ".#..", "..#.", "...."]),
];

const STEP_MASKS: [&[&str]; 5] = [
    &["checker", "solid"],
    &["dots", "checker", "solid"],
    &["dots", "checker", "frame", "solid"],
    &["sparse", "dots", "checker", "frame", "solid"],
    &["sparse", "dots", "checker", "frame", "pinhole", "solid"],
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Calendar,
    Matrix,
}

pub struct Input {
    pub mode: Mode,
    pub entries: Vec<(String, f32)>,
    pub grid: Vec<Vec<f32>>,
    pub rows: Vec<String>,
    pub columns: Vec<String>,
    pub weeks: usize,
    pub week_start: usize,
    pub steps: usize,
    pub max: Option<f32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Square {
    pub column: usize,
    pub row: usize,
    pub value: f32,
    pub bucket: usize,
    pub label: String,
}

#[derive(Clone, Debug)]
pub struct Model {
    pub columns: usize,
    pub rows: usize,
    pub squares: Vec<Square>,
    pub column_labels: Vec<(usize, String)>,
    pub row_labels: Vec<(usize, String)>,
    pub peak: f32,
}

pub struct Fit {
    pub cell: f32,
    pub columns: usize,
    pub left: f32,
    pub label_every: usize,
}

pub fn steps(requested: usize) -> usize {
    requested.clamp(MIN_STEPS, MAX_STEPS)
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year.rem_euclid(400);
    let shifted_month = (month + 9) % 12;
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn civil_from_days(days: i64) -> (i64, usize, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 { shifted_month + 3 } else { shifted_month - 9 };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month as usize, day)
}

pub fn day_number(date: &str) -> Option<i64> {
    let mut parts = date.get(..10)?.split('-').map(|part| part.parse::<i64>().ok());
    let (year, month, day) = (parts.next()??, parts.next()??, parts.next()??);
    (1..=12).contains(&month).then(|| days_from_civil(year, month, day))
}

fn weekday(day: i64) -> usize {
    (day + THURSDAY).rem_euclid(DAYS_PER_WEEK as i64) as usize
}

fn long_date(day: i64) -> String {
    let (year, month, date) = civil_from_days(day);
    format!("{}, {} {date}, {year}", WEEKDAYS[weekday(day)], MONTHS[month - 1])
}

pub fn bucket_of(value: f32, max: f32, steps: usize) -> usize {
    if value <= 0.0 {
        return 0;
    }
    ((value / max * steps as f32).ceil() as usize).clamp(1, steps)
}

fn month_labels(first_day: i64, columns: usize) -> Vec<(usize, String)> {
    let mut labels = Vec::new();
    let mut previous_month = civil_from_days(first_day).1;
    let mut previous_column: Option<usize> = None;
    for column in 0..columns {
        let (_, month, date) = civil_from_days(first_day + (column * DAYS_PER_WEEK + DAYS_PER_WEEK - 1) as i64);
        let starts = if column == 0 { date <= DAYS_PER_WEEK as i64 } else { month != previous_month };
        let spaced = previous_column.is_none_or(|last| column - last >= MONTH_LABEL_SPACING);
        if starts && spaced && column + 2 <= columns {
            labels.push((column, MONTHS[month - 1].to_string()));
            previous_column = Some(column);
        }
        previous_month = month;
    }
    labels
}

fn calendar(input: &Input, steps: usize) -> Model {
    let mut totals: BTreeMap<i64, f32> = BTreeMap::new();
    for (date, value) in &input.entries {
        if let Some(day) = day_number(date) {
            *totals.entry(day).or_default() += if value.is_finite() { *value } else { 0.0 };
        }
    }
    let last_day = totals.keys().next_back().copied().unwrap_or(0);
    let last_row = (last_day + 4 - input.week_start as i64).rem_euclid(DAYS_PER_WEEK as i64);
    let columns = input.weeks.max(1);
    let first_day = last_day - last_row - (columns as i64 - 1) * DAYS_PER_WEEK as i64;
    let peak = input.max.unwrap_or_else(|| totals.values().copied().fold(1.0, f32::max));
    let squares = (first_day..=last_day)
        .map(|day| {
            let value = totals.get(&day).copied().unwrap_or(0.0).max(0.0);
            let offset = (day - first_day) as usize;
            Square { column: offset / DAYS_PER_WEEK, row: offset % DAYS_PER_WEEK, value, bucket: bucket_of(value, peak, steps), label: long_date(day) }
        })
        .collect();
    let row_labels = LABELLED_WEEKDAYS
        .iter()
        .map(|&day| {
            let row = (day + DAYS_PER_WEEK - input.week_start) % DAYS_PER_WEEK;
            (row, WEEKDAYS[weekday(first_day + row as i64)].to_string())
        })
        .collect();
    Model { columns, rows: DAYS_PER_WEEK, squares, column_labels: month_labels(first_day, columns), row_labels, peak }
}

fn matrix(input: &Input, steps: usize) -> Model {
    let values: Vec<f32> = input.grid.iter().flatten().copied().filter(|value| value.is_finite()).collect();
    let peak = input.max.unwrap_or_else(|| values.iter().copied().fold(1.0, f32::max));
    let mut squares = Vec::new();
    for column in 0..input.columns.len() {
        for row in 0..input.rows.len() {
            let Some(raw) = input.grid.get(row).and_then(|line| line.get(column)).copied().filter(|raw| raw.is_finite()) else { continue };
            let value = raw.max(0.0);
            squares.push(Square { column, row, value, bucket: bucket_of(value, peak, steps), label: format!("{} · {}", input.rows[row], input.columns[column]) });
        }
    }
    Model {
        columns: input.columns.len(),
        rows: input.rows.len(),
        squares,
        column_labels: input.columns.iter().cloned().enumerate().collect(),
        row_labels: input.rows.iter().cloned().enumerate().collect(),
        peak,
    }
}

pub fn model(input: &Input) -> Model {
    let steps = steps(input.steps);
    match input.mode {
        Mode::Calendar => calendar(input, steps),
        Mode::Matrix => matrix(input, steps),
    }
}

pub fn fit(width: f32, base: f32, model: &Model, widest_column_label: usize) -> Fit {
    let widest_row = model.row_labels.iter().map(|(_, text)| text.chars().count()).max().unwrap_or(0);
    let left = (widest_row as f32 * AXIS_CHAR).ceil() + LABEL_GUTTER;
    let room = (width - left).max(0.0);
    let columns = model.columns as f32;
    let ideal = (room / (columns * PITCH as f32 - 1.0)).floor();
    let cell = MIN_CELL.max(base.min(ideal));
    let fits = (((room / cell + 1.0) / PITCH as f32).floor() as usize).max(1);
    let label_every = if widest_column_label == 0 { 1 } else { (((widest_column_label as f32 * AXIS_CHAR + LABEL_GAP) / (cell * PITCH as f32)).ceil() as usize).max(1) };
    Fit { cell, columns: model.columns.min(fits), left, label_every }
}

pub fn shifted(model: &Model, offset: usize) -> Model {
    if offset == 0 {
        return model.clone();
    }
    Model {
        columns: model.columns - offset,
        squares: model.squares.iter().filter(|square| square.column >= offset).map(|square| Square { column: square.column - offset, ..square.clone() }).collect(),
        column_labels: model.column_labels.iter().filter(|(column, _)| *column >= offset).map(|(column, text)| (column - offset, text.clone())).collect(),
        ..model.clone()
    }
}

fn mask_inked(x: u32, y: u32, bucket: usize, steps: usize) -> bool {
    let Some(name) = STEP_MASKS[steps - MIN_STEPS].get(bucket.wrapping_sub(1)) else { return false };
    MASKS.iter().find(|(candidate, _)| candidate == name).is_some_and(|(_, rows)| rows[y as usize].as_bytes()[x as usize] == b'#')
}

pub struct Paint {
    pub steps: usize,
    pub color: Color,
    pub foreground: Color,
    pub focus: Option<usize>,
    pub reveal: f32,
}

pub fn image(model: &Model, paint_options: &Paint) -> Image {
    let Paint { steps, color, foreground, focus, reveal } = *paint_options;
    let width = (model.columns as u32 * PITCH).saturating_sub(1).max(1);
    let height = (model.rows as u32 * PITCH).saturating_sub(1).max(1);
    let span = model.columns as f32 + model.rows as f32 * 0.8;
    let mut squares: BTreeMap<(usize, usize), &Square> = BTreeMap::new();
    for square in &model.squares {
        if (square.column as f32 + square.row as f32 * 0.8) / span <= reveal {
            squares.insert((square.column, square.row), square);
        }
    }
    let backing = Rgba8Pixel::new(foreground.red(), foreground.green(), foreground.blue(), (BACKING_ALPHA * 255.0).round() as u8);
    let clear = Rgba8Pixel::new(0, 0, 0, 0);
    paint(width, height, |x, y| {
        let (column, row, inside_x, inside_y) = ((x / PITCH) as usize, (y / PITCH) as usize, x % PITCH, y % PITCH);
        if inside_x >= SQUARE || inside_y >= SQUARE {
            return clear;
        }
        let Some(square) = squares.get(&(column, row)) else { return clear };
        if square.bucket > 0 && mask_inked(inside_x, inside_y, square.bucket, steps) {
            let mut ink = pixel(color);
            if focus.is_some_and(|bucket| bucket != square.bucket) {
                ink.a = (ink.a as f32 * DIMMED).round() as u8;
            }
            return ink;
        }
        backing
    })
}

pub fn swatch(bucket: usize, steps: usize, color: Color, foreground: Color) -> Image {
    let backing = Rgba8Pixel::new(foreground.red(), foreground.green(), foreground.blue(), (BACKING_ALPHA * 255.0).round() as u8);
    paint(SQUARE, SQUARE, |x, y| if bucket > 0 && mask_inked(x, y, bucket, steps) { pixel(color) } else { backing })
}

pub fn square_at(model: &Model, column: i32, row: i32) -> Option<&Square> {
    if column < 0 || row < 0 {
        return None;
    }
    model.squares.iter().find(|square| square.column == column as usize && square.row == row as usize)
}

pub fn readout(square: &Square, unit: &str, units: &str) -> String {
    let value = if square.value > 0.0 {
        let count = format_decimals(square.value, 1);
        let name = if square.value == 1.0 { unit } else { units };
        if name.is_empty() { count } else { format!("{count} {name}") }
    } else {
        "None".to_string()
    };
    format!("{}\n{value}", square.label)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(mode: Mode) -> Input {
        Input { mode, entries: Vec::new(), grid: Vec::new(), rows: Vec::new(), columns: Vec::new(), weeks: 4, week_start: 0, steps: 4, max: None }
    }

    fn calendar_input(entries: &[(&str, f32)], weeks: usize) -> Input {
        Input { entries: entries.iter().map(|(date, value)| (date.to_string(), *value)).collect(), weeks, ..input(Mode::Calendar) }
    }

    #[test]
    fn days_count_from_the_epoch_and_come_back_as_dates() {
        assert_eq!(day_number("1970-01-01"), Some(0));
        assert_eq!(day_number("2026-11-22"), Some(20_779));
        assert_eq!(civil_from_days(20_779), (2026, 11, 22));
        assert_eq!(civil_from_days(day_number("2024-02-29").unwrap()), (2024, 2, 29));
        assert_eq!(day_number("not a date"), None);
        assert_eq!(day_number("2026-13-01"), None);
    }

    #[test]
    fn dates_are_written_with_their_weekday() {
        assert_eq!(long_date(day_number("2026-11-22").unwrap()), "Sun, Nov 22, 2026");
        assert_eq!(long_date(0), "Thu, Jan 1, 1970");
    }

    #[test]
    fn values_map_to_densities_above_empty() {
        assert_eq!(bucket_of(0.0, 10.0, 4), 0);
        assert_eq!(bucket_of(0.1, 10.0, 4), 1);
        assert_eq!(bucket_of(5.0, 10.0, 4), 2);
        assert_eq!(bucket_of(10.0, 10.0, 4), 4);
        assert_eq!(bucket_of(99.0, 10.0, 4), 4, "larger values draw solid too");
    }

    #[test]
    fn a_calendar_has_a_column_per_week_ending_on_the_latest_day() {
        let model = model(&calendar_input(&[("2026-11-01", 1.0), ("2026-11-22", 5.0)], 4));
        assert_eq!((model.columns, model.rows), (4, 7));
        let last = model.squares.last().unwrap();
        assert_eq!((last.column, last.row), (3, 0), "a Sunday starts the last column");
        assert_eq!(last.value, 5.0);
        assert!(model.squares.iter().all(|square| square.row < 7 && square.column < 4));
    }

    #[test]
    fn several_entries_for_a_day_add_up() {
        let model = model(&calendar_input(&[("2026-11-22", 2.0), ("2026-11-22", 3.0)], 2));
        assert_eq!(model.squares.last().unwrap().value, 5.0);
        assert_eq!(model.peak, 5.0);
    }

    #[test]
    fn a_week_can_start_on_monday() {
        let sunday = ("2026-11-22", 1.0);
        let monday_first = Input { week_start: 1, ..calendar_input(&[sunday], 2) };
        let last = model(&monday_first).squares.last().cloned().unwrap();
        assert_eq!(last.row, 6, "a Sunday ends the week when it starts on Monday");
        assert_eq!(model(&monday_first).row_labels[0], (0, "Mon".to_string()));
    }

    #[test]
    fn a_matrix_keeps_its_labels_and_skips_missing_values() {
        let data = Input {
            grid: vec![vec![1.0, 2.0], vec![3.0]],
            rows: vec!["Mon".into(), "Tue".into()],
            columns: vec!["9:00".into(), "10:00".into()],
            ..input(Mode::Matrix)
        };
        let model = model(&data);
        assert_eq!(model.squares.len(), 3);
        assert_eq!(model.squares[0].label, "Mon · 9:00");
        assert!(square_at(&model, 1, 1).is_none());
        assert_eq!(square_at(&model, 1, 0).map(|square| square.value), Some(2.0));
    }

    #[test]
    fn the_grid_shrinks_to_fit_and_drops_the_oldest_columns() {
        let wide = model(&calendar_input(&[("2026-11-22", 1.0)], 53));
        let roomy = fit(2000.0, 3.0, &wide, 0);
        assert_eq!((roomy.cell, roomy.columns), (3.0, 53));
        let cramped = fit(150.0, 3.0, &wide, 0);
        assert!(cramped.cell >= MIN_CELL && cramped.columns < 53);
        let trimmed = shifted(&wide, 53 - cramped.columns);
        assert_eq!(trimmed.columns, cramped.columns);
        assert!(trimmed.squares.iter().all(|square| square.column < trimmed.columns));
    }

    #[test]
    fn denser_buckets_ink_more_of_their_square() {
        let count = |bucket: usize| (0..SQUARE).flat_map(|y| (0..SQUARE).map(move |x| (x, y))).filter(|&(x, y)| mask_inked(x, y, bucket, 4)).count();
        assert!(count(1) < count(2) && count(2) < count(4));
        assert_eq!(count(4), 16);
        assert_eq!(count(0), 0);
    }

    #[test]
    fn the_image_draws_squares_with_a_cell_of_gap() {
        let model = model(&calendar_input(&[("2026-11-22", 10.0)], 2));
        let options = Paint { steps: 4, color: Color::from_rgb_u8(255, 0, 0), foreground: Color::from_rgb_u8(255, 255, 255), focus: None, reveal: 1.0 };
        let image = image(&model, &options).to_rgba8().unwrap();
        assert_eq!((image.width(), image.height()), (2 * PITCH - 1, 7 * PITCH - 1));
        let width = image.width() as usize;
        let gap = image.as_slice()[4];
        assert_eq!(gap.a, 0, "the gap between squares is empty");
        let (last_x, last_y) = (PITCH as usize, 0);
        assert_eq!(image.as_slice()[last_y * width + last_x].r, 255, "the solid square is inked");
    }

    #[test]
    fn the_reveal_ripples_in_from_the_left() {
        let model = model(&calendar_input(&[("2026-11-22", 10.0)], 6));
        let draw = |reveal: f32| {
            let options = Paint { steps: 4, color: Color::from_rgb_u8(255, 0, 0), foreground: Color::from_rgb_u8(255, 255, 255), focus: None, reveal };
            image(&model, &options).to_rgba8().unwrap().as_slice().iter().filter(|pixel| pixel.a > 0).count()
        };
        assert!(draw(0.0) <= (SQUARE * SQUARE) as usize, "only the first square has appeared");
        assert!(draw(0.5) < draw(1.0));
    }

    #[test]
    fn a_focused_bucket_dims_the_others() {
        let model = model(&calendar_input(&[("2026-11-21", 2.0), ("2026-11-22", 10.0)], 2));
        let options = Paint { steps: 4, color: Color::from_rgb_u8(255, 0, 0), foreground: Color::from_rgb_u8(255, 255, 255), focus: Some(4), reveal: 1.0 };
        let image = image(&model, &options).to_rgba8().unwrap();
        assert!(image.as_slice().iter().any(|pixel| pixel.r == 255 && pixel.a == 255));
        assert!(image.as_slice().iter().any(|pixel| pixel.r == 255 && pixel.a > 0 && pixel.a < 255));
    }

    #[test]
    fn the_readout_names_the_day_and_counts_the_unit() {
        let square = Square { column: 0, row: 0, value: 3.0, bucket: 2, label: "Sun, Nov 22, 2026".into() };
        assert_eq!(readout(&square, "deploy", "deploys"), "Sun, Nov 22, 2026\n3 deploys");
        assert_eq!(readout(&Square { value: 1.0, ..square.clone() }, "deploy", "deploys"), "Sun, Nov 22, 2026\n1 deploy");
        assert_eq!(readout(&Square { value: 0.0, ..square }, "deploy", "deploys"), "Sun, Nov 22, 2026\nNone");
    }
}
