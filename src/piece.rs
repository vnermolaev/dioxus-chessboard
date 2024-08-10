use crate::chessboard::PlayerColor;
use crate::historical_board::HistoricalBoard;
use crate::move_builder::MoveBuilder;
use crate::pieces::compute_piece_img_src;
use crate::{finalize, PieceSet};
use dioxus::prelude::*;
use owlchess::Coord;

/// Component rendering pieces on [owlchess::Board].
#[component]
pub(crate) fn Piece(props: PieceProps) -> Element {
    let mut board = use_context::<Signal<HistoricalBoard>>();
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

    let Some(img_src) = compute_piece_img_src(props.pieces_set, board.read().get(props.coord))
    else {
        return rsx! {};
    };

    // Animation.
    let (x, y) = move_builder
        .read()
        .animations()
        .iter()
        .find(|(src, _)| *src == props.coord)
        .map(|(_, dst)| *dst)
        .map(|dst| {
            let (x, y) = coord_diff(dst, props.coord);
            let c = if let PlayerColor::White = props.color {
                1
            } else {
                -1
            };
            (c * x * 100, c * y * 100)
        })
        .unwrap_or_default();
    let is_animation_running = x != 0 || y != 0;

    // Additional styling.
    let mut classes = Vec::new();

    // Highlight if no animation is in progress.
    if !is_animation_running && matches!(move_builder.read().src(), Some(src) if src == props.coord)
    {
        classes.push("bg-lime-700/50");
    }

    // If piece is to be moved, bring it forward.
    classes.push(if x != 0 || y != 0 { "z-20" } else { "z-10" });

    let class = classes.join(" ");

    let ontransitionend = move |_ev| {
        finalize(&mut move_builder, &mut board);
    };

    rsx! {
        img {
            src: img_src,
            class,
            transition: "transform 0.5s ease",
            transform: "translateX({x}%) translateY({y}%)",
            ontransitionend,
        }
    }
}

#[derive(Props, Debug, PartialEq, Clone)]
pub(crate) struct PieceProps {
    coord: Coord,
    color: PlayerColor,
    pieces_set: PieceSet,
}

fn coord_diff(c1: Coord, c2: Coord) -> (i16, i16) {
    (
        c1.file() as i16 - c2.file() as i16,
        c1.rank() as i16 - c2.rank() as i16,
    )
}
