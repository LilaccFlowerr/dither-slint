use slint::{Image, Rgba8Pixel, SharedPixelBuffer};

slint::slint! {
    #[edition = "1.19"]
    export { Gallery, DitherEngine } from "examples/gallery/app.slint";
}

fn sample() -> Image {
    if let Some(path) = std::env::args().nth(1) {
        return Image::load_from_path(path.as_ref()).expect("could not load the image");
    }
    let (width, height) = (200u32, 130u32);
    let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);
    for (index, pixel) in buffer.make_mut_slice().iter_mut().enumerate() {
        let (x, y) = ((index as u32 % width) as f32, (index as u32 / width) as f32);
        let sky = y / height as f32;
        let sun = ((x - 130.0).hypot(y - 45.0) / 28.0).min(1.0);
        let light = (0.15 + 0.5 * (1.0 - sky)) * sun + (1.0 - sun) * 1.0;
        let hill = (y > 95.0 + 12.0 * (x / 35.0).sin()) as u8 as f32;
        let value = (light * (1.0 - hill) + 0.12 * hill) * 255.0;
        *pixel = Rgba8Pixel::new(value as u8, (value * 0.85) as u8, (value * 0.7) as u8, 255);
    }
    Image::from_rgba8(buffer)
}

fn main() -> Result<(), slint::PlatformError> {
    let gallery = Gallery::new()?;
    dither_slint::install!(gallery);
    gallery.set_sample(sample());
    gallery.run()
}
