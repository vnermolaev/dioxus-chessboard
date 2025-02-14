use crate::files::Files;
use crate::historical_board::HistoricalBoard;
use crate::move_builder::MoveBuilder;
use crate::promotion::Promotion;
use crate::ranks::Ranks;
use crate::square::Square;
use crate::PieceSet;
use dioxus::prelude::*;
use owlchess::board::PrettyStyle;
use owlchess::{Coord, File, Rank};
use std::fmt::{Debug, Formatter};
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;
use tracing::{debug, info, warn};

/// Classes to render the chessboard.
const CHESSBOARD_CLASSES: Asset = asset!("public/chessboard.css");
// Tailwind classes.
const TAILWIND_CLASSES: Asset = asset!("public/tailwind.css");

/// Component rendering [Chessboard].
#[component]
pub fn Chessboard(props: ChessboardProps) -> Element {
    let props = props.complete();
    debug!("{props:?}");

    use_context_provider(|| {
        Signal::new(
            HistoricalBoard::from_fen(&props.position)
                .expect("Board must be constructible from a valid position"),
        )
    });

    use_context_provider(|| Signal::new(MoveBuilder::new(props.uci_tx)));

    let historical_board = use_context::<Signal<HistoricalBoard>>();
    let mut move_builder = use_context::<Signal<MoveBuilder>>();

    if let Some(action) = props.action {
        maybe_update_board(
            action,
            props.is_interactive,
            &historical_board,
            &mut move_builder,
        );
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

    let mut chessboard_classes = vec!["chessboard"];

    if move_builder.read().check_promotion().is_some() {
        // Promotion is required.
        chessboard_classes.push("opacity-25");
    }

    rsx! {
        document::Link { rel: "stylesheet", href: CHESSBOARD_CLASSES }
        document::Link { rel: "stylesheet", href: TAILWIND_CLASSES }

        div { class: "relative",
            div { class: chessboard_classes.join(" "),
                for r in ranks.iter().cloned() {
                    div { class: "row",
                        for f in files.iter().cloned() {
                            Square {
                                is_interactive: props.is_interactive,
                                coord: Coord::from_parts(f, r),
                                color: props.color,
                                pieces_set: props.pieces_set,
                            }
                        }
                    }
                }
            }
            Ranks { color: props.color }
            Files { color: props.color }
            Promotion { color: props.color, pieces_set: props.pieces_set }
        }
    }
}

/// Examine [Action] and apply respective changes if the action has not yet been processed.
/// If the action was processed, does nothing.
/// If the board is not interactive, mark the action as processed.
fn maybe_update_board(
    action: Action,
    is_interactive: bool,
    historical_board: &Signal<HistoricalBoard>,
    move_builder: &mut Signal<MoveBuilder>,
) {
    let processed_action = PROCESSED_ACTION.load(Relaxed);
    if processed_action == action.discriminator {
        return;
    }
    PROCESSED_ACTION.store(action.discriminator, Relaxed);

    if !is_interactive {
        debug!("Chessboard is not interactive. Ignoring the request...");
        return;
    }

    debug!("Applying action: {action:?}");

    let board = historical_board.read();

    match action.action {
        ActionInner::MakeUciMove(uci) => {
            if move_builder.write().apply_uci_move(&uci, &board).is_ok() {
                info!("Injected move: {uci}");
            } else {
                warn!(
                    "Injected move {uci} is not legal in the current position\n{}",
                    board.pretty(PrettyStyle::Utf8)
                );
            }
        }
        ActionInner::RevertMove => {
            if let Some(m) = board.last_move() {
                move_builder.write().revert_move(m);
            }
        }
    }
}

/// [Chessboard] properties.
#[derive(PartialEq, Props, Clone)]
pub struct ChessboardProps {
    /// Is the board interactive?
    /// If you only need to display a position, set this to false.
    /// By default, the board will be interactive.
    is_interactive: Option<bool>,
    /// Color the player plays for, i.e., pieces at the bottom.
    color: PlayerColor,
    /// Starting position in FEN notation.
    position: Option<String>,
    /// Pieces set.
    pieces_set: Option<PieceSet>,
    /// Injected action.
    action: Option<Action>,
    /// Transmitter channel of moves made on the board.
    uci_tx: Option<Coroutine<String>>,
}

/// Action counter to make every [ActionInner] unique, i.e., [UniqueAction].
static NEXT_ACTION: AtomicU32 = AtomicU32::new(0);

/// Keeps track which injected [UniqueAction]'s have been processed.
/// At initialization, this value must be different from the one in [NEXT_ACTION].
static PROCESSED_ACTION: AtomicU32 = AtomicU32::new(1);

#[derive(Debug, Clone, PartialEq)]
pub struct Action {
    /// Value allowing to discriminate instances of this variant.
    discriminator: u32,
    action: ActionInner,
}

impl Action {
    pub fn make_move(m: &str) -> Self {
        Self {
            discriminator: NEXT_ACTION.fetch_add(1, Relaxed),
            action: ActionInner::MakeUciMove(m.to_string()),
        }
    }

    pub fn revert_move() -> Action {
        Self {
            discriminator: NEXT_ACTION.fetch_add(1, Relaxed),
            action: ActionInner::RevertMove,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// List of action [Chessboard] can receive via its client.
pub(crate) enum ActionInner {
    MakeUciMove(String),
    RevertMove,
}

/// Complete properties with absent optional values of [ChessboardProps] filled with default values.
struct CompleteChessboardProps {
    is_interactive: bool,
    color: PlayerColor,
    /// Starting position in FEN notation.
    position: String,
    pieces_set: PieceSet,
    action: Option<Action>,
    uci_tx: Option<Coroutine<String>>,
}

impl Debug for CompleteChessboardProps {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompleteChessboardProps")
            .field("is_interactive", &self.is_interactive)
            .field("color", &self.color)
            .field("position", &self.position)
            .field("pieces_set", &self.pieces_set)
            .field("action", &self.action)
            .finish()
    }
}

impl ChessboardProps {
    fn complete(self) -> CompleteChessboardProps {
        CompleteChessboardProps {
            is_interactive: self.is_interactive.unwrap_or(true),
            color: self.color,
            position: self
                .position
                .unwrap_or_else(|| Self::default_position().to_string()),
            pieces_set: self.pieces_set.unwrap_or(PieceSet::Standard),
            action: self.action,
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
