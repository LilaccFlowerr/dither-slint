#[path = "gallery/support.rs"]
mod support;

slint::slint! {
    #[edition = "1.19"]
    export { Gallery, DitherEngine } from "examples/gallery/app.slint";
}

fn main() -> Result<(), slint::PlatformError> {
    support::register_fields();
    let gallery = Gallery::new()?;
    dither_slint::install!(gallery);
    let sample = match std::env::args().nth(1) {
        Some(path) => slint::Image::load_from_path(path.as_ref()).expect("could not load the image"),
        None => support::sample(),
    };
    gallery.set_sample(sample);
    gallery.run()
}
