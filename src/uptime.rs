#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    Up,
    Degraded,
    Down,
    None,
}

impl Status {
    pub const ALL: [Self; 4] = [Self::Up, Self::Degraded, Self::Down, Self::None];

    pub fn text(self) -> &'static str {
        match self {
            Status::Up => "Operational",
            Status::Degraded => "Degraded",
            Status::Down => "Outage",
            Status::None => "No data",
        }
    }
}

/// One day. A negative `uptime` means there is no data for it.
#[derive(Clone, Debug, PartialEq)]
pub struct Day {
    pub date: String,
    pub uptime: f32,
    pub note: String,
    pub status: Option<Status>,
}

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// Up at 99.9% or more, degraded at 99% or more, else down.
pub fn status(day: &Day) -> Status {
    match day.status {
        Some(status) => status,
        None if day.uptime < 0.0 => Status::None,
        None if day.uptime >= 0.999 => Status::Up,
        None if day.uptime >= 0.99 => Status::Degraded,
        None => Status::Down,
    }
}

/// A share as a percentage with two decimals, rounded down so 99.999% never reads as 100%.
pub fn percent(share: f32) -> String {
    format!("{:.2}%", (share as f64 * 10_000.0 + 1e-6).floor() / 100.0)
}

/// The average uptime of the days that have data.
pub fn overall(days: &[Day]) -> Option<f32> {
    let recorded: Vec<f32> = days.iter().map(|day| day.uptime).filter(|uptime| *uptime >= 0.0).collect();
    (!recorded.is_empty()).then(|| recorded.iter().sum::<f32>() / recorded.len() as f32)
}

/// An ISO date such as `2026-10-13` as `Oct 13, 2026`. Anything else is shown as given.
pub fn date(iso: &str) -> String {
    let parts: Vec<&str> = iso.split('-').collect();
    let [year, month, day] = parts[..] else {
        return iso.to_string();
    };
    match (year.parse::<u32>(), month.parse::<usize>(), day.parse::<u32>()) {
        (Ok(year), Ok(month @ 1..=12), Ok(day)) => format!("{} {day}, {year}", MONTHS[month - 1]),
        _ => iso.to_string(),
    }
}

/// What a screen reader says for a day.
pub fn describe(day: &Day) -> String {
    let mut parts = vec![date(&day.date), status(day).text().to_string()];
    if day.uptime >= 0.0 {
        parts.push(format!("{} uptime", percent(day.uptime)));
    }
    if !day.note.is_empty() {
        parts.push(day.note.clone());
    }
    parts.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(uptime: f32) -> Day {
        Day { date: "2026-10-13".into(), uptime, note: String::new(), status: None }
    }

    #[test]
    fn the_status_follows_the_thresholds() {
        assert_eq!(status(&day(1.0)), Status::Up);
        assert_eq!(status(&day(0.999)), Status::Up);
        assert_eq!(status(&day(0.995)), Status::Degraded);
        assert_eq!(status(&day(0.99)), Status::Degraded);
        assert_eq!(status(&day(0.98)), Status::Down);
        assert_eq!(status(&day(-1.0)), Status::None);
    }

    #[test]
    fn a_given_status_wins() {
        assert_eq!(status(&Day { status: Some(Status::Down), ..day(1.0) }), Status::Down);
    }

    #[test]
    fn percentages_round_down() {
        assert_eq!(percent(1.0), "100.00%");
        assert_eq!(percent(0.99999), "99.99%");
        assert_eq!(percent(0.995), "99.50%");
        assert_eq!(percent(0.0), "0.00%");
    }

    #[test]
    fn the_overall_uptime_skips_days_without_data() {
        assert_eq!(overall(&[day(1.0), day(0.98), day(-1.0)]), Some(0.99));
        assert_eq!(overall(&[day(-1.0)]), None);
        assert_eq!(overall(&[]), None);
    }

    #[test]
    fn dates_read_as_month_day_year() {
        assert_eq!(date("2026-10-13"), "Oct 13, 2026");
        assert_eq!(date("2026-01-02"), "Jan 2, 2026");
        assert_eq!(date("yesterday"), "yesterday");
        assert_eq!(date("2026-13-01"), "2026-13-01");
    }

    #[test]
    fn a_description_names_the_day_status_uptime_and_note() {
        let noted = Day { note: "Database failover".into(), ..day(0.995) };
        assert_eq!(describe(&noted), "Oct 13, 2026, Degraded, 99.50% uptime, Database failover");
        assert_eq!(describe(&day(-1.0)), "Oct 13, 2026, No data");
    }
}
