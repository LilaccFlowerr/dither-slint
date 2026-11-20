#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FillVariant {
    Gradient,
    Dotted,
    Hatched,
    Solid,
}

impl FillVariant {
    pub const ALL: [Self; 4] = [Self::Gradient, Self::Dotted, Self::Hatched, Self::Solid];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FillEdge {
    Top,
    Bottom,
    Left,
    Right,
}

impl FillEdge {
    pub const ALL: [Self; 4] = [Self::Top, Self::Bottom, Self::Left, Self::Right];
}

pub fn texture_inked(variant: FillVariant, x: usize, y: usize) -> bool {
    let (x, y) = (x % 4, y % 4);
    match variant {
        FillVariant::Dotted => y % 2 == 0 && (x + y / 2) % 2 == 0,
        FillVariant::Hatched => (x + y) % 4 == 3,
        FillVariant::Gradient | FillVariant::Solid => true,
    }
}
