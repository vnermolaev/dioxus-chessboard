/// Classes to render the chessboard.
const _CHESSBOARD_CLASSES: &str = manganis::mg!(file("public/chessboard.css"));

/// Include Tailwind classes.
///
/// Ideally, this would use manganis::classes!;
/// unfortunately, I did not manage to include them this way.
const _TAILWIND_CLASSES: &str = manganis::mg!(file("public/tailwind.css"));

pub(crate) mod files;
pub(crate) mod historical_board;
pub(crate) mod move_builder;
pub(crate) mod piece;
pub(crate) mod promotion;
pub(crate) mod ranks;

mod chessboard;

pub use chessboard::{Action, Chessboard, ChessboardProps, PlayerColor};

mod communication;
pub use communication::ChessboardClient;

mod pieces;

pub use pieces::PieceSet;
