/// Classes to render the chessboard.
const _CHESSBOARD_CLASSES: &str = manganis::mg!(file("public/chessboard.css"));

/// Include Tailwind classes.
///
/// Ideally, this would use manganis::classes!;
/// unfortunately, I did not manage to include them this way.
const _TAILWIND_CLASSES: &str = manganis::mg!(file("public/tailwind.css"));

pub(crate) mod files;
pub(crate) mod piece;
pub(crate) mod promotion;
pub(crate) mod ranks;

pub(crate) mod move_builder;

mod chessboard;

pub use chessboard::{Chessboard, ChessboardProps, PlayerColor};

mod pieces;
pub use pieces::PieceSet;
