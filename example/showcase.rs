#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_chessboard::{Action, Chessboard, ChessboardClient, PieceSet, PlayerColor};
use tracing::{debug, Level};

#[cfg(feature = "showcase")]
use futures_util::StreamExt;

const _TAILWIND_URL: &str = manganis::mg!(file("public/tailwind.css"));

fn main() {
    dioxus_logger::init(Level::DEBUG).expect("failed to init logger");
    launch(App);
}

#[component]
fn App() -> Element {
    let client = ChessboardClient::get();
    client.send(Action::Uci("e2e4".to_string()));

    let mut color = use_signal(|| PlayerColor::White);
    let mut pieces_set = use_signal(|| PieceSet::Standard);
    let mut uci_content = use_signal(|| "".to_string());
    let mut uci = use_signal(|| None);

    let _castling =
        "r3kbnr/ppp1qppp/2np4/4p3/2B1P1b1/P1N2N2/1PPP1PPP/R1BQK2R w KQkq - 1 6".to_string();

    let _promotion = "rnbqkb1r/ppppn1P1/7p/8/8/4BN2/PPp1BPPP/RN1QK2R w KQkq - 2 9".to_string();

    let uci_tx = use_coroutine(|mut rx: UnboundedReceiver<String>| async move {
        while let Some(msg) = rx.next().await {
            debug!("Chessboard reports: {msg}");
        }
    });

    rsx! {
        div {
            class: "bg-gray-100 flex w-full space-x-4 min-h-screen",

            // Chessboard.
            div {
                class: "w-1/3 h-1/3 border border-black",
                Chessboard {
                    color: color.read().to_owned(),
                    pieces_set:  pieces_set.read().to_owned(),
                    uci: uci.read().to_owned(),
                    uci_tx
                }
            }

            // Controls.
            div {
                class: "bg-white p-6 rounded-lg shadow-lg w-1/3 h-1/3 space-y-4",

                // Pieces' Set Radio Input
                div {
                    class: "space-y-2 border border-gray-300 rounded-lg p-2",
                    label {
                        class: "block text-gray-700 font-semibold",
                        "Pieces' set"
                    },
                    div {
                        class: "flex items-center space-x-4",
                        label {
                            class: "inline-flex items-center",
                            input {
                                r#type: "radio",
                                class: "form-radio text-blue-500",
                                name: "pieces-set",
                                value: "standard",
                                checked: true,
                                oninput: move |_ev| {
                                    *pieces_set.write() = PieceSet::Standard
                                }
                            },
                            span {
                                class: "ml-2 text-gray-700",
                                "Standard"
                            }
                        },
                        label {
                            class: "inline-flex items-center",
                            input {
                                r#type: "radio",
                                class: "form-radio text-blue-500",
                                name: "pieces-set",
                                value: "funny",
                                oninput: move |_ev| {
                                    *pieces_set.write() = PieceSet::Funny
                                }
                            },
                            span {
                                class: "ml-2 text-gray-700",
                                "Funny"
                            }
                        }
                    }
                },

                // Inject UCI Move Text Input
                div {
                    class: "space-y-2 border border-gray-300 rounded-lg p-2",
                    label {
                        class: "block text-gray-700 font-semibold",
                        r#for: "uci-move",
                        "Inject UCI move"
                    },
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
                                *uci.write() = Some(value);
                            }
                        }
                    }
                    // Hint Text
                    p {
                        class: "text-gray-500 text-sm",
                        "Try a popular first move \"e2e4\""
                    }
                }

                // Flip the Board Button
                div {
                    class: "flex justify-start",
                    button {
                        class: "bg-blue-500 text-white font-bold py-2 px-4 rounded hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-opacity-50",
                        onclick: move|_| {
                            color.write().flip();
                            client.send(Action::Uci("Flipping the board".to_string()));
                        },
                        "Flip the board"
                    }
                },
            }
        }
    }
}
