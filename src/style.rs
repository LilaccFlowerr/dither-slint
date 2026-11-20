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

pub fn texture_inked(variant: TextureVariant, x: usize, y: usize) -> bool {
    let (x, y) = (x % 4, y % 4);
    match variant {
        TextureVariant::Dotted => y % 2 == 0 && (x + y / 2) % 2 == 0,
        TextureVariant::Hatched => (x + y) % 4 == 3,
    }
}
