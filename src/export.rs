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
            for index in 0..count {
                let value = indices.get(index).copied().unwrap_or(-1);
                cells[index] = if value >= 0 && (value as usize) < swatches.len() {
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
            for index in 0..count {
                let offset = index * 4;
                let alpha = data[offset + 3];
                if alpha == 0 {
                    cells[index] = -1;
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
                cells[index] = swatch as i32;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn pixels(pattern: &[&[u8]]) -> (Vec<u8>, usize, usize) {
        let height = pattern.len();
        let width = pattern[0].len();
        let mut data = Vec::new();
        for row in pattern {
            for &value in *row {
                let channel = if value == 1 { 255 } else { 0 };
                data.extend([channel, channel, channel, 255]);
            }
        }
        (data, width, height)
    }

    #[test]
    fn svg_from_pixels_merges_runs_into_strokes() {
        let (data, width, height) = pixels(&[&[0, 1, 0, 1], &[1, 0, 0, 1], &[1, 1, 1, 0]]);
        let source = Source::Pixels { data: &data, width, height };
        assert_eq!(
            to_svg(&source, &SvgOptions::default()),
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"4\" height=\"3\" viewBox=\"0 0 4 3\" shape-rendering=\"crispEdges\">\
             <rect width=\"4\" height=\"3\" fill=\"#ffffff\"/>\
             <path fill=\"none\" stroke=\"#000000\" d=\"M0 0.5h1m1 0h1m-2 1h2m0 1h1\"/>\
             </svg>"
        );
    }

    #[test]
    fn svg_from_indices_fills_tall_runs() {
        let indices = [0, 0, 1, 0, -1, 1];
        let source = Source::Indexed { indices: &indices, width: 3, height: 2 };
        let options = SvgOptions {
            palette: vec!["#111111".into(), "#eeeeee".into()],
            ..SvgOptions::default()
        };
        assert_eq!(
            to_svg(&source, &options),
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"3\" height=\"2\" viewBox=\"0 0 3 2\" shape-rendering=\"crispEdges\">\
             <rect width=\"3\" height=\"2\" fill=\"#111111\"/>\
             <path fill=\"#eeeeee\" d=\"M2 0h1v2h-1z\"/>\
             </svg>"
        );
    }

    #[test]
    fn svg_without_background_and_with_cell_size() {
        let (data, width, height) = pixels(&[&[0, 1], &[1, 0]]);
        let source = Source::Pixels { data: &data, width, height };
        let options = SvgOptions { cell: 2.5, background: false, ..SvgOptions::default() };
        let svg = to_svg(&source, &options);
        assert!(svg.contains("width=\"5\" height=\"5\" viewBox=\"0 0 2 2\""));
        assert!(!svg.contains("<rect"));
    }

    #[test]
    fn transparent_pixels_are_skipped() {
        let data = [0, 0, 0, 0, 255, 255, 255, 128];
        let source = Source::Pixels { data: &data, width: 2, height: 1 };
        let svg = to_svg(&source, &SvgOptions::default());
        assert!(svg.contains("stroke-opacity=\"0.502\""));
        assert!(!svg.contains("#000000"));
    }

    #[test]
    fn data_url_escapes() {
        assert_eq!(
            svg_data_url("<a b=\"#c\">100%</a>"),
            "url(\"data:image/svg+xml,%3Ca b='%23c'%3E100%25%3C/a%3E\")"
        );
    }
}
