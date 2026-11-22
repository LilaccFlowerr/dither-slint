use std::fs::File;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{Platform, WindowAdapter, WindowEvent};
use slint::{PhysicalSize, Rgb8Pixel, SharedPixelBuffer};

#[path = "gallery/support.rs"]
mod support;

slint::slint! {
    #[edition = "1.19"]
    export { Shots, DitherEngine, DitherTheme } from "examples/gallery/docs-shots.slint";
}

const SCALE: u32 = 2;

const SIZES: [(u32, u32); 20] =
    [(760, 560), (760, 420), (760, 480), (760, 420), (760, 400), (820, 900), (760, 520), (760, 420), (760, 420), (760, 420), (760, 520), (760, 420), (760, 420), (760, 420), (760, 640), (760, 480), (760, 420), (760, 560), (760, 420), (760, 560)];

struct Headless(Rc<MinimalSoftwareWindow>);

impl Platform for Headless {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        Ok(self.0.clone())
    }

    fn duration_since_start(&self) -> Duration {
        Duration::ZERO
    }
}

fn save(path: PathBuf, buffer: &SharedPixelBuffer<Rgb8Pixel>) {
    let mut bytes = Vec::with_capacity(buffer.as_slice().len() * 3);
    for pixel in buffer.as_slice() {
        bytes.extend([pixel.r, pixel.g, pixel.b]);
    }
    let mut encoder = png::Encoder::new(File::create(path).unwrap(), buffer.width(), buffer.height());
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header().unwrap().write_image_data(&bytes).unwrap();
}

fn main() {
    let out = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "docs/src/assets/shots".into()));
    std::fs::create_dir_all(&out).unwrap();
    support::register_fields();

    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Headless(window.clone()))).unwrap();
    window.dispatch_event(WindowEvent::ScaleFactorChanged { scale_factor: SCALE as f32 });

    for (which, (width, height)) in SIZES.into_iter().enumerate() {
        let name = support::PAGES[which];
        window.set_size(PhysicalSize::new(width * SCALE, height * SCALE));
        let shots = Shots::new().unwrap();
        dither_slint::install!(shots);
        slint::ComponentHandle::global::<DitherTheme>(&shots).set_animated(false);
        shots.set_which(which as i32);
        shots.set_sample(support::sample());
        slint::ComponentHandle::show(&shots).unwrap();
        slint::platform::update_timers_and_animations();

        let mut buffer = SharedPixelBuffer::<Rgb8Pixel>::new(width * SCALE, height * SCALE);
        window.window().request_redraw();
        window.draw_if_needed(|renderer| {
            renderer.render(buffer.make_mut_slice(), (width * SCALE) as usize);
        });
        save(out.join(format!("{name}.png")), &buffer);
        println!("{name}.png");
        slint::ComponentHandle::hide(&shots).unwrap();
    }
}
