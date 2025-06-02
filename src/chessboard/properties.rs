use crate::chessboard::action::Action;
use crate::chessboard::san_move::SanMove;
use crate::{Color, PieceSet};
use dioxus::prelude::*;
use std::fmt::Debug;

/// [Chessboard] properties.
#[derive(PartialEq, Props, Clone)]
pub struct ChessboardProps {
    /// Is the board interactive?
    /// If you only need to display a position, set this to false.
    /// By default, the board will be interactive.
    is_interactive: Option<bool>,
    /// [`Color`] the player plays for, i.e., pieces at the bottom.
    player_color: Color,

    /// In a single-player mode, the player will only be able to move pieces of the `player_color`.
    /// Otherwise, the board allows all moves.
    single_player_mode: Option<bool>,

    /// The starting position in FEN notation.
    ///
    /// **IMPORTANT:** This value sets only the initial position.
    /// The chessboard component will not update if the user changes this starting position,
    /// because it initializes an internal state that remains immutable with respect to property changes.
    /// To update the position of an existing component, use [`Action::set_position`].
    starting_position: Option<String>,
    /// Pieces set.
    pieces_set: Option<PieceSet>,
    /// Injected action.
    action: Option<Action>,
    /// Transmitter channel of moves made on the board.
    san_tx: Option<Coroutine<SanMove>>,
}

impl ChessboardProps {
    pub(crate) fn complete(self) -> CompleteChessboardProps {
        CompleteChessboardProps {
            is_interactive: self.is_interactive.unwrap_or(true),
            color: self.player_color,
            // By default, allow exploration mode.
            single_player_mode: self.single_player_mode.unwrap_or_default(),
            starting_position: self
                .starting_position
                .unwrap_or_else(|| Self::default_position().to_string()),
            pieces_set: self.pieces_set.unwrap_or(PieceSet::Standard),
            action: self.action,
            san_tx: self.san_tx,
        }
    }
    pub fn default_position() -> &'static str {
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    }
}

/// Complete properties with absent optional values of [`ChessboardProps`] filled with default values.
pub struct CompleteChessboardProps {
    pub is_interactive: bool,
    pub color: Color,
    pub single_player_mode: bool,
    /// Starting position in FEN notation.
    pub starting_position: String,
    pub pieces_set: PieceSet,
    pub action: Option<Action>,
    pub san_tx: Option<Coroutine<SanMove>>,
}

impl Debug for CompleteChessboardProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompleteChessboardProps")
            .field("is_interactive", &self.is_interactive)
            .field("color", &self.color)
            .field("single_player_mode", &self.single_player_mode)
            .field("starting position", &self.starting_position)
            .field("pieces_set", &self.pieces_set)
            .field("action", &self.action)
            .finish()
    }
}
