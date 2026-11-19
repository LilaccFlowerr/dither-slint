pub const FAST_MS: u32 = 150;
pub const BASE_MS: u32 = 240;
pub const SLOW_MS: u32 = 600;
pub const EASE: [f32; 4] = [0.23, 1.0, 0.32, 1.0];

pub const DEFAULT_FPS: f64 = 30.0;
const MAX_DELTA: f64 = 0.1;

pub struct FrameClock {
    interval: f64,
    time: f64,
    last_tick: f64,
    last_draw: f64,
}

impl FrameClock {
    pub fn new(fps: f64, now_ms: f64) -> Self {
        Self {
            interval: 1000.0 / fps.max(1.0),
            time: 0.0,
            last_tick: now_ms,
            last_draw: f64::NEG_INFINITY,
        }
    }

    pub fn tick(&mut self, now_ms: f64) -> Option<(f64, f64)> {
        let delta = ((now_ms - self.last_tick) / 1000.0).min(MAX_DELTA);
        self.last_tick = now_ms;
        self.time += delta;
        if now_ms - self.last_draw < self.interval - 2.0 {
            return None;
        }
        let since_draw = ((now_ms - self.last_draw) / 1000.0).min(MAX_DELTA);
        self.last_draw = now_ms;
        Some((self.time, since_draw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sixty_hertz_draws_at_thirty() {
        let mut clock = FrameClock::new(DEFAULT_FPS, 0.0);
        let drawn: Vec<bool> = (1..=6)
            .map(|step| clock.tick(step as f64 * 16.0).is_some())
            .collect();
        assert_eq!(drawn, [true, false, true, false, true, false]);
    }

    #[test]
    fn long_frames_do_not_jump_the_motion() {
        let mut clock = FrameClock::new(DEFAULT_FPS, 0.0);
        clock.tick(10.0);
        let (time, delta) = clock.tick(5000.0).unwrap();
        assert!((time - 0.11).abs() < 1e-9);
        assert_eq!(delta, 0.1);
    }
}
