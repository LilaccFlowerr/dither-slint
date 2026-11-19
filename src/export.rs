use std::collections::HashMap;

pub enum Source<'a> {
    Pixels {
        data: &'a [u8],
        width: usize,
        height: usize,
    },
    Indexed {
        indices: &'a [i32],
        width: usize,
        height: usize,
    },
}

impl Source<'_> {
    fn size(&self) -> (usize, usize) {
        match self {
            Source::Pixels { width, height, .. } | Source::Indexed { width, height, .. } => {
                (*width, *height)
            }
        }
    }
}

pub struct SvgOptions {
    pub cell: f32,
    pub palette: Vec<String>,
    pub background: bool,
}

impl Default for SvgOptions {
    fn default() -> Self {
        Self {
            cell: 1.0,
            palette: Vec::new(),
            background: true,
        }
    }
}

struct Swatch {
    fill: String,
    opacity: f32,
}

fn format_opacity(opacity: f32) -> String {
    let rounded = (opacity * 1000.0).round() / 1000.0;
    format!("{rounded}")
}

fn to_cells(source: &Source, palette: &[String]) -> (Vec<i32>, Vec<Swatch>) {
    let (width, height) = source.size();
    let count = width * height;
    let mut cells = vec![0i32; count];

    match source {
        Source::Indexed { indices, .. } => {
            let swatches: Vec<Swatch> = palette
                .iter()
                .map(|color| Swatch {
                    fill: color.clone(),
                    opacity: 1.0,
                })
                .collect();
            for (index, cell) in cells.iter_mut().enumerate() {
                let value = indices.get(index).copied().unwrap_or(-1);
                *cell = if value >= 0 && (value as usize) < swatches.len() {
                    value
                } else {
                    -1
                };
            }
            (cells, swatches)
        }
        Source::Pixels { data, .. } => {
            let mut lookup: HashMap<[u8; 4], usize> = HashMap::new();
            let mut swatches: Vec<Swatch> = Vec::new();
            for (index, cell) in cells.iter_mut().enumerate() {
                let offset = index * 4;
                let alpha = data[offset + 3];
                if alpha == 0 {
                    *cell = -1;
                    continue;
                }
                let key = [data[offset], data[offset + 1], data[offset + 2], alpha];
                let swatch = *lookup.entry(key).or_insert_with(|| {
                    swatches.push(Swatch {
                        fill: format!("#{:02x}{:02x}{:02x}", key[0], key[1], key[2]),
                        opacity: alpha as f32 / 255.0,
                    });
                    swatches.len() - 1
                });
                *cell = swatch as i32;
            }
            (cells, swatches)
        }
    }
}

fn most_used(cells: &[i32], swatches: &[Swatch]) -> i32 {
    let mut counts = vec![0u32; swatches.len()];
    for &cell in cells {
        if cell >= 0 {
            counts[cell as usize] += 1;
        }
    }
    let mut best = -1;
    let mut best_count = 0;
    for (index, &value) in counts.iter().enumerate() {
        if value > best_count && swatches[index].opacity == 1.0 {
            best = index as i32;
            best_count = value;
        }
    }
    best
}

struct Run {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    swatch: usize,
}

fn pair(a: f32, b: f32) -> String {
    if b < 0.0 {
        format!("{a}{b}")
    } else {
        format!("{a} {b}")
    }
}

#[derive(Default)]
struct SwatchPath {
    fill: String,
    stroke: String,
    pen_x: f32,
    pen_y: f32,
}

impl SwatchPath {
    fn add(&mut self, run: &Run) {
        if run.height > 1 {
            self.fill.push_str(&format!(
                "M{}h{}v{}h{}z",
                pair(run.x as f32, run.y as f32),
                run.width,
                run.height,
                -(run.width as i64)
            ));
            return;
        }
        let y = run.y as f32 + 0.5;
        let x = run.x as f32;
        if self.stroke.is_empty() {
            self.stroke
                .push_str(&format!("M{}h{}", pair(x, y), run.width));
        } else {
            self.stroke.push_str(&format!(
                "m{}h{}",
                pair(x - self.pen_x, y - self.pen_y),
                run.width
            ));
        }
        self.pen_x = x + run.width as f32;
        self.pen_y = y;
    }
}

fn close(paths: &mut [SwatchPath], runs: impl Iterator<Item = Run>) {
    let mut ordered: Vec<Run> = runs.collect();
    ordered.sort_by_key(|run| (run.y, run.x));
    for run in &ordered {
        paths[run.swatch].add(run);
    }
}

fn merge_runs(
    cells: &[i32],
    width: usize,
    height: usize,
    skip: i32,
    swatch_count: usize,
) -> Vec<SwatchPath> {
    let mut paths: Vec<SwatchPath> = (0..swatch_count).map(|_| SwatchPath::default()).collect();

    let mut open: HashMap<(usize, usize, i32), Run> = HashMap::new();
    for y in 0..height {
        let mut next: HashMap<(usize, usize, i32), Run> = HashMap::new();
        let mut x = 0;
        while x < width {
            let swatch = cells[y * width + x];
            let mut end = x + 1;
            while end < width && cells[y * width + end] == swatch {
                end += 1;
            }
            if swatch >= 0 && swatch != skip {
                let key = (x, end - x, swatch);
                if let Some(mut above) = open.remove(&key) {
                    above.height += 1;
                    next.insert(key, above);
                } else {
                    next.insert(
                        key,
                        Run {
                            height: 1,
                            swatch: swatch as usize,
                            width: end - x,
                            x,
                            y,
                        },
                    );
                }
            }
            x = end;
        }
        close(&mut paths, open.drain().map(|(_, run)| run));
        open = next;
    }
    close(&mut paths, open.drain().map(|(_, run)| run));
    paths
}

pub fn to_svg(source: &Source, options: &SvgOptions) -> String {
    let (width, height) = source.size();
    let (cells, swatches) = to_cells(source, &options.palette);
    let fill = if options.background {
        most_used(&cells, &swatches)
    } else {
        -1
    };
    let paths = merge_runs(&cells, width, height, fill, swatches.len());

    let mut parts = vec![format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" shape-rendering=\"crispEdges\">",
        width as f32 * options.cell,
        height as f32 * options.cell,
        width,
        height
    )];
    if fill >= 0 {
        parts.push(format!(
            "<rect width=\"{}\" height=\"{}\" fill=\"{}\"/>",
            width, height, swatches[fill as usize].fill
        ));
    }
    for (swatch, path) in swatches.iter().zip(&paths) {
        let opacity = (swatch.opacity < 1.0).then(|| format_opacity(swatch.opacity));
        if !path.fill.is_empty() {
            let fill_opacity = opacity
                .as_ref()
                .map(|value| format!(" fill-opacity=\"{value}\""))
                .unwrap_or_default();
            parts.push(format!(
                "<path fill=\"{}\"{} d=\"{}\"/>",
                swatch.fill, fill_opacity, path.fill
            ));
        }
        if !path.stroke.is_empty() {
            let stroke_opacity = opacity
                .as_ref()
                .map(|value| format!(" stroke-opacity=\"{value}\""))
                .unwrap_or_default();
            parts.push(format!(
                "<path fill=\"none\" stroke=\"{}\"{} d=\"{}\"/>",
                swatch.fill, stroke_opacity, path.stroke
            ));
        }
    }
    parts.push("</svg>".to_string());
    parts.concat()
}

pub fn svg_data_url(svg: &str) -> String {
    let escaped = svg
        .replace('%', "%25")
        .replace('"', "'")
        .replace('#', "%23")
        .replace('<', "%3C")
        .replace('>', "%3E");
    format!("url(\"data:image/svg+xml,{escaped}\")")
}

const MAX_CANVAS_SIDE: usize = 16_384;

pub fn to_png(
    data: &[u8],
    width: usize,
    height: usize,
    scale: usize,
) -> Result<Vec<u8>, png::EncodingError> {
    let largest = width.max(height).max(1);
    let factor = scale.min(MAX_CANVAS_SIDE / largest).max(1);
    let (out_width, out_height) = (width * factor, height * factor);

    let mut scaled = Vec::with_capacity(out_width * out_height * 4);
    for y in 0..out_height {
        for x in 0..out_width {
            let offset = ((y / factor) * width + x / factor) * 4;
            scaled.extend_from_slice(&data[offset..offset + 4]);
        }
    }

    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, out_width as u32, out_height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(&scaled)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode(bytes: &[u8]) -> (u32, u32, Vec<u8>) {
        let mut reader = png::Decoder::new(std::io::Cursor::new(bytes)).read_info().unwrap();
        let mut buffer = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut buffer).unwrap();
        buffer.truncate(info.buffer_size());
        (info.width, info.height, buffer)
    }

    #[test]
    fn png_keeps_pixels() {
        let data = [255, 0, 0, 255, 0, 0, 255, 255];
        let (width, height, pixels) = decode(&to_png(&data, 2, 1, 1).unwrap());
        assert_eq!((width, height), (2, 1));
        assert_eq!(pixels, data);
    }

    #[test]
    fn png_scales_with_hard_edges() {
        let data = [255, 0, 0, 255, 0, 0, 255, 255];
        let (width, height, pixels) = decode(&to_png(&data, 2, 1, 2).unwrap());
        assert_eq!((width, height), (4, 2));
        let red = [255, 0, 0, 255];
        let blue = [0, 0, 255, 255];
        let row: Vec<u8> = [red, red, blue, blue].concat();
        assert_eq!(pixels, [row.clone(), row].concat());
    }
}
