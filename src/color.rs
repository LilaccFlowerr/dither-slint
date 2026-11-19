pub type Rgba = [f32; 4];

pub const DEFAULT_CELL: f32 = 3.0;

const NAMED: [(&str, &str); 8] = [
    ("ink", "#5471f5"),
    ("blue", "#0593e6"),
    ("purple", "#9e60e8"),
    ("green", "#32b364"),
    ("pink", "#e4569e"),
    ("orange", "#ed8725"),
    ("red", "#ea3c3c"),
    ("grey", "#808080"),
];

fn parse_hex(value: &str) -> Option<Rgba> {
    let digits = value.strip_prefix('#')?;
    let digits = match digits.len() {
        3 => digits.chars().flat_map(|c| [c, c]).collect::<String>(),
        6 | 8 => digits.to_string(),
        _ => return None,
    };
    let byte = |i: usize| u8::from_str_radix(digits.get(i * 2..i * 2 + 2)?, 16).ok();
    let alpha = if digits.len() == 8 { byte(3)? } else { 255 };
    Some([byte(0)?, byte(1)?, byte(2)?, alpha].map(|v| v as f32 / 255.0))
}

pub fn resolve_color(value: &str) -> Option<Rgba> {
    let value = value.trim();
    let hex = NAMED
        .iter()
        .find(|(name, _)| *name == value)
        .map_or(value, |(_, hex)| hex);
    parse_hex(hex)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex() {
        assert_eq!(resolve_color("#ff0000"), Some([1.0, 0.0, 0.0, 1.0]));
        assert_eq!(resolve_color("#fff"), Some([1.0, 1.0, 1.0, 1.0]));
        assert_eq!(resolve_color("nope"), None);
    }

    #[test]
    fn names() {
        assert!(resolve_color("green").is_some());
    }
}
