use glam::{Vec2, Vec3};

fn fract(x: f32) -> f32 {
    x - x.floor()
}

fn modulo(x: f32, y: f32) -> f32 {
    x - y * (x / y).floor()
}

pub fn hash21(p: Vec2) -> f32 {
    let mut p = (p * Vec2::new(123.34, 456.21)).map(fract);
    p += p.dot(p + 45.32);
    fract(p.x * p.y)
}

fn permute(x: Vec3) -> Vec3 {
    ((x * 34.0 + 1.0) * x).map(|v| modulo(v, 289.0))
}

pub fn snoise(v: Vec2) -> f32 {
    const C: [f32; 4] = [0.211_324_87, 0.366_025_42, -0.577_350_26, 0.024_390_243];
    let mut i = (v + v.dot(Vec2::splat(C[1]))).floor();
    let x0 = v - i + i.dot(Vec2::splat(C[0]));
    let i1 = if x0.x > x0.y { Vec2::new(1.0, 0.0) } else { Vec2::new(0.0, 1.0) };
    let x12_xy = x0 + C[0] - i1;
    let x12_zw = x0 + C[2];
    i = i.map(|v| modulo(v, 289.0));
    let p = permute(permute(Vec3::splat(i.y) + Vec3::new(0.0, i1.y, 1.0)) + i.x + Vec3::new(0.0, i1.x, 1.0));
    let mut m = (Vec3::splat(0.5) - Vec3::new(x0.dot(x0), x12_xy.dot(x12_xy), x12_zw.dot(x12_zw))).max(Vec3::ZERO);
    m = m * m;
    m = m * m;
    let x = (p * C[3]).map(fract) * 2.0 - 1.0;
    let h = x.abs() - 0.5;
    let ox = (x + 0.5).floor();
    let a0 = x - ox;
    m *= 1.792_842_9 - 0.853_734_73 * (a0 * a0 + h * h);
    let g = Vec3::new(
        a0.x * x0.x + h.x * x0.y,
        a0.y * x12_xy.x + h.y * x12_xy.y,
        a0.z * x12_zw.x + h.z * x12_zw.y,
    );
    130.0 * m.dot(g)
}

pub fn fbm(mut p: Vec2) -> f32 {
    let mut sum = 0.0;
    let mut amplitude = 0.5;
    for _ in 0..4 {
        sum += amplitude * snoise(p);
        p = rotate(p);
        amplitude *= 0.5;
    }
    (0.5 + 0.5 * sum / 0.9375).clamp(0.0, 1.0)
}

pub fn rotate(p: Vec2) -> Vec2 {
    Vec2::new(0.8 * p.x + 0.6 * p.y, -0.6 * p.x + 0.8 * p.y) * 2.02 + Vec2::new(11.3, 4.7)
}

pub fn fbm_octaves(mut p: Vec2, octaves: usize) -> f32 {
    let (mut sum, mut amplitude, mut norm) = (0.0, 0.5, 0.0);
    for _ in 0..octaves {
        sum += amplitude * snoise(p);
        norm += amplitude;
        p = rotate(p);
        amplitude *= 0.5;
    }
    (0.5 + 0.5 * sum / norm).clamp(0.0, 1.0)
}

pub fn spectrum(h: f32) -> Vec3 {
    let shifted = Vec3::splat(fract(h) * 6.0) + Vec3::new(0.0, 4.0, 2.0);
    let rgb = (shifted.map(|v| modulo(v, 6.0)) - 3.0).abs() - 1.0;
    let rgb = rgb.clamp(Vec3::ZERO, Vec3::ONE);
    rgb * rgb * (3.0 - 2.0 * rgb)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn samples() -> impl Iterator<Item = Vec2> {
        (0..40).flat_map(|y| (0..40).map(move |x| Vec2::new(x as f32 * 0.37 - 7.0, y as f32 * 0.29 - 5.0)))
    }

    #[test]
    fn hash_stays_in_the_unit_range_and_is_stable() {
        for p in samples() {
            let value = hash21(p);
            assert!((0.0..1.0).contains(&value));
            assert_eq!(value, hash21(p));
        }
    }

    #[test]
    fn simplex_noise_stays_in_range_and_varies() {
        let values: Vec<f32> = samples().map(snoise).collect();
        assert!(values.iter().all(|v| (-1.0..=1.0).contains(v)));
        assert!(values.iter().any(|&v| v > 0.4) && values.iter().any(|&v| v < -0.4));
        assert_eq!(snoise(Vec2::ZERO), 0.0);
    }

    #[test]
    fn fbm_stays_in_the_unit_range() {
        assert!(samples().map(fbm).all(|v| (0.0..=1.0).contains(&v)));
    }

    #[test]
    fn the_spectrum_runs_red_green_blue_and_wraps() {
        assert_eq!(spectrum(0.0), Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(spectrum(1.0 / 3.0), Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(spectrum(2.0 / 3.0), Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(spectrum(1.0), spectrum(0.0));
    }
}
