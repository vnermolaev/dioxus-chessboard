use crate::history::BoardAction;
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
    pub(crate) move_tx: Option<Coroutine<BoardAction>>,
    step_pointer: usize,
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
    pub fn initialize(
        fen: &str,
        move_tx: Option<Coroutine<BoardAction>>,
    ) -> Result<Self, HistoricalBoardError> {
        Board::from_str(fen)
            .map(|board| Self {
                move_tx,
                step_pointer: 0,
                history: vec![Step::Last(board)],
            })
            .map_err(HistoricalBoardError::Fen)
    }

    /// Tries to apply a [`Move`] to the [`Board`], which is currently pointed to by the step pointer.
    /// [`Step`]'s after the step pointer are discarded and the injected moved with the new [`Board`] become the last [`Step`].
    pub fn make_move(&mut self, m: Move) -> Result<(), HistoricalBoardError> {
        debug!("Making a move {m:?}");

        // 1 is added because the argument represents the length if the vector after truncation.
        self.history.truncate(self.step_pointer + 1);

        let step = self.history.pop().expect(Self::INVARIANT_AT_LEAST_1_STEP);

        let board = step.into_board();

        let new_board = board.make_move(m)?;

        self.history
            .push(Step::Intermediate(IntermediateStep { board, m }));

        self.history.push(Step::Last(new_board));

        self.step_pointer = self.history.len() - 1;

        self.report_move();

        Ok(())
    }

    /// Insights to the history.
    ///
    /// Returns the [`Move`] associated with the [`Step`] that immediately precedes
    /// the [`Step`] currently pointed to by the step pointer.
    pub fn get_previous_move(&self) -> Option<Move> {
        debug!(
            "Get previous move: pointer = {}/{}",
            self.step_pointer,
            self.history.len() - 1
        );

        if self.step_pointer == 0 {
            return None;
        }

        self.history
            .get(self.step_pointer.saturating_sub(1))
            .and_then(|s| s.as_move())
            .inspect(|m| debug!("Previous move: {m:?}"))
    }

    /// Insights to the history.
    ///
    /// Returns the [`Move`] associated with the [`Step`] that immediately follows
    /// the [`Step`] currently pointed to by the step pointer.
    pub fn get_next_move(&self) -> Option<Move> {
        debug!(
            "Get next move: pointer = {}/{}",
            self.step_pointer,
            self.history.len() - 1
        );

        if self.step_pointer == self.history.len() - 1 {
            return None;
        }

        self.history
            .get(self.step_pointer)
            .and_then(|s| s.as_move())
            .inspect(|m| debug!("Next move: {m:?}"))
    }

    /// Navigation through the history.
    ///
    /// Decrements the step pointer.
    pub fn step_back(&mut self) {
        if self.step_pointer == 0 {
            debug!("Stepping back in impossible. Current step is the last.");
            return;
        }

        // Subtraction is safe.
        self.step_pointer -= 1;

        let current_step_view = self
            .history
            .get(self.step_pointer)
            .expect("Step pointer out of bounds");

        if let Some(tx) = self.move_tx.as_ref() {
            let Step::Intermediate(s) = current_step_view else {
                panic!("Stepping back from any Step should be an intermediate step.");
            };

            tx.send(BoardAction::StepBack(s.into()));
        }

        debug!(
            "Stepping back: new pointer = {}/{}",
            self.step_pointer,
            self.history.len() - 1
        );
    }

    /// Navigation through the history.
    ///
    /// Increments the step pointer,
    /// provided the step pointer is not pointing to the last [`Step`].
    pub fn step_forward(&mut self) {
        let current_step_view = self
            .history
            .get(self.step_pointer)
            .expect("Step pointer out of bounds");

        // if self.step_pointer < self.history.len() - 1 {
        //     self.step_pointer += 1;
        // }

        // We can move forward from an intermediate step.
        let Step::Intermediate(s) = current_step_view else {
            debug!("Stepping forward in impossible. Current step is the last.");
            return;
        };

        self.step_pointer += 1;

        if let Some(tx) = self.move_tx.as_ref() {
            tx.send(BoardAction::StepForward(s.into()));
        }

        debug!(
            "Stepping forward: new pointer = {}/{}",
            self.step_pointer,
            self.history.len() - 1
        );
    }

    pub fn last_move(&self) -> Option<Move> {
        if let [.., Step::Intermediate(IntermediateStep { m, .. }), _last] = &self.history[..] {
            Some(*m)
        } else {
            None
        }
    }

    pub fn revert_last_move(&mut self) -> Option<Move> {
        debug!("Reverting the last move");

        let last = self.history.pop().expect(Self::INVARIANT_AT_LEAST_1_STEP);
        let last_but_1 = self.history.pop();

        let (board, m) = match last_but_1 {
            None => (last.into_board(), None),
            Some(Step::Intermediate(IntermediateStep { board, m })) => (board, Some(m)),
            Some(Step::Last(_)) => unreachable!("{}", Self::INVARIANT_LAST_BUT_1_IS_INTERMEDIATE),
        };

        self.history.push(Step::Last(board));
        self.step_pointer = self.history.len() - 1;

        m
    }

    pub fn side_to_move(&self) -> Color {
        self.current_board().side()
    }

    pub fn set_start(&mut self) {
        self.step_pointer = 0;

        if let Some(tx) = self.move_tx.as_ref() {
            tx.send(BoardAction::SetStartPosition);
        }
    }

    pub fn set_end(&mut self) {
        self.step_pointer = self.history.len() - 1;

        if let Some(tx) = self.move_tx.as_ref() {
            tx.send(BoardAction::SetEndPosition);
        }
    }

    fn represent_current_board(&self) -> String {
        format!(
            "Board:\n{}\nMove by: {}",
            self.current_board().pretty(PrettyStyle::Utf8),
            self.side_to_move().as_long_str()
        )
    }

    /// Retrieve references to:
    /// - the board immediately preceding the current last board,
    /// - and the move that transitions the preceding board to the current last board.
    fn last_intermediate_step(&self) -> Option<(&Board, &Move)> {
        if let [.., Step::Intermediate(IntermediateStep { board, m }), Step::Last(_)] =
            &self.history[..]
        {
            Some((board, m))
        } else {
            None
        }
    }

    fn current_board(&self) -> &Board {
        if let [.., Step::Last(board)] = &self.history[..] {
            board
        } else {
            panic!("[BUG] History must have the last step");
        }
    }

    /// Returns the [`Board`] currently referenced by the `step_pointer`.
    fn current_board_view(&self) -> &Board {
        self.current_step_view().as_board()
    }

    /// Returns the [`Step`] currently referenced by the `step_pointer`.
    fn current_step_view(&self) -> &Step {
        self.history
            .get(self.step_pointer)
            .expect("Step pointer out of bounds")
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

            tx.send(BoardAction::Apply(SanMove::new(&san_repr, piece, color)));
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
        self.current_board_view()
    }
}

enum Step {
    Last(Board),
    Intermediate(IntermediateStep),
}

impl Step {
    fn into_board(self) -> Board {
        match self {
            Step::Last(board) => board,
            Step::Intermediate(IntermediateStep { board, .. }) => board,
        }
    }

    fn as_move(&self) -> Option<Move> {
        match self {
            Step::Last(_) => None,
            Step::Intermediate(IntermediateStep { m, .. }) => Some(*m),
        }
    }

    fn as_board(&self) -> &Board {
        match self {
            Step::Last(board) => board,
            Step::Intermediate(IntermediateStep { board, .. }) => board,
        }
    }
}

struct IntermediateStep {
    board: Board,
    m: Move,
}

impl From<&IntermediateStep> for SanMove {
    fn from(step: &IntermediateStep) -> Self {
        let IntermediateStep { board, m } = step;

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

        SanMove::new(&san_repr, piece, color)
    }
}

#[derive(Error, Debug)]
pub enum HistoricalBoardError {
    #[error("Fen parsing error: {0}")]
    Fen(#[from] FenParseError),
    #[error("Move validation error: {0}")]
    Validation(#[from] ValidateError),
}
