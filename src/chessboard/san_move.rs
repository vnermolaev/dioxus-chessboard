use crate::Color;
use std::fmt::Display;

/// SAN-encoded chess move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanMove {
    pub san_repr: String,
    pub piece: owlchess::Piece,
    pub color: Color,
}

impl SanMove {
    pub fn new(san_repr: &str, piece: owlchess::Piece, color: Color) -> Self {
        Self {
            san_repr: san_repr.to_string(),
            piece,
            color,
        }
    }
}

impl Display for SanMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
