#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_chessboard::{Action, Chessboard, PieceSet, PlayerColor};
use tracing::{debug, Level};

#[cfg(feature = "showcase")]
use futures_util::StreamExt;

/// Classes to render the chessboard.
const STYLE_CSS: Asset = asset!("/example/dist.css");

fn main() {
    dioxus_logger::init(Level::DEBUG).expect("failed to init logger");
    launch(App);
}

#[component]
fn App() -> Element {
    let mut color = use_signal(|| PlayerColor::White);
    let mut pieces_set = use_signal(|| PieceSet::Standard);
    let mut is_interactive = use_signal(|| true);
    let mut action = use_signal(|| None);
    let mut uci_content = use_signal(|| "".to_string());

    let _castling =
        "r3kbnr/ppp1qppp/2np4/4p3/2B1P1b1/P1N2N2/1PPP1PPP/R1BQK2R w KQkq - 1 6".to_string();

    let _promotion = "rnbqkb1r/ppppn1P1/7p/8/8/4BN2/PPp1BPPP/RN1QK2R w KQkq - 2 9".to_string();

    let uci_tx = use_coroutine(|mut rx: UnboundedReceiver<String>| async move {
        while let Some(msg) = rx.next().await {
            debug!("Chessboard reports: {msg}");
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: STYLE_CSS }

        div { class: "flex w-full space-x-4 min-h-screen",

            // Chessboard.
            div { class: "w-1/3 h-1/3 border border-black",
                Chessboard {
                    is_interactive: is_interactive.read().to_owned(),
                    color: color.read().to_owned(),
                    pieces_set: pieces_set.read().to_owned(),
                    position: _promotion,
                    action: action.read().to_owned(),
                    uci_tx,
                }
            }

            // Controls.
            div { class: "bg-white p-6 rounded-lg shadow-lg w-1/3 h-1/3 space-y-4",

                // Pieces' Set Radio Input
                div { class: "space-y-2 border border-gray-300 rounded-lg p-2",
                    label { class: "block text-gray-700 font-semibold", "Pieces' set" }
                    div { class: "flex items-center space-x-4",
                        label { class: "inline-flex items-center",
                            input {
                                r#type: "radio",
                                class: "form-radio text-blue-500",
                                name: "pieces-set",
                                value: "standard",
                                checked: true,
                                oninput: move |_ev| { *pieces_set.write() = PieceSet::Standard },
                            }
                            span { class: "ml-2 text-gray-700", "Standard" }
                        }

                        label { class: "inline-flex items-center",
                            input {
                                r#type: "radio",
                                class: "form-radio text-blue-500",
                                name: "pieces-set",
                                value: "funny",
                                oninput: move |_ev| { *pieces_set.write() = PieceSet::Funny },
                            }
                            span { class: "ml-2 text-gray-700", "Funny" }
                        }
                    }
                }

                // Interactivity Radio Input
                div { class: "space-y-2 border border-gray-300 rounded-lg p-2",
                    label { class: "block text-gray-700 font-semibold", "Interactivity" }
                    div { class: "flex items-center space-x-4",
                        label { class: "inline-flex items-center",
                            input {
                                r#type: "radio",
                                class: "form-radio text-blue-500",
                                name: "interactive",
                                value: "false",
                                oninput: move |_ev| { *is_interactive.write() = false },
                            }
                            span { class: "ml-2 text-gray-700", "False" }
                        }

                        label { class: "inline-flex items-center",
                            input {
                                r#type: "radio",
                                class: "form-radio text-blue-500",
                                name: "interactive",
                                value: "true",
                                checked: true,
                                oninput: move |_ev| { *is_interactive.write() = true },
                            }
                            span { class: "ml-2 text-gray-700", "True" }
                        }
                    }
                }

                // Inject UCI Move Text Input
                div { class: "space-y-2 border border-gray-300 rounded-lg p-2",
                    label {
                        class: "block text-gray-700 font-semibold",
                        r#for: "uci-move",
                        "Inject UCI move"
                    }
                    input {
                        r#type: "text",
                        id: "uci-move",
                        class: "form-input mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:border-blue-500 focus:ring focus:ring-blue-200 focus:ring-opacity-50",
                        placeholder: "Press Enter to send the move to Chessboard",
                        oninput: move |ev| {
                            *uci_content.write() = ev.value();
                        },
                        onkeypress: move |ev| {
                            if ev.key() == Key::Enter {
                                let value = uci_content.read().to_owned();
                                debug!("{value}");
                                *action.write() = Some(Action::make_move(&value));
                            }
                        },
                    }
                    // Hint Text
                    p { class: "text-gray-500 text-sm", "Try a popular first move \"e2e4\"" }
                }

                // Flip the Board Button
                div { class: "flex justify-start",
                    button {
                        class: "bg-blue-500 text-white font-bold py-2 px-4 rounded hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-opacity-50",
                        onclick: move |_| color.write().flip(),
                        "Flip the board"
                    }
                }

                // Go back.
                div { class: "flex justify-start",
                    button {
                        class: "bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded",
                        onclick: move |_| {
                            *action.write() = Some(Action::revert_move());
                        },
                        "Revert last move"
                    }
                }
            }
        }
    }
}
