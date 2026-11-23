use dither_slint::field::{self, Cell};
use dither_slint::glam::Vec2;
use dither_slint::noise::fbm;

pub fn register() {
    field::register("moonrise-1", |cell: &Cell| {
        let horizon = 0.4;
        (1.0 - (cell.uv.y - horizon).abs() * 2.5).max(0.0) * 0.5
    });

    field::register("moonrise-2", |cell: &Cell| {
        let aspect = cell.resolution.x / cell.resolution.y;
        let (moon_x, horizon) = (0.7, 0.4);
        let moon = Vec2::new(moon_x * aspect, horizon + 0.3);
        let at = Vec2::new(cell.uv.x * aspect, cell.uv.y);

        let glow = (1.0 - (cell.uv.y - horizon).abs() * 2.5).max(0.0) * 0.5;
        let distance = at.distance(moon);
        let disc = ((0.07 - distance) * 60.0).clamp(0.0, 1.0);
        let halo = (1.0 - distance * 3.0).max(0.0) * 0.3;
        glow.max(halo).max(disc)
    });

    field::register("moonrise-3", |cell: &Cell| {
        let aspect = cell.resolution.x / cell.resolution.y;
        let (moon_x, horizon) = (0.7, 0.4);
        let moon = Vec2::new(moon_x * aspect, horizon + 0.3);
        let at = Vec2::new(cell.uv.x * aspect, cell.uv.y);

        let glow = (1.0 - (cell.uv.y - horizon).abs() * 2.5).max(0.0) * 0.5;
        let distance = at.distance(moon);
        let disc = ((0.07 - distance) * 60.0).clamp(0.0, 1.0);
        let halo = (1.0 - distance * 3.0).max(0.0) * 0.3;
        let sky = glow.max(halo).max(disc);

        let drift = Vec2::new(cell.time * 0.03, 0.0);
        let clouds = (fbm(at * 2.5 + drift) - 0.5).max(0.0) * (1.0 - cell.uv.y);
        (sky + clouds * 0.6).min(1.0)
    });

    field::register("moonrise-4", |cell: &Cell| {
        let aspect = cell.resolution.x / cell.resolution.y;
        let (moon_x, horizon) = (0.7, 0.4);
        let moon = Vec2::new(moon_x * aspect, horizon + 0.3);
        let at = Vec2::new(cell.uv.x * aspect, cell.uv.y);

        let glow = (1.0 - (cell.uv.y - horizon).abs() * 2.5).max(0.0) * 0.5;
        let distance = at.distance(moon);
        let disc = ((0.07 - distance) * 60.0).clamp(0.0, 1.0);
        let halo = (1.0 - distance * 3.0).max(0.0) * 0.3;
        let sky = glow.max(halo).max(disc);

        let drift = Vec2::new(cell.time * 0.03, 0.0);
        let clouds = (fbm(at * 2.5 + drift) - 0.5).max(0.0) * (1.0 - cell.uv.y);
        if cell.uv.y >= horizon {
            return (sky + clouds * 0.6).min(1.0);
        }

        let depth = (horizon - cell.uv.y) / horizon;
        let column = (1.0 - (at.x - moon.x).abs() * 6.0).max(0.0);
        let ripple = 0.5 + 0.5 * (cell.uv.y * 120.0 + fbm(at * 6.0) * 6.0 - cell.time * 1.5).sin();
        ripple * (0.18 + column * 0.9) * (1.0 - depth)
    });

    field::register("moonrise-5", |cell: &Cell| {
        let aspect = cell.resolution.x / cell.resolution.y;
        let (moon_x, horizon) = (cell.params[0], cell.params[1]);
        let moon = Vec2::new(moon_x * aspect, horizon + 0.3);
        let at = Vec2::new(cell.uv.x * aspect, cell.uv.y);

        let glow = (1.0 - (cell.uv.y - horizon).abs() * 2.5).max(0.0) * 0.5;
        let distance = at.distance(moon);
        let disc = ((0.07 - distance) * 60.0).clamp(0.0, 1.0);
        let halo = (1.0 - distance * 3.0).max(0.0) * 0.3;
        let sky = glow.max(halo).max(disc);

        let drift = Vec2::new(cell.time * 0.03, 0.0);
        let clouds = (fbm(at * 2.5 + drift) - 0.5).max(0.0) * (1.0 - cell.uv.y);
        if cell.uv.y >= horizon {
            return (sky + clouds * 0.6).min(1.0);
        }

        let depth = (horizon - cell.uv.y) / horizon;
        let column = (1.0 - (at.x - moon.x).abs() * 6.0).max(0.0);
        let ripple = 0.5 + 0.5 * (cell.uv.y * 120.0 + fbm(at * 6.0) * 6.0 - cell.time * 1.5).sin();
        ripple * (0.18 + column * 0.9) * (1.0 - depth)
    });
}
