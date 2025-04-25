use owlchess::board::{FenParseError, PrettyStyle};
use owlchess::moves::ValidateError;
use owlchess::{Board, Color, Move};
use std::fmt::Display;
use std::mem;
use std::ops::Deref;
use std::str::FromStr;
use thiserror::Error;

pub struct HistoricalBoard {
    /// Current [Board].
    board: Board,
    /// A sequence of [`Board`]'s and associated [`Move`]'s which brings to the next version of [`Board`].
    /// until the value in `board` is produced.
    history: Vec<(Board, Move)>,
}

impl HistoricalBoard {
    /// Construct a new board from FEN notation.
    pub fn from_fen(fen: &str) -> Result<Self, HistoricalBoardError> {
        Board::from_str(fen)
            .map(|board| Self {
                board,
                history: vec![],
            })
            .map_err(HistoricalBoardError::Fen)
    }

    /// Tries to apply a [`Move`] to the inner [`Board`],
    /// if successful, pushed the old [`Board`] and the applied [`Move`] to history.
    pub fn make_move(&mut self, m: Move) -> Result<(), HistoricalBoardError> {
        let new_board = self.board.make_move(m)?;
        self.history
            .push((mem::replace(&mut self.board, new_board), m));

        Ok(())
    }

    pub fn last_move(&self) -> Option<Move> {
        self.history.last().map(|(_, m)| *m)
    }

    pub fn revert_last_move(&mut self) -> Option<Move> {
        if let Some((board, m)) = self.history.pop() {
            self.board = board;

            Some(m)
        } else {
            None
        }
    }

    pub fn side_to_move(&self) -> Color {
        self.board.side()
    }

    fn represent_current_board(&self) -> String {
        format!(
            "Board:\n{}\nMove by: {}",
            self.board.pretty(PrettyStyle::Utf8),
            self.side_to_move().as_long_str()
        )
    }
}

impl Display for HistoricalBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.represent_current_board())
    }
}

impl Deref for HistoricalBoard {
    type Target = Board;

    fn deref(&self) -> &Self::Target {
        &self.board
    }
}

#[derive(Error, Debug)]
pub enum HistoricalBoardError {
    #[error("Fen parsing error: {0}")]
    Fen(#[from] FenParseError),
    #[error("Move validation error: {0}")]
    Validation(#[from] ValidateError),
}
