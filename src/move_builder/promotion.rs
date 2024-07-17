use owlchess::Coord;

#[derive(Debug)]
/// Promotion is a two-step process:
/// * [Self::PrePromotion] corresponds to the case when after a move is built and information on a promotion piece is required,
/// * [Self::Promotion]
pub enum Promotion {
    PrePromotion {
        src: Coord,
        dst: Coord,
        // Supporting animation is just (src, dst).
    },
    Promotion {
        src: Coord,
        dst: Coord,
    },
}

impl Promotion {
    pub(crate) fn src(&self) -> Coord {
        match self {
            Self::PrePromotion { src, .. } => *src,
            Self::Promotion { src, .. } => *src,
        }
    }

    pub(crate) fn dst(&self) -> Coord {
        match self {
            Self::PrePromotion { dst, .. } => *dst,
            Self::Promotion { dst, .. } => *dst,
        }
    }

    pub(crate) fn animations(&self) -> Vec<(Coord, Coord)> {
        match self {
            Self::PrePromotion { src, dst } => vec![(*src, *dst)],
            _ => vec![],
        }
    }
}
