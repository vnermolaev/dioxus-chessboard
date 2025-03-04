use crate::chessboard::PlayerColor;
use crate::historical_board::HistoricalBoard;
use crate::move_builder::MoveBuilder;
use crate::pieces::compute_piece_img_src;
use crate::{finalize, PieceSet};
use dioxus::prelude::*;
use owlchess::moves::PromotePiece;
use owlchess::{Cell, File, Piece, Rank};

/// Component rendering a selection of pieces when a pawn gets promoted.
#[component]
pub(crate) fn Promotion(props: PromotionProperties) -> Element {
    let board = use_context::<Signal<HistoricalBoard>>();
    let move_builder = use_context::<Signal<MoveBuilder>>();

    // If no promotion, return.
    let Some((src, dst)) = move_builder.read().check_promotion() else {
        return rsx! {};
    };

    let color = board
        .read()
        .get(src)
        .color()
        .expect("Promotion is verified to be in progress");

    let pieces = {
        let pieces = vec![
            PromotePiece::Queen,
            PromotePiece::Knight,
            PromotePiece::Rook,
            PromotePiece::Bishop,
        ];

        match (props.color, dst.rank()) {
            // Player plays for White and White is promoting or
            // player plays for Black and Black is promoting
            (PlayerColor::White, Rank::R8) | (PlayerColor::Black, Rank::R1) => pieces,
            // Player plays for White and Black is promoting or
            // player plays for Black and White is promoting and
            (PlayerColor::White, Rank::R1) | (PlayerColor::Black, Rank::R8) => {
                pieces.into_iter().rev().collect()
            }
            _ => unreachable!("Promotion happens only if destination square has Rank 1 or 8"),
        }
    };

    let promotion_container_classes = {
        let mut classes = vec!["promotion-position", "w-1/8", "h-1/2"];

        let shift = match props.color {
            PlayerColor::White => File::iter().position(|f| f == dst.file()).unwrap(),
            PlayerColor::Black => 7 - File::iter().position(|f| f == dst.file()).unwrap(),
        };
        let left = format!("left-{shift}/8");
        classes.push(&left);

        let alignment = match (props.color, dst.rank()) {
            // Player plays for White and White is promoting or
            // player plays for Black and Black is promoting
            (PlayerColor::White, Rank::R8) | (PlayerColor::Black, Rank::R1) => "promotion-top",
            // Player plays for White and Black is promoting or
            // player plays for Black and White is promoting and
            (PlayerColor::White, Rank::R1) | (PlayerColor::Black, Rank::R8) => "promotion-bottom",
            _ => unreachable!("Promotion happens only if destination square has Rank 1 or 8"),
        };
        classes.push(alignment);

        classes.join(" ")
    };

    rsx! {
        div { class: promotion_container_classes,
            for piece in pieces {
                PromotePiece { color, piece, pieces_set: props.pieces_set }
            }
        }
    }
}

#[derive(Props, Debug, PartialEq, Clone)]
pub struct PromotionProperties {
    color: PlayerColor,
    pieces_set: PieceSet,
}

/// Component rendering a promotion piece for a pawn.
#[component]
fn PromotePiece(props: PromotePieceProps) -> Element {
    let mut board = use_context::<Signal<HistoricalBoard>>();
    let mut move_builder = use_context::<Signal<MoveBuilder>>();

    let cell = Cell::from_parts(props.color, Piece::from(props.piece));

    let src = compute_piece_img_src(props.pieces_set, cell)
        .unwrap_or_else(|| panic!("Cell {cell} must be occupied"));

    let onclick = move |_ev| {
        {
            let board = board.read();
            move_builder.write().promote(props.piece, &board);
        }
        finalize(&mut move_builder, &mut board);
    };

    rsx! {
        div { class: "promotion-piece-container",
            img { class: "promotion-piece", src, onclick }
        }
    }
}

#[derive(Props, Debug, PartialEq, Clone)]
struct PromotePieceProps {
    color: owlchess::Color,
    piece: PromotePiece,
    pieces_set: PieceSet,
}
