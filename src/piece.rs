use crate::history::HistoricalBoard;
use crate::move_builder::MoveBuilder;
use crate::pieces::compute_piece_img_src;
use crate::{finalize, PieceSet};
use dioxus::prelude::*;
use owlchess::{Color, Coord};

/// Component rendering pieces on [owlchess::Board].
#[component]
pub(crate) fn Piece(props: PieceProps) -> Element {
    let mut move_builder = use_context::<Signal<MoveBuilder>>();

    // If promotion is required,
    // do _not_ place pieces in src and dst.
    let promotion_requirements = move_builder.read().check_promotion();
    if promotion_requirements
        .map(|(src, dst)| props.coord == src || props.coord == dst)
        .unwrap_or_default()
    {
        return rsx! {};
    }

    let mut board = use_context::<Signal<HistoricalBoard>>();

    let Some(img_src) = compute_piece_img_src(props.pieces_set, board.read().get(props.coord))
    else {
        return rsx! {};
    };

    // Animation.
    let animation = move_builder
        .read()
        .animation_displacement(props.coord, props.color);

    let ontransitionend = move |_ev| {
        finalize(&mut move_builder, &mut board);
    };

    rsx! {
        img {
            src: img_src,
            class: "scaled",
            z_index: if animation.is_some() { "10000" },
            transition: if animation.is_some() { "transform 0.5s ease" },
            transform: if let Some((x, y)) = animation { "translateX({x}%) translateY({y}%) scale(var(--piece-scale))" },
            ontransitionend,
        }
    }
}

#[derive(Props, Debug, PartialEq, Clone)]
pub(crate) struct PieceProps {
    coord: Coord,
    color: Color,
    pieces_set: PieceSet,
}
