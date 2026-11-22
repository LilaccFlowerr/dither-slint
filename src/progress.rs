#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Span {
    pub start: f32,
    pub share: f32,
}

fn limit(max: f32) -> f32 {
    max.max(1e-9)
}

pub fn spans(values: &[f32], max: f32) -> Vec<Span> {
    let max = limit(max);
    let mut room = 1.0_f32;
    values
        .iter()
        .map(|value| {
            let share = room.min((value.max(0.0) / max).clamp(0.0, 1.0));
            let span = Span { start: 1.0 - room, share };
            room -= share;
            span
        })
        .collect()
}

pub fn total(values: &[f32]) -> f32 {
    values.iter().map(|value| value.max(0.0)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parts_follow_each_other_from_the_left() {
        let spans = spans(&[20.0, 30.0, 10.0], 100.0);
        let starts: Vec<f32> = spans.iter().map(|span| span.start).collect();
        let shares: Vec<f32> = spans.iter().map(|span| span.share).collect();
        assert_eq!(shares, [0.2, 0.3, 0.1]);
        assert!((starts[1] - 0.2).abs() < 1e-6 && (starts[2] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn parts_are_clipped_to_the_track() {
        let spans = spans(&[70.0, 50.0, 10.0], 100.0);
        assert!((spans[1].share - 0.3).abs() < 1e-6, "the second part only gets what is left");
        assert_eq!(spans[2].share, 0.0, "nothing is left for the third");
        assert!((spans[1].start - 0.7).abs() < 1e-6);
    }

    #[test]
    fn negative_values_count_as_nothing() {
        let spans = spans(&[-5.0, 25.0], 100.0);
        assert_eq!(spans[0].share, 0.0);
        assert_eq!(spans[1].start, 0.0);
        assert_eq!(total(&[-5.0, 25.0]), 25.0);
    }

    #[test]
    fn a_maximum_of_zero_does_not_divide_by_zero() {
        let spans = spans(&[1.0], 0.0);
        assert_eq!(spans[0].share, 1.0);
    }

    #[test]
    fn there_is_one_span_for_every_part() {
        assert_eq!(spans(&[], 100.0).len(), 0);
        assert_eq!(spans(&[1.0, 2.0, 3.0, 4.0], 100.0).len(), 4);
    }
}
