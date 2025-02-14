use crate::move_builder::state::State;
use crate::PlayerColor;
use dioxus::prelude::Coroutine;
use owlchess::moves::{uci, PromotePiece, Style};
use owlchess::{Board, Coord, Move};
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
    pub fn put_square_coord(&mut self, coord: Coord, board: &Board) {
        self.deref_mut().put_square_coord(coord, board)
    }

    /// Tries to apply a UCI move, if illegal it will error.
    pub fn apply_uci_move(&mut self, uci: &str, board: &Board) -> Result<(), uci::ParseError> {
        self.deref_mut().apply_uci_move(uci, board)
    }

    /// Reverts the [Move] m.
    /// Internally maintains a fictional move which always finalizes to [None].
    pub fn revert_move(&mut self, m: Move) {
        self.deref_mut().revert_move(m);
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

    /// Find a destination [Coord] for an animation with its source at a given [Coord], if it exists.
    pub fn find_animation(&self, source: Coord) -> Option<Coord> {
        self.animations()
            .iter()
            .find_map(|(src, dst)| if *src == source { Some(*dst) } else { None })
    }

    /// Computes a displacement in percentage for animation with its source at a given [Coord].
    pub fn animation_displacement(&self, source: Coord, color: PlayerColor) -> Option<(i16, i16)> {
        fn coord_diff(c1: Coord, c2: Coord) -> (i16, i16) {
            (
                c1.file() as i16 - c2.file() as i16,
                c1.rank() as i16 - c2.rank() as i16,
            )
        }

        self.find_animation(source).map(|dst| {
            let (x, y) = coord_diff(dst, source);
            let c = if let PlayerColor::White = color {
                1
            } else {
                -1
            };
            (c * x * 100, c * y * 100)
        })
    }

    pub fn finalize(&mut self, board: &Board) -> MoveAction {
        let m = self.deref_mut().finalize();

        if let (MoveAction::Apply(ref m), Some(ref uci_move_tx)) = (m, self.uci_move_tx) {
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

#[derive(Clone, Copy)]
pub enum MoveAction {
    None,
    Apply(Move),
    Revert,
}
