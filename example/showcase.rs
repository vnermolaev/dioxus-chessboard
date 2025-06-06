#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_chessboard::{Action, BoardAction, Chessboard, ChessboardProps, Color, PieceSet};
use tracing::{debug, Level};

#[cfg(feature = "showcase")]
use futures_util::StreamExt;

/// Classes to render the chessboard.
const STYLE_CSS: Asset = asset!("/example/dist.css");

const LEFT: Asset = asset!("/example/img/left.svg");
const LEFT_WALL: Asset = asset!("/example/img/left_to_the_wall.svg");
const RIGHT: Asset = asset!("/example/img/right.svg");
const RIGHT_WALL: Asset = asset!("/example/img/right_to_the_wall.svg");

fn main() {
    dioxus_logger::init(Level::DEBUG).expect("failed to init logger");
    launch(App);
}

#[component]
fn App() -> Element {
    let mut player_color = use_signal(|| Color::White);
    let mut single_player_mode = use_signal(|| false);
    let mut pieces_set = use_signal(|| PieceSet::Standard);
    let mut is_interactive = use_signal(|| true);
    let mut action = use_signal(|| None);
    let mut san_content = use_signal(|| "".to_string());

    let castling =
        "r3kbnr/ppp1qppp/2np4/4p3/2B1P1b1/P1N2N2/1PPP1PPP/R1BQK2R w KQkq - 1 6".to_string();

    let promotion = "rnbqkb1r/ppppn1P1/7p/8/8/4BN2/PPp1BPPP/RN1QK2R w KQkq - 2 9".to_string();

    let san_tx = use_coroutine(|mut rx: UnboundedReceiver<BoardAction>| async move {
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
                    player_color: player_color.read().to_owned(),
                    single_player_mode: single_player_mode.read().to_owned(),
                    pieces_set: pieces_set.read().to_owned(),
                    action: action.read().to_owned(),
                    san_tx,
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

                // Single player mode Radio Input
                div { class: "space-y-2 border border-gray-300 rounded-lg p-2",
                    label { class: "block text-gray-700 font-semibold", "Single player mode" }
                    span { class: "text-xs text-gray-500", "(Player's pieces are at the bottom)" }
                    div { class: "flex items-center space-x-4",
                        label { class: "inline-flex items-center",
                            input {
                                r#type: "radio",
                                class: "form-radio text-blue-500",
                                name: "single_player_mode",
                                value: "false",
                                checked: true,
                                oninput: move |_ev| { *single_player_mode.write() = false },
                            }
                            span { class: "ml-2 text-gray-700", "False" }
                        }

                        label { class: "inline-flex items-center",
                            input {
                                r#type: "radio",
                                class: "form-radio text-blue-500",
                                name: "single_player_mode",
                                value: "true",
                                oninput: move |_ev| { *single_player_mode.write() = true },
                            }
                            span { class: "ml-2 text-gray-700", "True" }
                        }
                    }
                }

                // Inject SAN Move Text Input
                div { class: "space-y-2 border border-gray-300 rounded-lg p-2",
                    label {
                        class: "block text-gray-700 font-semibold",
                        r#for: "san-move",
                        "Inject SAN-encoded move"
                    }
                    input {
                        r#type: "text",
                        id: "san-move",
                        class: "form-input mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:border-blue-500 focus:ring focus:ring-blue-200 focus:ring-opacity-50",
                        placeholder: "Press Enter to send the move to Chessboard",
                        oninput: move |ev| {
                            *san_content.write() = ev.value();
                        },
                        onkeypress: move |ev| {
                            if ev.key() == Key::Enter {
                                let value = san_content.read().to_owned();
                                debug!("{value}");
                                *action.write() = Some(Action::make_move(&value));
                            }
                        },
                    }
                    // Hint Text
                    p { class: "text-gray-500 text-sm", "Try a popular first move for white \"e4\"" }
                }

                // Positions Radio Input
                div { class: "space-y-2 border border-gray-300 rounded-lg p-2",
                    label { class: "block text-gray-700 font-semibold", "Positions" }
                    div { class: "flex items-center space-x-4",
                        label { class: "inline-flex items-center",
                            input {
                                r#type: "radio",
                                class: "form-radio text-blue-500",
                                name: "position",
                                value: "default",
                                checked: true,
                                oninput: move |_ev| {
                                    *action.write() = Some(
                                        Action::set_position(ChessboardProps::default_position()),
                                    );
                                },
                            }
                            span { class: "ml-2 text-gray-700", "Starting position" }
                        }

                        label { class: "inline-flex items-center",
                            input {
                                r#type: "radio",
                                class: "form-radio text-blue-500",
                                name: "position",
                                value: "castling",
                                oninput: move |_ev| {
                                    *action.write() = Some(Action::set_position(&castling));
                                },
                            }
                            span { class: "ml-2 text-gray-700", "Test Castling" }
                        }

                        label { class: "inline-flex items-center",
                            input {
                                r#type: "radio",
                                class: "form-radio text-blue-500",
                                name: "position",
                                value: "promotion",
                                oninput: move |_ev| {
                                    *action.write() = Some(Action::set_position(&promotion));
                                },
                            }
                            span { class: "ml-2 text-gray-700", "Test Promotion" }
                        }
                    }
                }

                // Navigation
                div { class: "space-y-2 border border-gray-300 rounded-lg p-2",
                    label { class: "block text-gray-700 font-semibold", "Navigation" }
                    div { class: "inline-flex items-center gap-2 rounded-lg bg-gray-100 p-2 shadow",

                        // «| (to first / “left-to-the-wall”)
                        button {
                            class: "p-1 rounded transition filter hover:bg-gray-400",
                            onclick: move |_| action.set(Some(Action::set_start_position())),
                            img {
                                src: LEFT_WALL,
                                alt: "First",
                                class: "select-none",
                            }
                        }

                        // ‹ (previous / “left”)
                        button {
                            class: "p-1 rounded transition filter hover:bg-gray-400",
                            onclick: move |_| action.set(Some(Action::prev())),
                            img {
                                src: LEFT,
                                alt: "Previous",
                                class: "select-none",
                            }
                        }

                        // › (next / “right”)
                        button {
                            class: "p-1 rounded transition filter hover:bg-gray-400",
                            onclick: move |_| action.set(Some(Action::next())),
                            img {
                                src: RIGHT,
                                alt: "Next",
                                class: "select-none",
                            }
                        }

                        // |» (to last / “right-to-the-wall”)
                        button {
                            class: "p-1 rounded transition filter hover:bg-gray-400",
                            onclick: move |_| action.set(Some(Action::set_end_position())),
                            img {
                                src: RIGHT_WALL,
                                alt: "Last",
                                class: "select-none",
                            }
                        }
                    }
                }


                // Flip the Board Button
                div { class: "flex justify-start",
                    button {
                        class: "bg-blue-500 text-white font-bold py-2 px-4 rounded hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-opacity-50",
                        onclick: move |_| {
                            let inverted_color = player_color.read().inv();
                            *player_color.write() = inverted_color;
                        },
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
