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
    /// Construct a new board from FEN notation.
    pub fn from_fen(fen: &str) -> Result<Self, FenParseError> {
        Board::from_str(fen).map(|board| Self {
            board,
            history: vec![],
        })
    }

    /// Tries to apply a [Move] to the inner [Board],
    /// if successful, pushed the old [Board] and the applied [Move] to history.
    pub fn make_move(&mut self, m: Move) -> Result<(), ValidateError> {
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
}

impl Deref for HistoricalBoard {
    type Target = Board;

    fn deref(&self) -> &Self::Target {
        &self.board
    }
}
