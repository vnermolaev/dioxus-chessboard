use crate::communication::get_chessboard_receiver;
use crate::files::Files;
use crate::historical_board::HistoricalBoard;
use crate::move_builder::MoveBuilder;
use crate::piece::Piece;
use crate::promotion::Promotion;
use crate::ranks::Ranks;
use crate::PieceSet;
use dioxus::prelude::*;
use owlchess::board::PrettyStyle;
use owlchess::{Coord, File, Rank};
use tracing::{debug, info, warn};

/// Classes to render the chessboard.
const CHESSBOARD_CLASSES: &str = asset!("public/chessboard.css");

/// Component rendering [Chessboard].
#[component]
pub fn Chessboard(props: ChessboardProps) -> Element {
    let props = props.complete();

    use_context_provider(|| {
        Signal::new(
            HistoricalBoard::from_fen(&props.position)
                .expect("Board must be constructible from a valid position"),
        )
    });

    use_context_provider(|| Signal::new(MoveBuilder::new(props.uci_tx)));

    let board = use_context::<Signal<HistoricalBoard>>();
    let mut move_builder = use_context::<Signal<MoveBuilder>>();

    // Spawn processing of injected actions.
    if let Some(mut rx) = get_chessboard_receiver() {
        debug!("Initializing chessboard message loop");
        spawn(async move {
            while let Some(action) = rx.recv().await {
                debug!("Chessboard must act: {action:?}");
                match action {
                    Action::MakeUciMove(uci) => {
                        let board = board.read();

                        if move_builder.write().apply_uci_move(&uci, &board).is_ok() {
                            info!("Injected move: {uci}");
                        } else {
                            warn!(
                                "Injected move {uci} is not legal in the current position\n{}",
                                board.pretty(PrettyStyle::Utf8)
                            );
                        }
                    }
                    Action::RevertMove => {
                        let Some(m) = board.read().last_move() else {
                            continue;
                        };

                        move_builder.write().revert_move(m);

                        // board.write().revert_move();
                    }
                }
            }
        });
    }

    let (files, ranks) = match props.color {
        PlayerColor::White => (
            File::iter().collect::<Vec<_>>(),
            Rank::iter().collect::<Vec<_>>(),
        ),
        PlayerColor::Black => (
            File::iter().collect::<Vec<_>>().into_iter().rev().collect(),
            Rank::iter().collect::<Vec<_>>().into_iter().rev().collect(),
        ),
    };

    let is_promotion_required = move_builder.read().check_promotion().is_some();
    let class = if is_promotion_required {
        "opacity-25"
    } else {
        ""
    };

    rsx! {
        head::Link { rel: "stylesheet", href: CHESSBOARD_CLASSES }

        div {
            class: "relative",
            div {
                id: "chessboard",
                class,
                div {
                    class: "chessboard",
                    for r in ranks.iter().cloned() {
                        for f in files.iter().cloned() {
                            div {
                                id: format!("{f}{r}"),
                                onclick: move |_ev| {
                                    move_builder.write().put_square(f, r, &board.read());
                                },
                                Piece {
                                    coord: Coord::from_parts(f, r),
                                    color: props.color,
                                    pieces_set: props.pieces_set
                                }
                            }
                        }
                    }
                }
                Ranks { color: props.color }
                Files { color: props.color }
            }
            Promotion { color: props.color, pieces_set: props.pieces_set }
        }
    }
}

/// [Chessboard] properties.
#[derive(PartialEq, Props, Clone)]
pub struct ChessboardProps {
    /// Color the player plays for, i.e., pieces at the bottom.
    color: PlayerColor,
    /// Starting position in FEN notation.
    position: Option<String>,
    /// Pieces set.
    pieces_set: Option<PieceSet>,
    /// Transmitter channel of moves made on the board.
    uci_tx: Option<Coroutine<String>>,
}

#[derive(Debug)]
/// List of action [Chessboard] can receive via its client.
pub(crate) enum Action {
    MakeUciMove(String),
    RevertMove,
}

/// Complete properties with absent optional values of [ChessboardProps] filled with default values.
struct CompleteChessboardProps {
    color: PlayerColor,
    /// Starting position in FEN notation.
    position: String,
    pieces_set: PieceSet,
    uci_tx: Option<Coroutine<String>>,
}

impl ChessboardProps {
    fn complete(self) -> CompleteChessboardProps {
        CompleteChessboardProps {
            color: self.color,
            position: self
                .position
                .unwrap_or_else(|| Self::default_position().to_string()),
            pieces_set: self.pieces_set.unwrap_or(PieceSet::Standard),
            uci_tx: self.uci_tx,
        }
    }
    fn default_position() -> &'static str {
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    }
}

/// Color of the player's pieces.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum PlayerColor {
    White,
    Black,
}

impl PlayerColor {
    pub fn flip(&mut self) {
        match self {
            Self::White => *self = Self::Black,
            Self::Black => *self = Self::White,
        }
    }
}
