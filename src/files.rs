use crate::chessboard::PlayerColor;
use dioxus::prelude::*;
use owlchess::File;

/// Component applying the file notation to [Chessboard].
#[component]
pub(crate) fn Files(props: FilesProps) -> Element {
    let files = match props.color {
        PlayerColor::White => File::iter().collect::<Vec<_>>(),
        PlayerColor::Black => File::iter().collect::<Vec<_>>().into_iter().rev().collect(),
    };

    rsx! {
        div {
            id: "files",
            class: "absolute w-full bottom-0 text-xs flex flex-row font-semibold",
            for file in files {
                div {
                    class: "pointer-events-none pl-1 w-1/8",
                    style: format!("color: {}",
                        match props.color {
                            PlayerColor::White => if file.index() % 2 == 0 { "var(--color-dark)" } else { "var(--color-light)"},
                            PlayerColor::Black => if file.index() % 2 == 0 { "var(--color-light)"} else  { "var(--color-dark)" },
                        }
                    ),
                    { file.to_string() }
                }
            }
        }
    }
}

#[derive(Props, Debug, PartialEq, Clone)]
pub(crate) struct FilesProps {
    color: PlayerColor,
}
