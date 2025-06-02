use crate::history::HistoricalBoard;
use crate::move_builder::MoveBuilder;
use crate::piece::Piece;
use crate::PieceSet;
use dioxus::core_macro::{component, Props};
use dioxus::dioxus_core::Element;
use dioxus::prelude::*;
use owlchess::{Color, Coord};

/// A component rendering a square, potentially with a [`Piece`] inside.
#[component]
pub(crate) fn Square(props: SquareProps) -> Element {
    let board = use_context::<Signal<HistoricalBoard>>();
    let mut move_builder = use_context::<Signal<MoveBuilder>>();

    // Highlight a selected square if no animation is in progress.
    let is_selected = move_builder.read().find_animation(props.coord).is_none()
        && matches!(move_builder.read().src(), Some(src) if src == props.coord);

    rsx! {
        div {
            id: format!("{}", props.coord),
            class: if is_selected { "move-source" },
            onclick: move |_ev| {
                if props.is_interactive {
                    move_builder.write().put_square_coord(props.coord, &board.read());
                }
            },
            Piece {
                coord: props.coord,
                color: props.color,
                pieces_set: props.pieces_set,
            }
        }
    }
}

#[derive(Props, Debug, PartialEq, Clone)]
pub(crate) struct SquareProps {
    is_interactive: bool,
    coord: Coord,
    color: Color,
    pieces_set: PieceSet,
}
