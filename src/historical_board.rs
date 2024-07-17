use owlchess::board::FenParseError;
use owlchess::moves::ValidateError;
use owlchess::{Board, Move};
use std::mem;
use std::ops::Deref;
use std::str::FromStr;

pub struct HistoricalBoard {
    /// Current [Board].
    board: Board,
    /// Old [Board] and a [Move] which brings to the next version of [Board]
    /// until the value in `board` is produced.
    history: Vec<(Board, Move)>,
}

impl HistoricalBoard {
    pub fn from_fen(fen: &str) -> Result<Self, FenParseError> {
        Board::from_str(fen).map(|board| Self {
            board,
            history: vec![],
        })
    }

    pub fn make_move(&mut self, m: Move) -> Result<(), ValidateError> {
        let new_board = self.board.make_move(m)?;
        self.history
            .push((mem::replace(&mut self.board, new_board), m));

        Ok(())
    }
}

impl Deref for HistoricalBoard {
    type Target = Board;

    fn deref(&self) -> &Self::Target {
        &self.board
    }
}
