use crate::chessboard::PlayerColor;
use dioxus::prelude::*;
use owlchess::Rank;

/// Component applying the rank notation to [Chessboard].
#[component]
pub(crate) fn Ranks(props: RanksProps) -> Element {
    let ranks = match props.color {
        PlayerColor::White => Rank::iter().collect::<Vec<_>>(),
        PlayerColor::Black => Rank::iter().collect::<Vec<_>>().into_iter().rev().collect(),
    };

    rsx! {
        div {
            id: "ranks",
            class: "ranks",
            for rank in ranks {
                div {
                    class: "rank h-1/8",
                    style: format!(
                        "color: {}",
                        match props.color {
                            PlayerColor::White => {
                                if rank.index() % 2 == 0 {
                                    "var(--color-light)"
                                } else {
                                    "var(--color-dark)"
                                }
                            }
                            PlayerColor::Black => {
                                if rank.index() % 2 == 0 {
                                    "var(--color-dark)"
                                } else {
                                    "var(--color-light)"
                                }
                            }
                        },
                    ),
                    {rank.to_string()}
                }
            }
        }
    }
}

#[derive(Props, Debug, PartialEq, Clone)]
pub(crate) struct RanksProps {
    color: PlayerColor,
}
