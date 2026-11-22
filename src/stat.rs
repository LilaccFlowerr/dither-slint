#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Up,
    Down,
    Flat,
}

impl Direction {
    pub const ALL: [Self; 3] = [Self::Up, Self::Down, Self::Flat];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    Good,
    Bad,
    Neutral,
}

impl Outcome {
    pub const ALL: [Self; 3] = [Self::Good, Self::Bad, Self::Neutral];
}

#[derive(Clone, PartialEq, Debug)]
pub struct Change {
    pub text: String,
    pub direction: Direction,
    pub outcome: Outcome,
}

/// A number with thousands separators and at most three decimals.
pub fn format_number(value: f32) -> String {
    format_decimals(value, 3)
}

/// A number with thousands separators and at most `decimals` decimals.
pub fn format_decimals(value: f32, decimals: usize) -> String {
    let text = format!("{:.*}", decimals, value.abs());
    let (whole, fraction) = text.split_once('.').unwrap_or((&text, ""));
    let mut grouped = String::new();
    for (index, digit) in whole.chars().enumerate() {
        if index > 0 && (whole.len() - index) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    let fraction = fraction.trim_end_matches('0');
    let sign = if value < 0.0 && (grouped != "0" || !fraction.is_empty()) { "−" } else { "" };
    if fraction.is_empty() { format!("{sign}{grouped}") } else { format!("{sign}{grouped}.{fraction}") }
}

const UNITS: [&str; 5] = ["", "K", "M", "B", "T"];

/// A short number such as 1.2K or 3M, with at most one decimal.
pub fn format_compact(value: f32) -> String {
    let mut scaled = value.abs();
    let mut unit = 0;
    while unit + 1 < UNITS.len() && (scaled / 1000.0 >= 1.0 || (scaled * 10.0).round() / 10.0 >= 1000.0) {
        scaled /= 1000.0;
        unit += 1;
    }
    let text = format!("{:.1}", scaled);
    let text = text.trim_end_matches('0').trim_end_matches('.');
    let sign = if value < 0.0 && text != "0" { "−" } else { "" };
    format!("{sign}{text}{}", UNITS[unit])
}

/// What a change in percent looks like, and whether it is a good one when `good` is the wanted direction.
pub fn describe_change(delta: f32, good: Direction) -> Change {
    let amount = format!("{:.1}%", delta.abs());
    let direction = match delta {
        delta if delta > 0.0 => Direction::Up,
        delta if delta < 0.0 => Direction::Down,
        _ => Direction::Flat,
    };
    let (sign, outcome) = match direction {
        Direction::Up => ("+", if good == Direction::Up { Outcome::Good } else { Outcome::Bad }),
        Direction::Down => ("−", if good == Direction::Down { Outcome::Good } else { Outcome::Bad }),
        Direction::Flat => ("", Outcome::Neutral),
    };
    Change { text: format!("{sign}{amount}"), direction, outcome }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thousands_are_separated() {
        assert_eq!(format_number(0.0), "0");
        assert_eq!(format_number(999.0), "999");
        assert_eq!(format_number(1000.0), "1,000");
        assert_eq!(format_number(1234567.0), "1,234,567");
    }

    #[test]
    fn decimals_are_kept_without_trailing_zeros() {
        assert_eq!(format_number(12.5), "12.5");
        assert_eq!(format_number(1234.25), "1,234.25");
        assert_eq!(format_number(7.0004), "7");
    }

    #[test]
    fn the_number_of_decimals_can_be_chosen() {
        assert_eq!(format_decimals(1234.56, 1), "1,234.6");
        assert_eq!(format_decimals(72.0, 1), "72");
        assert_eq!(format_decimals(72.449, 0), "72");
    }

    #[test]
    fn negative_numbers_get_a_minus() {
        assert_eq!(format_number(-1500.0), "−1,500");
        assert_eq!(format_number(-0.0001), "0");
    }

    #[test]
    fn compact_numbers_use_a_unit_and_one_decimal() {
        assert_eq!(format_compact(0.0), "0");
        assert_eq!(format_compact(999.0), "999");
        assert_eq!(format_compact(1200.0), "1.2K");
        assert_eq!(format_compact(12000.0), "12K");
        assert_eq!(format_compact(3_400_000.0), "3.4M");
        assert_eq!(format_compact(999_960.0), "1M");
        assert_eq!(format_compact(-2500.0), "−2.5K");
        assert_eq!(format_compact(0.26), "0.3");
    }

    #[test]
    fn a_rise_is_good_when_up_is_wanted() {
        let change = describe_change(12.44, Direction::Up);
        assert_eq!((change.text.as_str(), change.direction, change.outcome), ("+12.4%", Direction::Up, Outcome::Good));
    }

    #[test]
    fn a_rise_is_bad_when_down_is_wanted() {
        let change = describe_change(3.0, Direction::Down);
        assert_eq!((change.direction, change.outcome), (Direction::Up, Outcome::Bad));
        let change = describe_change(-8.0, Direction::Down);
        assert_eq!((change.text.as_str(), change.outcome), ("−8.0%", Outcome::Good));
    }

    #[test]
    fn no_change_is_neutral_and_has_no_sign() {
        let change = describe_change(0.0, Direction::Up);
        assert_eq!((change.text.as_str(), change.direction, change.outcome), ("0.0%", Direction::Flat, Outcome::Neutral));
    }
}
