use crate::Matrix;
use crate::style::cells_path;

pub fn dither_path(level: f32, matrix: Matrix) -> String {
    let map = matrix.map();
    cells_path(map.size, map.size, |x, y| {
        level > map.thresholds[y * map.size + x]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_level_inks_nothing() {
        assert_eq!(dither_path(0.0, Matrix::Bayer4), "");
    }

    #[test]
    fn full_level_inks_every_row() {
        assert_eq!(
            dither_path(1.0, Matrix::Bayer4),
            "M0 0h4v1h-4zM0 1h4v1h-4zM0 2h4v1h-4zM0 3h4v1h-4z"
        );
    }

    #[test]
    fn half_level_on_bayer2() {
        assert_eq!(dither_path(0.5, Matrix::Bayer2), "M0 0h1v1h-1zM1 1h1v1h-1z");
    }
}
