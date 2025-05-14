use owlchess::board::{FenParseError, PrettyStyle};
use owlchess::moves::ValidateError;
use owlchess::{Board, Color, Move};
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;
use thiserror::Error;
use tracing::debug;

pub struct HistoricalBoard {
    h: usize,
    /// A sequence of [`Board`]'s and associated [`Move`]'s which brings to the next version of [`Board`].
    /// until the value in `board` is produced.
    /// INVARIANT: Length of the history is at least 1.
    history: Vec<Step>,
}

impl HistoricalBoard {
    const INVARIANT_AT_LEAST_1_STEP: &'static str = "[BUG] History must contain at least 1 element";
    const INVARIANT_LAST_BUT_1_IS_INTERMEDIATE: &'static str =
        "[BUG] Last but one historical element must be intermediate step";
    /// Construct a new board from FEN notation.
    pub fn from_fen(fen: &str) -> Result<Self, HistoricalBoardError> {
        Board::from_str(fen)
            .map(|board| Self {
                h: 0,
                history: vec![Step::Last(board)],
            })
            .map_err(HistoricalBoardError::Fen)
    }

    /// Tries to apply a [`Move`] to the inner [`Board`],
    /// if successful, pushed the old [`Board`] and the applied [`Move`] to history.
    pub fn make_move(&mut self, m: Move) -> Result<(), HistoricalBoardError> {
        // let step = self
        //     .history
        //     .get(self.h)
        //     .expect("[BUG] HistoryIndex out of bounds");
        //
        // if matches!(step, Step::Intermediate(_, stored_move) if stored_move == &m) {
        //     self.h += 1;
        //     return Ok(());
        // }
        //
        // // Now, move `m` is either applied to the last board or an intermediate board.
        // // In the latter case the applied move is different from the expected move, so the history must be invalidated

        self.history.truncate(self.h + 1);

        let step = self.history.pop().expect(Self::INVARIANT_AT_LEAST_1_STEP);

        let board = step.dissolve();

        let new_board = board.make_move(m)?;

        self.history.push(Step::Intermediate(board, m));

        self.history.push(Step::Last(new_board));

        self.h = self.history.len() - 1;

        Ok(())
    }

    pub fn last_move(&self) -> Option<Move> {
        if let [.., Step::Intermediate(_, m), _last] = &self.history[..] {
            Some(*m)
        } else {
            None
        }
    }

    pub fn revert_last_move(&mut self) -> Option<Move> {
        let last = self.history.pop().expect(Self::INVARIANT_AT_LEAST_1_STEP);
        let last_but_1 = self.history.pop();

        let (board, m) = match last_but_1 {
            None => (last.dissolve(), None),
            Some(Step::Intermediate(board, m)) => (board, Some(m)),
            Some(Step::Last(_)) => unreachable!("{}", Self::INVARIANT_LAST_BUT_1_IS_INTERMEDIATE),
        };

        self.history.push(Step::Last(board));
        self.h = self.history.len() - 1;

        m
    }

    pub fn side_to_move(&self) -> Color {
        self.current_board().side()
    }

    pub fn set_start(&mut self) {
        self.h = 0;
    }

    pub fn set_prev(&mut self) {
        debug!("Set prev call");
        self.h = self.h.saturating_sub(1);
    }

    pub fn set_next(&mut self) {
        self.h = self.h.saturating_add(1);
    }

    pub fn set_end(&mut self) {
        self.h = self.history.len() - 1;
    }

    fn represent_current_board(&self) -> String {
        format!(
            "Board:\n{}\nMove by: {}",
            self.current_board().pretty(PrettyStyle::Utf8),
            self.side_to_move().as_long_str()
        )
    }

    fn current_board(&self) -> &Board {
        if let [.., Step::Last(board)] = &self.history[..] {
            board
        } else {
            panic!("[BUG] History must have last step");
        }
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
        self.current_board()
    }
}

enum Step {
    Last(Board),
    Intermediate(Board, Move),
}

impl Step {
    fn dissolve(self) -> Board {
        match self {
            Step::Last(board) => board,
            Step::Intermediate(board, _) => board,
        }
    }
}

#[derive(Error, Debug)]
pub enum HistoricalBoardError {
    #[error("Fen parsing error: {0}")]
    Fen(#[from] FenParseError),
    #[error("Move validation error: {0}")]
    Validation(#[from] ValidateError),
}
