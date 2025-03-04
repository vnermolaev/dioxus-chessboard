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
            class: "files",
            for file in files {
                div {
                    class: "file w-1/8",
                    style: format!(
                        "color: {}",
                        match props.color {
                            PlayerColor::White => {
                                if file.index() % 2 == 0 {
                                    "var(--color-light)"
                                } else {
                                    "var(--color-dark)"
                                }
                            }
                            PlayerColor::Black => {
                                if file.index() % 2 == 0 {
                                    "var(--color-dark)"
                                } else {
                                    "var(--color-light)"
                                }
                            }
                        },
                    ),
                    {file.to_string()}
                }
            }
        }
    }
}

#[derive(Props, Debug, PartialEq, Clone)]
pub(crate) struct FilesProps {
    color: PlayerColor,
}
