use crate::SanMove;
use dioxus::hooks::Coroutine;
use owlchess::board::{FenParseError, PrettyStyle};
use owlchess::moves::{Style, ValidateError};
use owlchess::{Board, Color, Move};
use std::fmt::Display;
use std::ops::Deref;
use std::str::FromStr;
use thiserror::Error;
use tracing::debug;

pub struct HistoricalBoard {
    /// When a move is successfully applied, it will be reported to this channel.
    pub(crate) move_tx: Option<Coroutine<SanMove>>,
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
    pub fn from_fen(
        fen: &str,
        move_tx: Option<Coroutine<SanMove>>,
    ) -> Result<Self, HistoricalBoardError> {
        Board::from_str(fen)
            .map(|board| Self {
                move_tx,
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

        let board = step.to_board();

        let new_board = board.make_move(m)?;

        self.history.push(Step::Intermediate(board, m));

        self.history.push(Step::Last(new_board));

        self.h = self.history.len() - 1;

        self.report_move();

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
            None => (last.to_board(), None),
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

    fn last_intermediate_step(&self) -> Option<(&Board, &Move)> {
        if let [.., Step::Intermediate(board, m), Step::Last(_)] = &self.history[..] {
            Some((board, m))
        } else {
            None
        }
    }

    fn current_board(&self) -> &Board {
        if let [.., Step::Last(board)] = &self.history[..] {
            board
        } else {
            panic!("[BUG] History must have last step");
        }
    }

    fn report_move(&self) {
        if let Some(ref tx) = self.move_tx {
            let Some((board, m)) = self.last_intermediate_step() else {
                return;
            };

            // There is a valid move and a coroutine to report it.
            let san_repr = m
                .styled(board, Style::San)
                .expect("Board and move form a valid intermediate step")
                .to_string();

            let src_cell = board.get(m.src());

            let piece = src_cell
                .piece()
                .expect("Move is valid, thus src must contain a piece");

            let color = src_cell
                .color()
                .expect("Move is valid, thus src must contain a piece");

            tx.send(SanMove::new(&san_repr, piece, color));
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
    fn to_board(self) -> Board {
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
