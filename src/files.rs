use dioxus::prelude::*;
use owlchess::{Color, File};

/// Component applying the file notation to [Chessboard].
#[component]
pub(crate) fn Files(props: FilesProps) -> Element {
    let files = match props.color {
        Color::White => File::iter().collect::<Vec<_>>(),
        Color::Black => File::iter().collect::<Vec<_>>().into_iter().rev().collect(),
    };

    rsx! {
        div { id: "files", class: "files",
            for file in files {
                div {
                    class: "file w-1/8",
                    style: format!(
                        "color: {}",
                        match props.color {
                            Color::White => {
                                if file.index() % 2 == 0 {
                                    "var(--color-light)"
                                } else {
                                    "var(--color-dark)"
                                }
                            }
                            Color::Black => {
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
    color: Color,
}
