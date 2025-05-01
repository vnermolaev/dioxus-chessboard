use crate::chessboard::SanMove;
use crate::move_builder::state::State;
use dioxus::prelude::Coroutine;
use owlchess::moves::{san, PromotePiece, Style};
use owlchess::{Board, Color, Coord, Move};
use std::ops::{Deref, DerefMut};

mod applicable_move;
mod promotion;
mod state;

pub struct MoveBuilder {
    /// When the move reaches [`State::ApplicableMove`],
    /// the corresponding SAN will be sent out.
    san_move_tx: Option<Coroutine<SanMove>>,
    /// [`State`] of the builder.
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
    pub fn new(san_move_tx: Option<Coroutine<SanMove>>) -> Self {
        Self {
            san_move_tx,
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

    /// Tries to apply a SAN move, if illegal it will error.
    pub fn apply_san_move(&mut self, san: &str, board: &Board) -> Result<(), san::ParseError> {
        self.deref_mut().apply_san_move(san, board)
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
    pub fn animation_displacement(&self, source: Coord, color: Color) -> Option<(i16, i16)> {
        fn coord_diff(c1: Coord, c2: Coord) -> (i16, i16) {
            (
                c1.file() as i16 - c2.file() as i16,
                c1.rank() as i16 - c2.rank() as i16,
            )
        }

        self.find_animation(source).map(|dst| {
            let (x, y) = coord_diff(dst, source);
            let c = if let Color::White = color { 1 } else { -1 };
            (c * x * 100, c * y * 100)
        })
    }

    pub fn finalize(&mut self, board: &Board) -> MoveAction {
        let m = self.deref_mut().finalize();

        if let (MoveAction::Apply(ref m), Some(ref san_move_tx)) = (m, self.san_move_tx) {
            // There is a valid move and a coroutine to report it.
            let san_repr = m
                .styled(board, Style::San)
                .expect("Move must be correctly finalized")
                .to_string();

            let src_cell = board.get(m.src());

            let piece = src_cell
                .piece()
                .expect("Move is valid, thus src must contain a piece");

            let color = src_cell
                .color()
                .expect("Move is valid, thus src must contain a piece");

            san_move_tx.send(SanMove::new(&san_repr, piece, color));
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
