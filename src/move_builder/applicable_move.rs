use owlchess::{Coord, Move};

/// A [Move] can be built step by step or immediately injected.
#[derive(Debug)]
pub enum ApplicableMove {
    /// [Move] built through a stp-by-step process of selecting source/destination/promotion.
    Manual(Move),
    /// [Move] injected immediately by uci.
    Automatic(Move),
    /// Fictional [Move] to manage step-backs.
    /// It shall never be applied to a [Board].
    Revert(Move),
}

impl ApplicableMove {
    pub(crate) fn src(&self) -> Coord {
        match self {
            Self::Manual(m) => m.src(),
            Self::Automatic(m) => m.src(),
            Self::Revert(m) => m.src(),
        }
    }

    pub(crate) fn dst(&self) -> Coord {
        match self {
            Self::Manual(m) => m.dst(),
            Self::Automatic(m) => m.dst(),
            Self::Revert(m) => m.dst(),
        }
    }

    pub(crate) fn animations(&self) -> Vec<(Coord, Coord)> {
        match self {
            Self::Automatic(m) => vec![(m.src(), m.dst())],
            Self::Revert(m) => vec![(m.src(), m.dst())],
            _ => vec![],
        }
    }
}
