#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchSize {
    Sm,
    Md,
    Lg,
}

impl SwitchSize {
    pub const ALL: [Self; 3] = [Self::Sm, Self::Md, Self::Lg];

    pub const fn target(&self) -> (f32, f32, f32) {
        // (width, height, pad)
        match self {
            Self::Sm => (34.0, 20.0, 2.0),
            Self::Md => (42.0, 24.0, 2.0),
            Self::Lg => (54.0, 30.0, 4.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwitchGeometry {
    pub width: f32,
    pub height: f32,
    pub pad: f32,
    pub thumb: f32,
    pub travel: f32,
    pub radius: f32,
    pub thumb_radius: f32,
}

pub fn snap(px: f32, cell: f32) -> f32 {
    let cell = cell.max(1.0);
    ((px / cell).round().max(1.0)) * cell
}

pub fn geometry_of(size: SwitchSize, cell: f32) -> SwitchGeometry {
    let (target_w, target_h, target_pad) = size.target();
    let pad = snap(target_pad, cell);
    let height = snap(target_h, cell).max(pad * 2.0 + cell * 2.0);
    let thumb = height - pad * 2.0;
    let width = snap(target_w, cell).max(thumb + pad * 2.0 + cell * 2.0);
    let radius = (height * 0.32).round();
    let thumb_radius = (radius - pad).max(2.0);
    let travel = width - thumb - pad * 2.0;

    SwitchGeometry {
        width,
        height,
        pad,
        thumb,
        travel,
        radius,
        thumb_radius,
    }
}

pub const WAVE_LEVELS: [f32; 4] = [3.0 / 16.0, 6.0 / 16.0, 9.0 / 16.0, 12.0 / 16.0];
pub const WAVE_DURATION_MS: u32 = 180;
pub const WAVE_STAGGER_MS: u32 = 20;

pub fn wave_delay_ms(index: usize, turning_on: bool) -> u32 {
    let index = index.min(4);
    if turning_on {
        (index as u32) * WAVE_STAGGER_MS
    } else {
        ((4 - index) as u32) * WAVE_STAGGER_MS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_snaps_to_cell_multiples() {
        for size in SwitchSize::ALL {
            for cell in [1.0, 2.0, 3.0, 4.0] {
                let geom = geometry_of(size, cell);
                assert_eq!(geom.pad % cell, 0.0);
                assert_eq!(geom.height % cell, 0.0);
                assert_eq!(geom.thumb % cell, 0.0);
                assert_eq!(geom.width % cell, 0.0);
                assert_eq!(geom.travel % cell, 0.0);
                assert!(geom.travel > 0.0);
                assert_eq!(geom.width, geom.thumb + geom.pad * 2.0 + geom.travel);
            }
        }
    }

    #[test]
    fn default_geometry_matches_dither_sdk_at_2px() {
        let md = geometry_of(SwitchSize::Md, 2.0);
        assert_eq!(md.width, 42.0);
        assert_eq!(md.height, 24.0);
        assert_eq!(md.pad, 2.0);
        assert_eq!(md.thumb, 20.0);
        assert_eq!(md.travel, 18.0);
        assert_eq!(md.radius, 8.0);
        assert_eq!(md.thumb_radius, 6.0);

        let sm = geometry_of(SwitchSize::Sm, 2.0);
        assert_eq!(sm.width, 34.0);
        assert_eq!(sm.height, 20.0);
        assert_eq!(sm.pad, 2.0);
        assert_eq!(sm.thumb, 16.0);
        assert_eq!(sm.travel, 14.0);

        let lg = geometry_of(SwitchSize::Lg, 2.0);
        assert_eq!(lg.width, 54.0);
        assert_eq!(lg.height, 30.0);
        assert_eq!(lg.pad, 4.0);
        assert_eq!(lg.thumb, 22.0);
        assert_eq!(lg.travel, 24.0);
    }

    #[test]
    fn wave_delays_stagger_in_correct_direction() {
        assert_eq!(wave_delay_ms(0, true), 0);
        assert_eq!(wave_delay_ms(1, true), 20);
        assert_eq!(wave_delay_ms(2, true), 40);
        assert_eq!(wave_delay_ms(3, true), 60);
        assert_eq!(wave_delay_ms(4, true), 80);

        assert_eq!(wave_delay_ms(4, false), 0);
        assert_eq!(wave_delay_ms(3, false), 20);
        assert_eq!(wave_delay_ms(2, false), 40);
        assert_eq!(wave_delay_ms(1, false), 60);
        assert_eq!(wave_delay_ms(0, false), 80);
    }
}
