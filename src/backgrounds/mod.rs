use std::collections::HashMap;
use std::sync::Arc;

use crate::field::Field;

mod noise_field;
mod plasma;
mod ripples;
mod waves;

pub fn all() -> HashMap<String, Arc<dyn Field>> {
    let fields: [(&str, Arc<dyn Field>); 4] = [
        ("noise-field", Arc::new(noise_field::NoiseField)),
        ("plasma", Arc::new(plasma::Plasma)),
        ("ripples", Arc::new(ripples::Ripples)),
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
