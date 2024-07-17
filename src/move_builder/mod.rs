use crate::move_builder::state::State;
use dioxus::prelude::Coroutine;
use owlchess::moves::{uci, PromotePiece, Style};
use owlchess::{Board, Coord, File, Move, Rank};
use std::ops::{Deref, DerefMut};

mod applicable_move;
mod promotion;
mod state;

pub struct MoveBuilder {
    /// When the move reaches [State::ApplicableMove],
    /// the corresponding uci will be sent out.
    uci_move_tx: Option<Coroutine<String>>,
    /// [State] of the builder.
    state: State,
}

impl Deref for MoveBuilder {
    type Target = State;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
impl DerefMut for MoveBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl MoveBuilder {
    pub fn new(uci_move_tx: Option<Coroutine<String>>) -> Self {
        Self {
            uci_move_tx,
            state: State::new(),
        }
    }

    pub fn src(&self) -> Option<Coord> {
        self.deref().src()
    }

    #[allow(dead_code)] // Maybe used in the future.
    pub fn dst(&self) -> Option<Coord> {
        self.deref().dst()
    }

    /// Puts a square into [State].
    pub fn put_square(&mut self, file: File, rank: Rank, board: &Board) {
        self.deref_mut().put_square(file, rank, board)
    }

    pub fn put_uci_move(&mut self, uci: &str, board: &Board) -> Result<(), uci::ParseError> {
        self.deref_mut().put_uci_move(uci, board)
    }

    pub fn check_promotion(&self) -> Option<(Coord, Coord)> {
        self.deref().check_promotion()
    }

    pub fn promote(&mut self, piece: PromotePiece, board: &Board) {
        self.deref_mut().promote(piece, board)
    }

    pub fn animations(&self) -> Vec<(Coord, Coord)> {
        self.deref().animations()
    }

    pub fn finalize(&mut self, board: &Board) -> Option<Move> {
        let m = self.deref_mut().finalize();

        if let (Some(ref m), Some(ref uci_move_tx)) = (m, self.uci_move_tx) {
            // There is a valid move and a coroutine to report it.
            let uci = m
                .styled(board, Style::Uci)
                .expect("Move must be correctly finalized")
                .to_string();
            uci_move_tx.send(uci);
        }

        m
    }
}
