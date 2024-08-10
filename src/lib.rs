pub(crate) mod files;
pub(crate) mod historical_board;
pub(crate) mod move_builder;
pub(crate) mod piece;
pub(crate) mod promotion;
pub(crate) mod ranks;

mod chessboard;
pub use chessboard::{Chessboard, ChessboardProps, PlayerColor};

mod communication;
pub use communication::ChessboardClient;

mod pieces;
pub use pieces::PieceSet;

use crate::historical_board::HistoricalBoard;
use crate::move_builder::{MoveAction, MoveBuilder};
use dioxus::prelude::{Readable, Signal, Writable};
use owlchess::board::PrettyStyle;
use tracing::debug;

fn finalize(move_builder: &mut Signal<MoveBuilder>, board: &mut Signal<HistoricalBoard>) {
    // Try finalizing the move builder and apply the move.
    let finalized = {
        let board = board.read();
        move_builder.write().finalize(&board)
    };

    match finalized {
        MoveAction::Apply(m) => {
            debug!("Applying the move {m:?}");
            board.write().make_move(m).expect("Move must be valid");
            debug!("New board\n{}", board.read().pretty(PrettyStyle::Utf8));
        }
        MoveAction::Revert => {
            debug!("Reverting the last move");
            let m = board.write().revert_last_move();
            debug!(
                "Move {m:?} has been reverted\nNew board{}\n",
                board.read().pretty(PrettyStyle::Utf8)
            );
        }
        MoveAction::None => {}
    }
}
