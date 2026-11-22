use std::collections::HashMap;
use std::sync::Arc;

use crate::field::Field;

mod animated_gradient;
mod aurora;
mod beams;
mod contours;
mod eclipse;
mod fade;
mod halftone;
mod horizon;
mod light_rays;
mod mesh_gradient;
mod metaballs;
mod noise_field;
mod orb;
mod plasma;
mod retro_grid;
mod ridgelines;
mod ripples;
mod silk;
mod smoke;
mod spotlight;
mod starfield;
mod tunnel;
mod waves;

pub fn all() -> HashMap<String, Arc<dyn Field>> {
    let fields: [(&str, Arc<dyn Field>); 23] = [
        ("animated-gradient", Arc::new(animated_gradient::AnimatedGradient)),
        ("aurora", Arc::new(aurora::Aurora)),
        ("beams", Arc::new(beams::Beams)),
        ("contours", Arc::new(contours::Contours)),
        ("eclipse", Arc::new(eclipse::Eclipse)),
        ("fade", Arc::new(fade::Fade)),
        ("halftone", Arc::new(halftone::Halftone)),
        ("horizon", Arc::new(horizon::Horizon)),
        ("light-rays", Arc::new(light_rays::LightRays)),
        ("mesh-gradient", Arc::new(mesh_gradient::MeshGradient)),
        ("metaballs", Arc::new(metaballs::Metaballs)),
        ("noise-field", Arc::new(noise_field::NoiseField)),
        ("orb", Arc::new(orb::Orb)),
        ("plasma", Arc::new(plasma::Plasma)),
        ("retro-grid", Arc::new(retro_grid::RetroGrid)),
        ("ridgelines", Arc::new(ridgelines::Ridgelines)),
        ("ripples", Arc::new(ripples::Ripples)),
        ("silk", Arc::new(silk::Silk)),
        ("smoke", Arc::new(smoke::Smoke)),
        ("spotlight", Arc::new(spotlight::Spotlight)),
        ("starfield", Arc::new(starfield::Starfield)),
        ("tunnel", Arc::new(tunnel::Tunnel)),
        ("waves", Arc::new(waves::Waves)),
    ];
    fields.into_iter().map(|(name, field)| (name.to_string(), field)).collect()
}

pub(crate) fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub(crate) fn param(params: &[f32], index: usize) -> f32 {
    params.get(index).copied().unwrap_or(0.0)
}

pub(crate) fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
