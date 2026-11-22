pub type Rgba = [f32; 4];

fn to_linear(c: f64) -> f64 {
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

fn to_srgb(c: f64) -> f64 {
    if c <= 0.0031308 { c * 12.92 } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 }
}

fn to_oklab([r, g, b, _]: Rgba) -> [f64; 3] {
    let (r, g, b) = (to_linear(r as f64), to_linear(g as f64), to_linear(b as f64));
    let l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b).cbrt();
    let m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b).cbrt();
    let s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b).cbrt();
    [
        0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    ]
}

fn from_oklab([lightness, a, b]: [f64; 3], alpha: f32) -> Rgba {
    let l = (lightness + 0.3963377774 * a + 0.2158037573 * b).powi(3);
    let m = (lightness - 0.1055613458 * a - 0.0638541728 * b).powi(3);
    let s = (lightness - 0.0894841775 * a - 1.2914855480 * b).powi(3);
    let channel = |c: f64| to_srgb(c).clamp(0.0, 1.0) as f32;
    [
        channel(4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s),
        channel(-1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s),
        channel(-0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s),
        alpha,
    ]
}

pub fn mix_oklab(base: Rgba, other: Rgba, amount: f32) -> Rgba {
    let amount = amount.clamp(0.0, 1.0);
    let alpha = base[3] + (other[3] - base[3]) * amount;
    if alpha <= 0.0 {
        return [0.0; 4];
    }
    let (from, to) = (to_oklab(base), to_oklab(other));
    let (amount, base_alpha, other_alpha) = (amount as f64, base[3] as f64, other[3] as f64);
    let weight = |i: usize| {
        let premultiplied = from[i] * base_alpha + (to[i] * other_alpha - from[i] * base_alpha) * amount;
        premultiplied / alpha as f64
    };
    from_oklab([weight(0), weight(1), weight(2)], alpha)
}

/// A CSS `hsl()` colour: hue in degrees, saturation and lightness from 0 to 1.
pub fn hsl(hue: f32, saturation: f32, lightness: f32) -> Rgba {
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let channel = |n: f32| {
        let k = (n + hue / 30.0).rem_euclid(12.0);
        lightness - chroma / 2.0 * (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0)
    };
    [channel(0.0), channel(8.0), channel(4.0), 1.0]
}

/// Two shades of one random hue: a strong one and a lighter one.
pub fn random_pair() -> [Rgba; 2] {
    use std::hash::{BuildHasher, Hasher};
    let bits = std::collections::hash_map::RandomState::new().build_hasher().finish();
    let hue = (bits % 361) as f32;
    let saturation = 0.6 + ((bits >> 16) % 41) as f32 / 100.0;
    [hsl(hue, saturation, 0.5), hsl(hue, saturation, 0.65)]
}

#[cfg(test)]
mod tests {
    #[test]
    fn hsl_matches_css() {
        assert_eq!(hsl(0.0, 1.0, 0.5), [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(hsl(120.0, 1.0, 0.5), [0.0, 1.0, 0.0, 1.0]);
        let grey = hsl(200.0, 0.0, 0.5);
        assert!(grey[..3].iter().all(|c| (c - 0.5).abs() < 1e-6));
    }

    #[test]
    fn a_random_pair_is_one_hue_in_two_shades() {
        let [strong, light] = random_pair();
        let sum = |c: Rgba| c[0] + c[1] + c[2];
        assert!(sum(light) > sum(strong));
    }

    use super::*;

    const INK: Rgba = [0.329, 0.443, 0.961, 1.0];

    fn close(a: Rgba, b: Rgba) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < 0.002)
    }

    #[test]
    fn mix_ends() {
        let white = [1.0, 1.0, 1.0, 1.0];
        assert!(close(mix_oklab(INK, white, 0.0), INK));
        assert!(close(mix_oklab(INK, white, 1.0), white));
    }

    #[test]
    fn mix_towards_black_darkens() {
        let dark = mix_oklab(INK, [0.0, 0.0, 0.0, 1.0], 0.48);
        assert!(dark[..3].iter().zip(&INK[..3]).all(|(d, i)| d < i));
    }

    #[test]
    fn mix_with_transparent_keeps_hue() {
        let tint = mix_oklab(INK, [0.0; 4], 0.91);
        assert!(close(tint, [INK[0], INK[1], INK[2], 0.09]));
    }
}
