use wasm_bindgen::prelude::wasm_bindgen;

#[path = "../../examples/gallery/support.rs"]
mod support;

slint::slint! {
    #[edition = "1.19"]
    export { Shots, DitherEngine } from "../examples/gallery/docs-shots.slint";
}

fn requested_page() -> i32 {
    let search = web_sys::window().and_then(|window| window.location().search().ok()).unwrap_or_default();
    let name = search.trim_start_matches('?').split('&').find_map(|pair| pair.strip_prefix("demo="));
    support::PAGES.iter().position(|page| Some(*page) == name).unwrap_or(0) as i32
}

#[wasm_bindgen(start)]
pub fn start() {
    support::register_fields();
    let shots = Shots::new().unwrap();
    dither_slint::install!(shots);
    shots.set_which(requested_page());
    shots.set_sample(support::sample());
    shots.run().unwrap();
}
