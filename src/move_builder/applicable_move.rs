use owlchess::{Coord, File, Move, MoveKind, Rank};
use tracing::debug;

/// A [`Move`] can be built step by step or immediately injected.
#[derive(Debug)]
pub enum ApplicableMove {
    /// [`Move`] built through a stp-by-step process of selecting source/destination/promotion.
    Manual(Move),
    /// [`Move`] injected immediately by SAN.
    Automatic(Move),
    /// Fictional [`Move`] to manage step-backs.
    /// It shall never be applied to a [`Board`].
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
            Self::Revert(m) => {
                let animations = match m.kind() {
                    // MoveKind::CastlingKingside => {}
                    // MoveKind::CastlingQueenside => {}
                    // MoveKind::PawnDouble => {}
                    // MoveKind::Enpassant => {}
                    // MoveKind::PromoteKnight => {}
                    // MoveKind::PromoteBishop => {}
                    // MoveKind::PromoteRook => {}
                    // MoveKind::PromoteQueen => {}
                    _ => vec![(m.src(), m.dst())],
                };

                debug!("Reverting {m:?}, animation: {animations:?}");
                animations
            }
            Self::Manual(m) => {
                fn coord(f: File, r: Rank) -> Coord {
                    Coord::from_parts(f, r)
                }

                // Compute supporting animations.
                match m.kind() {
                    MoveKind::CastlingKingside => {
                        // King steps from the E to G file.
                        // Rook steps from the H to F file.
                        let rank = m.src().rank();
                        vec![
                            (coord(File::E, rank), coord(File::G, rank)),
                            (coord(File::H, rank), coord(File::F, rank)),
                        ]
                    }
                    MoveKind::CastlingQueenside => {
                        // King steps from the E to C file.
                        // Rook steps from the A to D file.
                        let rank = m.src().rank();
                        vec![
                            (coord(File::E, rank), coord(File::C, rank)),
                            (coord(File::A, rank), coord(File::D, rank)),
                        ]
                    }
                    _ => vec![(m.src(), m.dst())],
                }
            }
        }
    }
}
