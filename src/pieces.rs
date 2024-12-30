use dioxus::prelude::Asset;
use funny::*;
use owlchess::{Cell, Color, Piece};
use standard::*;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PieceSet {
    Standard,
    Funny,
}

pub(crate) fn compute_piece_img_src(pieces_set: PieceSet, cell: Cell) -> Option<Asset> {
    if cell.is_occupied() {
        // Unwraps are safe because the cell is occupied.
        let piece = cell.piece().unwrap();
        let color = cell.color().unwrap();
        Some(match (pieces_set, piece, color) {
            // Pieces set 1.
            (PieceSet::Standard, Piece::Bishop, Color::White) => PIECE_1_B_WHITE,
            (PieceSet::Standard, Piece::King, Color::White) => PIECE_1_K_WHITE,
            (PieceSet::Standard, Piece::Knight, Color::White) => PIECE_1_N_WHITE,
            (PieceSet::Standard, Piece::Pawn, Color::White) => PIECE_1_P_WHITE,
            (PieceSet::Standard, Piece::Queen, Color::White) => PIECE_1_Q_WHITE,
            (PieceSet::Standard, Piece::Rook, Color::White) => PIECE_1_R_WHITE,
            (PieceSet::Standard, Piece::Bishop, Color::Black) => PIECE_1_B_BLACK,
            (PieceSet::Standard, Piece::King, Color::Black) => PIECE_1_K_BLACK,
            (PieceSet::Standard, Piece::Knight, Color::Black) => PIECE_1_N_BLACK,
            (PieceSet::Standard, Piece::Pawn, Color::Black) => PIECE_1_P_BLACK,
            (PieceSet::Standard, Piece::Queen, Color::Black) => PIECE_1_Q_BLACK,
            (PieceSet::Standard, Piece::Rook, Color::Black) => PIECE_1_R_BLACK,

            // Pieces set 2.
            (PieceSet::Funny, Piece::Bishop, Color::White) => PIECE_2_B_WHITE,
            (PieceSet::Funny, Piece::King, Color::White) => PIECE_2_K_WHITE,
            (PieceSet::Funny, Piece::Knight, Color::White) => PIECE_2_N_WHITE,
            (PieceSet::Funny, Piece::Pawn, Color::White) => PIECE_2_P_WHITE,
            (PieceSet::Funny, Piece::Queen, Color::White) => PIECE_2_Q_WHITE,
            (PieceSet::Funny, Piece::Rook, Color::White) => PIECE_2_R_WHITE,
            (PieceSet::Funny, Piece::Bishop, Color::Black) => PIECE_2_B_BLACK,
            (PieceSet::Funny, Piece::King, Color::Black) => PIECE_2_K_BLACK,
            (PieceSet::Funny, Piece::Knight, Color::Black) => PIECE_2_N_BLACK,
            (PieceSet::Funny, Piece::Pawn, Color::Black) => PIECE_2_P_BLACK,
            (PieceSet::Funny, Piece::Queen, Color::Black) => PIECE_2_Q_BLACK,
            (PieceSet::Funny, Piece::Rook, Color::Black) => PIECE_2_R_BLACK,
        })
    } else {
        None
    }
}

/// Piece set 1.
mod standard {
    use dioxus::prelude::*;

    /// White pieces.
    pub const PIECE_1_B_WHITE: Asset = asset!("public/pieces/standard/b-white.svg");
    pub const PIECE_1_K_WHITE: Asset = asset!("public/pieces/standard/k-white.svg");
    pub const PIECE_1_N_WHITE: Asset = asset!("public/pieces/standard/n-white.svg");
    pub const PIECE_1_P_WHITE: Asset = asset!("public/pieces/standard/p-white.svg");
    pub const PIECE_1_Q_WHITE: Asset = asset!("public/pieces/standard/q-white.svg");
    pub const PIECE_1_R_WHITE: Asset = asset!("public/pieces/standard/r-white.svg");

    ///Black pieces.
    pub const PIECE_1_B_BLACK: Asset = asset!("public/pieces/standard/b-black.svg");
    pub const PIECE_1_K_BLACK: Asset = asset!("public/pieces/standard/k-black.svg");
    pub const PIECE_1_N_BLACK: Asset = asset!("public/pieces/standard/n-black.svg");
    pub const PIECE_1_P_BLACK: Asset = asset!("public/pieces/standard/p-black.svg");
    pub const PIECE_1_Q_BLACK: Asset = asset!("public/pieces/standard/q-black.svg");
    pub const PIECE_1_R_BLACK: Asset = asset!("public/pieces/standard/r-black.svg");
}

/// Pieces set 2.
mod funny {
    use dioxus::prelude::*;

    /// White pieces.
    pub const PIECE_2_B_WHITE: Asset = asset!("public/pieces/funny/b-white.svg");
    pub const PIECE_2_K_WHITE: Asset = asset!("public/pieces/funny/k-white.svg");
    pub const PIECE_2_N_WHITE: Asset = asset!("public/pieces/funny/n-white.svg");
    pub const PIECE_2_P_WHITE: Asset = asset!("public/pieces/funny/p-white.svg");
    pub const PIECE_2_Q_WHITE: Asset = asset!("public/pieces/funny/q-white.svg");
    pub const PIECE_2_R_WHITE: Asset = asset!("public/pieces/funny/r-white.svg");

    /// Black pieces.
    pub const PIECE_2_B_BLACK: Asset = asset!("public/pieces/funny/b-black.svg");
    pub const PIECE_2_K_BLACK: Asset = asset!("public/pieces/funny/k-black.svg");
    pub const PIECE_2_N_BLACK: Asset = asset!("public/pieces/funny/n-black.svg");
    pub const PIECE_2_P_BLACK: Asset = asset!("public/pieces/funny/p-black.svg");
    pub const PIECE_2_Q_BLACK: Asset = asset!("public/pieces/funny/q-black.svg");
    pub const PIECE_2_R_BLACK: Asset = asset!("public/pieces/funny/r-black.svg");
}
