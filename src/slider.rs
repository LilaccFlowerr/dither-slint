#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliderSize {
    Sm,
    Md,
    Lg,
}

impl SliderSize {
    pub const ALL: [Self; 3] = [Self::Sm, Self::Md, Self::Lg];

    pub const fn spec(&self) -> (f32, f32, f32) {
        // (thumb_width, thumb_height, track_height)
        match self {
            Self::Sm => (14.0, 20.0, 6.0),
            Self::Md => (18.0, 24.0, 8.0),
            Self::Lg => (22.0, 30.0, 12.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliderVariant {
    Thumb,
    Bar,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SliderGeometry {
    pub cell: f32,
    pub rows: u32,
    pub track_height: f32,
    pub thumb_width: f32,
    pub thumb_height: f32,
    pub travel: f32,
}

pub fn snap(px: f32, cell: f32) -> f32 {
    let cell = cell.max(1.0);
    ((px / cell).round().max(1.0)) * cell
}

pub fn geometry_of(size: SliderSize, cell: f32, total_width: f32, variant: SliderVariant) -> SliderGeometry {
    let cell = cell.max(1.0);
    let (spec_w, spec_h, spec_track) = size.spec();

    let rows = (spec_track / cell).round().max(2.0) as u32;
    let track_height = rows as f32 * cell;

    let (thumb_width, thumb_height) = match variant {
        SliderVariant::Thumb => {
            let across = (spec_w / cell).round().max(3.0) as u32;
            let tall = (spec_h / cell).round().max(rows as f32 + 4.0) as u32;
            let final_across = if across % 2 == 0 { across + 1 } else { across };
            (final_across as f32 * cell, tall as f32 * cell)
        }
        SliderVariant::Bar => (cell, track_height),
    };

    let travel = (total_width - thumb_width).max(0.0);

    SliderGeometry {
        cell,
        rows,
        track_height,
        thumb_width,
        thumb_height,
        travel,
    }
}

pub fn tick_count(min: f32, max: f32, step: f32) -> usize {
    if step <= 0.0 || max <= min {
        return 0;
    }
    let count = ((max - min) / step + 1e-6).floor() as usize + 1;
    if count > 101 {
        0
    } else {
        count
    }
}

pub fn tick_position_px(index: usize, min: f32, max: f32, step: f32, travel: f32, thumb_w: f32) -> f32 {
    if max <= min {
        return thumb_w / 2.0;
    }
    let val = min + index as f32 * step;
    let ratio = ((val - min) / (max - min)).clamp(0.0, 1.0);
    thumb_w / 2.0 + ratio * travel
}

pub fn is_tick_active(tick_val: f32, current_val: f32, tolerance: f32) -> bool {
    tick_val <= current_val + tolerance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_snaps_to_cell_grid() {
        for size in SliderSize::ALL {
            for cell in [1.0, 2.0, 3.0, 4.0] {
                let geom_thumb = geometry_of(size, cell, 200.0, SliderVariant::Thumb);
                assert_eq!(geom_thumb.track_height % cell, 0.0);
                assert_eq!(geom_thumb.thumb_width % cell, 0.0);
                assert_eq!(geom_thumb.thumb_height % cell, 0.0);
                assert!(geom_thumb.thumb_height >= geom_thumb.track_height);

                let geom_bar = geometry_of(size, cell, 200.0, SliderVariant::Bar);
                assert_eq!(geom_bar.track_height, geom_bar.thumb_height);
                assert_eq!(geom_bar.thumb_width, cell);
            }
        }
    }

    #[test]
    fn default_geometry_matches_dither_sdk_at_2px() {
        let md = geometry_of(SliderSize::Md, 2.0, 200.0, SliderVariant::Thumb);
        assert_eq!(md.rows, 4);
        assert_eq!(md.track_height, 8.0);
        assert_eq!(md.thumb_width, 18.0);
        assert_eq!(md.thumb_height, 24.0);

        let sm = geometry_of(SliderSize::Sm, 2.0, 200.0, SliderVariant::Thumb);
        assert_eq!(sm.rows, 3);
        assert_eq!(sm.track_height, 6.0);
        assert_eq!(sm.thumb_width, 14.0);
        assert_eq!(sm.thumb_height, 20.0);

        let lg = geometry_of(SliderSize::Lg, 2.0, 200.0, SliderVariant::Thumb);
        assert_eq!(lg.rows, 6);
        assert_eq!(lg.track_height, 12.0);
        assert_eq!(lg.thumb_width, 22.0);
        assert_eq!(lg.thumb_height, 30.0);
    }

    #[test]
    fn tick_counts_and_positions() {
        assert_eq!(tick_count(0.0, 10.0, 1.0), 11);
        assert_eq!(tick_count(0.0, 100.0, 10.0), 11);
        assert_eq!(tick_count(0.0, 10.0, 0.0), 0);
        assert_eq!(tick_count(0.0, 1000.0, 1.0), 0); // > 101 ticks rejected

        let pos_start = tick_position_px(0, 0.0, 10.0, 1.0, 100.0, 18.0);
        assert_eq!(pos_start, 9.0);

        let pos_end = tick_position_px(10, 0.0, 10.0, 1.0, 100.0, 18.0);
        assert_eq!(pos_end, 109.0);

        assert!(is_tick_active(5.0, 5.0, 1e-4));
        assert!(is_tick_active(4.0, 5.0, 1e-4));
        assert!(!is_tick_active(6.0, 5.0, 1e-4));
    }
}
