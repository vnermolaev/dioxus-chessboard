use crate::SanMove;
use std::fmt::Display;

/// Description of navigation between the game steps on the [`crate::history::HistoricalBoard`]
/// and a new [`owlchess::Move`] application.
#[derive(Debug)]
pub enum BoardAction {
    /// Wraps an [`owlchess::Move`] that has been applied to the old board to get the new board.
    Apply(SanMove),
    /// In a historical sequence:
    /// Intermediate(board_1, move_1) -> Intermediate(board_2, move_2) -> Last(board_3),
    /// stepping back from intermediate step 2 yields the SAN move (move_**1**) which, when applied to board_1, produces board_2.
    StepBack(SanMove),
    /// In a historical sequence:
    /// Intermediate(board_1, move_1) -> Intermediate(board_2, move_2) -> Last(board_3),
    /// stepping forward from intermediate step 2 yields the SAN move (move_**2**) which, when applied to board_2, produces board_3.
    StepForward(SanMove),
    SetStartPosition,
    SetEndPosition,
}

impl Display for BoardAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Apply(m) => write!(f, "Apply {m}"),
            Self::StepBack(m) => write!(f, "Step back {m}"),
            Self::StepForward(m) => write!(f, "Step forward {m}"),
            Self::SetStartPosition => write!(f, "Setting start position"),
            Self::SetEndPosition => write!(f, "Setting end position"),
        }
    }
}
