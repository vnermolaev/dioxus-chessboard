mod chessboard;
pub(crate) mod files;
pub(crate) mod historical_board;
pub(crate) mod move_builder;
pub(crate) mod piece;
mod pieces;
pub(crate) mod promotion;
pub(crate) mod ranks;
mod square;

pub use chessboard::{Action, Chessboard, ChessboardProps, SanMove};
pub use owlchess::Color;
pub use pieces::PieceSet;

use crate::historical_board::HistoricalBoard;
use crate::move_builder::{MoveAction, MoveBuilder};
use dioxus::prelude::{Readable, Signal, Writable};
use owlchess::board::PrettyStyle;
use tracing::debug;

/// Tries finalizing the state of [`MoveBuilder`] and apply the [`owlchess::Move`].
fn finalize(move_builder: &mut Signal<MoveBuilder>, board: &mut Signal<HistoricalBoard>) {
    let finalized = move_builder.write().finalize();

    match finalized {
        MoveAction::Apply(m) => {
            board.write().make_move(m).expect("Move must be valid");
            debug!("New board\n{}", board.read());
        }
        MoveAction::Revert => {
            let m = board.write().revert_last_move();
            debug!(
                "Move {m:?} has been reverted \nNew board\n{}\n",
                board.read().pretty(PrettyStyle::Utf8)
            );
        }
        MoveAction::StepBack => {
            board.write().step_back();
        }
        MoveAction::StepForward => {
            board.write().step_forward();
        }
        MoveAction::None => {}
    }
}
