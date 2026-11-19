use crate::Matrix;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FillVariant {
    Gradient,
    Dotted,
    Hatched,
    Solid,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TextureVariant {
    Dotted,
    Hatched,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FillEdge {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bloom {
    Off,
    Low,
    High,
}

pub struct BloomLevel {
    pub blur: f32,
    pub strength: f32,
}

pub fn bloom_level(bloom: Bloom) -> Option<BloomLevel> {
    match bloom {
        Bloom::Off => None,
        Bloom::Low => Some(BloomLevel { blur: 10.0, strength: 0.3 }),
        Bloom::High => Some(BloomLevel { blur: 18.0, strength: 0.55 }),
    }
}

pub const SERIES_COLORS: [&str; 8] = [
    "ink", "orange", "green", "pink", "purple", "blue", "red", "grey",
];

pub fn series_color(color: Option<&str>, index: usize) -> &str {
    color.unwrap_or(SERIES_COLORS[index % SERIES_COLORS.len()])
}

fn cells_path(width: usize, height: usize, inked: impl Fn(usize, usize) -> bool) -> String {
    let mut path = String::new();
    for y in 0..height {
        let mut start: Option<usize> = None;
        for x in 0..=width {
            let on = x < width && inked(x, y);
            match (on, start) {
                (true, None) => start = Some(x),
                (false, Some(from)) => {
                    path.push_str(&format!(
                        "M{} {}h{}v1h{}z",
                        from,
                        y,
                        x - from,
                        from as i64 - x as i64
                    ));
                    start = None;
                }
                _ => {}
            }
        }
    }
    path
}

pub struct FillTile {
    pub size: usize,
    pub path: String,
}

pub fn fill_tile(variant: TextureVariant) -> FillTile {
    match variant {
        TextureVariant::Dotted => FillTile {
            path: cells_path(4, 4, |x, y| y % 2 == 0 && (x + y / 2) % 2 == 0),
            size: 4,
        },
        TextureVariant::Hatched => FillTile {
            path: cells_path(4, 4, |x, y| (x + y) % 4 == 3),
            size: 4,
        },
    }
}

pub struct RampOptions {
    pub edge: FillEdge,
    pub length: f32,
    pub extent: Option<f32>,
    pub from: f32,
    pub to: f32,
    pub matrix: Matrix,
}

pub struct RampTile {
    pub width: usize,
    pub height: usize,
    pub path: String,
}

pub fn dither_ramp(options: &RampOptions) -> RampTile {
    let RampOptions { edge, length, from, to, matrix, .. } = *options;
    let extent = options.extent.unwrap_or(length);
    let map = matrix.map();
    let size = map.size;
    let fade = length.round().max(1.0) as usize;
    let along = ((fade as f32).max(extent) / size as f32).ceil() as usize * size;
    let vertical = matches!(edge, FillEdge::Top | FillEdge::Bottom);
    let (width, height) = if vertical { (size, along) } else { (along, size) };
    let level_at = |index: usize| {
        let step = if matches!(edge, FillEdge::Top | FillEdge::Left) {
            index
        } else {
            along - 1 - index
        };
        if step >= fade {
            to
        } else {
            from + (to - from) * ((step as f32 + 0.5) / fade as f32)
        }
    };
    let path = cells_path(width, height, |x, y| {
        level_at(if vertical { y } else { x }) > map.thresholds[(y % size) * size + (x % size)]
    });
    RampTile { width, height, path }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ramp(edge: FillEdge, length: f32, extent: Option<f32>, from: f32, to: f32) -> RampTile {
        dither_ramp(&RampOptions { edge, length, extent, from, to, matrix: Matrix::Bayer4 })
    }

    #[test]
    fn dotted_tile() {
        let tile = fill_tile(TextureVariant::Dotted);
        assert_eq!(tile.size, 4);
        assert_eq!(tile.path, "M0 0h1v1h-1zM2 0h1v1h-1zM1 2h1v1h-1zM3 2h1v1h-1z");
    }

    #[test]
    fn hatched_tile() {
        let tile = fill_tile(TextureVariant::Hatched);
        assert_eq!(tile.path, "M3 0h1v1h-1zM2 1h1v1h-1zM1 2h1v1h-1zM0 3h1v1h-1z");
    }

    #[test]
    fn full_coverage_inks_every_cell() {
        let tile = ramp(FillEdge::Top, 4.0, None, 1.0, 1.0);
        assert_eq!((tile.width, tile.height), (4, 4));
        assert_eq!(tile.path, "M0 0h4v1h-4zM0 1h4v1h-4zM0 2h4v1h-4zM0 3h4v1h-4z");
    }

    #[test]
    fn zero_coverage_inks_nothing() {
        assert_eq!(ramp(FillEdge::Left, 4.0, None, 0.0, 0.0).path, "");
    }

    #[test]
    fn tile_length_rounds_up_to_whole_matrices() {
        let vertical = ramp(FillEdge::Bottom, 4.0, Some(9.0), 1.0, 0.0);
        assert_eq!((vertical.width, vertical.height), (4, 12));
        let horizontal = ramp(FillEdge::Right, 4.0, Some(9.0), 1.0, 0.0);
        assert_eq!((horizontal.width, horizontal.height), (12, 4));
    }

    #[test]
    fn ramp_is_denser_at_its_edge() {
        let tile = ramp(FillEdge::Top, 16.0, None, 1.0, 0.0);
        let rows: Vec<usize> = tile.path.split('M').skip(1).map(|run| {
            run.split(' ').nth(1).unwrap().split('h').next().unwrap().parse().unwrap()
        }).collect();
        let top = rows.iter().filter(|&&y| y < 4).count();
        let bottom = rows.iter().filter(|&&y| y >= 12).count();
        assert!(top > bottom, "top {top}, bottom {bottom}");
    }

    #[test]
    fn series_colors_cycle() {
        assert_eq!(series_color(None, 0), "ink");
        assert_eq!(series_color(None, 8), "ink");
        assert_eq!(series_color(None, 2), "green");
        assert_eq!(series_color(Some("#fff"), 5), "#fff");
    }

    #[test]
    fn bloom_levels() {
        assert!(bloom_level(Bloom::Off).is_none());
        assert_eq!(bloom_level(Bloom::High).unwrap().blur, 18.0);
    }
}
