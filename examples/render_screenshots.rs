use std::fs;
use std::path::Path;
use slint::Color;
use dither_slint::Matrix;
use dither_slint::export::to_png;
use dither_slint::tabs::{BoxRect, TabsVariant, tabs_indicator};

fn save_png(path: &Path, image: slint::Image, scale: usize) {
    let buffer = image.to_rgba8().unwrap();
    let width = buffer.width() as usize;
    let height = buffer.height() as usize;
    let slice = buffer.as_slice();
    let mut raw_bytes = Vec::with_capacity(slice.len() * 4);
    for p in slice {
        raw_bytes.push(p.r);
        raw_bytes.push(p.g);
        raw_bytes.push(p.b);
        raw_bytes.push(p.a);
    }
    let png_bytes = to_png(&raw_bytes, width, height, scale).expect("failed to encode png");
    fs::write(path, png_bytes).expect("failed to write png");
    println!("Saved: {}", path.display());
}

fn compose_card_png(
    path: &Path,
    tabs_img: slint::Image,
    bg_width: usize,
    bg_height: usize,
    tab_x_offset: usize,
    tab_y_offset: usize,
    scale: usize,
) {
    let tabs_buf = tabs_img.to_rgba8().unwrap();
    let tw = tabs_buf.width() as usize;
    let th = tabs_buf.height() as usize;
    let t_slice = tabs_buf.as_slice();

    // Dark background #1b1a19, container card #22201d
    let mut canvas = vec![0u8; bg_width * bg_height * 4];
    for y in 0..bg_height {
        for x in 0..bg_width {
            let idx = (y * bg_width + x) * 4;
            // Background #141312
            canvas[idx] = 0x14;
            canvas[idx + 1] = 0x13;
            canvas[idx + 2] = 0x12;
            canvas[idx + 3] = 0xff;
        }
    }

    // Blend tabs indicator onto canvas
    for ty in 0..th {
        for tx in 0..tw {
            let cx = tab_x_offset + tx;
            let cy = tab_y_offset + ty;
            if cx < bg_width && cy < bg_height {
                let p = t_slice[ty * tw + tx];
                if p.a > 0 {
                    let c_idx = (cy * bg_width + cx) * 4;
                    let alpha = p.a as f32 / 255.0;
                    let inv = 1.0 - alpha;
                    canvas[c_idx] = (p.r as f32 * alpha + canvas[c_idx] as f32 * inv) as u8;
                    canvas[c_idx + 1] = (p.g as f32 * alpha + canvas[c_idx + 1] as f32 * inv) as u8;
                    canvas[c_idx + 2] = (p.b as f32 * alpha + canvas[c_idx + 2] as f32 * inv) as u8;
                }
            }
        }
    }

    let png_bytes = to_png(&canvas, bg_width, bg_height, scale).expect("encode png");
    fs::write(path, png_bytes).expect("write png");
    println!("Saved composed card: {}", path.display());
}

fn main() {
    let out_dir = Path::new("target/screenshots");
    fs::create_dir_all(out_dir).unwrap();

    let blue = Color::from_rgb_u8(5, 147, 230); // DitherPalette.blue
    let green = Color::from_rgb_u8(50, 179, 100); // DitherPalette.green

    // Tab dimensions:
    // Container: width 240px, height 40px (120 cols x 20 rows at 2px cell)
    // Overview: x: 4, y: 4, w: 72, h: 32
    // Deploys:  x: 80, y: 4, w: 68, h: 32
    // Settings: x: 152, y: 4, w: 76, h: 32
    let overview_box = BoxRect::new(4.0, 4.0, 72.0, 32.0);
    let deploys_box = BoxRect::new(80.0, 4.0, 68.0, 32.0);

    // 1. Pill at rest (Overview)
    let pill_overview = tabs_indicator(
        120, 20, 2.0,
        TabsVariant::Pill,
        Matrix::Bayer4,
        blue,
        overview_box,
        overview_box,
        1.0,
    );
    save_png(&out_dir.join("tabs_pill_overview.png"), pill_overview.clone(), 3);
    compose_card_png(&out_dir.join("card_pill_overview.png"), pill_overview, 140, 30, 10, 5, 2);

    // 2. Pill gliding mid-flight from Overview to Deploys (progress = 0.38)
    let pill_gliding = tabs_indicator(
        120, 20, 2.0,
        TabsVariant::Pill,
        Matrix::Bayer4,
        blue,
        overview_box,
        deploys_box,
        0.38,
    );
    save_png(&out_dir.join("tabs_pill_gliding.png"), pill_gliding.clone(), 3);
    compose_card_png(&out_dir.join("card_pill_gliding.png"), pill_gliding, 140, 30, 10, 5, 2);

    // 3. Pill at rest (Deploys in Green)
    let pill_deploys_green = tabs_indicator(
        120, 20, 2.0,
        TabsVariant::Pill,
        Matrix::Bayer4,
        green,
        deploys_box,
        deploys_box,
        1.0,
    );
    save_png(&out_dir.join("tabs_pill_green.png"), pill_deploys_green.clone(), 3);
    compose_card_png(&out_dir.join("card_pill_green.png"), pill_deploys_green, 140, 30, 10, 5, 2);

    // 4. Underline at rest (Overview)
    let underline_overview_box = BoxRect::new(4.0, 0.0, 72.0, 40.0);
    let underline_deploys_box = BoxRect::new(80.0, 0.0, 68.0, 40.0);

    let underline_rest = tabs_indicator(
        120, 20, 2.0,
        TabsVariant::Underline,
        Matrix::Bayer4,
        blue,
        underline_overview_box,
        underline_overview_box,
        1.0,
    );
    save_png(&out_dir.join("tabs_underline_overview.png"), underline_rest.clone(), 3);
    compose_card_png(&out_dir.join("card_underline_overview.png"), underline_rest, 140, 30, 10, 5, 2);

    // 5. Underline gliding
    let underline_glide = tabs_indicator(
        120, 20, 2.0,
        TabsVariant::Underline,
        Matrix::Bayer4,
        blue,
        underline_overview_box,
        underline_deploys_box,
        0.42,
    );
    save_png(&out_dir.join("tabs_underline_gliding.png"), underline_glide.clone(), 3);
    compose_card_png(&out_dir.join("card_underline_gliding.png"), underline_glide, 140, 30, 10, 5, 2);

    println!("All screenshots rendered successfully!");
}
