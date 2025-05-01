use crate::files::Files;
use crate::historical_board::HistoricalBoard;
use crate::move_builder::MoveBuilder;
use crate::promotion::Promotion;
use crate::ranks::Ranks;
use crate::square::Square;
use crate::PieceSet;
use dioxus::prelude::*;
use owlchess::board::PrettyStyle;
use owlchess::{Color, Coord, File, Rank};
use std::fmt::{Debug, Display};
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;
use tracing::{debug, info, warn};

const CHESSBOARD_STYLES: Asset = asset!("/public/css/chessboard.css");

/// Component rendering [Chessboard].
#[component]
pub fn Chessboard(props: ChessboardProps) -> Element {
    let props = props.complete();
    debug!("Rendering with properties: {props:#?}");

    // Initialize the move history.
    use_context_provider(|| {
        Signal::new(
            HistoricalBoard::from_fen(&props.starting_position)
                .expect("Valid FEN position description is expected"),
        )
    });

    // Initialize the move builder.
    use_context_provider(|| Signal::new(MoveBuilder::new(props.san_tx)));

    let mut historical_board = use_context::<Signal<HistoricalBoard>>();
    let mut move_builder = use_context::<Signal<MoveBuilder>>();

    // Compute if the board is interactive for the **player**.
    let is_interactive = {
        let side_to_move = historical_board.read().side_to_move();

        // Board is interactive if
        // - it is configured to be interactive, and
        // - either it is in the analysis mode,
        // - or
        //   - the next move is expected from the configured player.
        // (general interactivity)   (analysis mode)          (a right color piece move)
        props.is_interactive && (!props.single_player_mode || side_to_move == props.color)
    };

    if let Some(action) = props.action {
        // Board always accepts actions.
        maybe_update_board(action, &mut historical_board, &mut move_builder);
    }

    let (files, ranks) = match props.color {
        Color::White => (
            File::iter().collect::<Vec<_>>(),
            Rank::iter().collect::<Vec<_>>(),
        ),
        Color::Black => (
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
        document::Link { rel: "stylesheet", href: CHESSBOARD_STYLES }

        div { position: "relative",
            div { class: chessboard_classes.join(" "),
                for r in ranks.iter().cloned() {
                    div { class: "row",
                        for f in files.iter().cloned() {
                            Square {
                                is_interactive,
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
fn maybe_update_board(
    action: Action,
    historical_board: &mut Signal<HistoricalBoard>,
    move_builder: &mut Signal<MoveBuilder>,
) {
    let processed_action = PROCESSED_ACTION.load(Relaxed);
    if processed_action == action.discriminator {
        return;
    }
    PROCESSED_ACTION.store(action.discriminator, Relaxed);

    debug!("Received action: {action:?}");

    match action.action {
        ActionInner::MakeSanMove(san) => {
            let board = historical_board.read();

            if move_builder.write().apply_san_move(&san, &board).is_ok() {
                info!("Injected move: {san}");
            } else {
                warn!(
                    "Injected move {san} is not legal in the current position\n{}",
                    board.pretty(PrettyStyle::Utf8)
                );
            }
        }
        ActionInner::RevertMove => {
            let board = historical_board.read();
            if let Some(m) = board.last_move() {
                move_builder.write().revert_move(m);
            }
        }
        ActionInner::SetPosition { fen } => {
            *historical_board.write() = HistoricalBoard::from_fen(&fen)
                .expect("Valid FEN position description is expected");
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
    /// [`Color`] the player plays for, i.e., pieces at the bottom.
    player_color: Color,

    /// In single player mode, the player will only be able to move pieces of the `player_color`.
    /// Otherwise, the board allows all moves.
    single_player_mode: Option<bool>,

    /// The starting position in FEN notation.
    ///
    /// **IMPORTANT:** This value sets only the initial position.
    /// The chessboard component will not update if the user changes this starting position,
    /// because it initializes an internal state that remains immutable with respect to property changes.
    /// To update the position of an existing component, use [`Action::set_position`].
    starting_position: Option<String>,
    /// Pieces set.
    pieces_set: Option<PieceSet>,
    /// Injected action.
    action: Option<Action>,
    /// Transmitter channel of moves made on the board.
    san_tx: Option<Coroutine<SanMove>>,
}

impl ChessboardProps {
    fn complete(self) -> CompleteChessboardProps {
        CompleteChessboardProps {
            is_interactive: self.is_interactive.unwrap_or(true),
            color: self.player_color,
            // By default, allow exploration mode.
            single_player_mode: self.single_player_mode.unwrap_or_default(),
            starting_position: self
                .starting_position
                .unwrap_or_else(|| Self::default_position().to_string()),
            pieces_set: self.pieces_set.unwrap_or(PieceSet::Standard),
            action: self.action,
            san_tx: self.san_tx,
        }
    }
    pub fn default_position() -> &'static str {
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    }
}

/// SAN-encoded chess move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanMove {
    pub san_repr: String,
    pub piece: owlchess::Piece,
    pub color: Color,
}

impl SanMove {
    pub fn new(san_repr: &str, piece: owlchess::Piece, color: Color) -> Self {
        Self {
            san_repr: san_repr.to_string(),
            piece,
            color,
        }
    }
}

impl Display for SanMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Action counter to make every [ActionInner] unique, i.e., [Action].
static NEXT_ACTION: AtomicU32 = AtomicU32::new(0);

/// Keeps track which injected [ActionInner]'s have been processed.
/// At initialization, this value must be different from the one in [NEXT_ACTION].
static PROCESSED_ACTION: AtomicU32 = AtomicU32::new(1);

#[derive(Debug, Clone, PartialEq)]
pub struct Action {
    /// Value allowing to discriminate instances of this variant.
    discriminator: u32,
    action: ActionInner,
}

impl Action {
    /// Make a SAN-encoded move.
    pub fn make_move(m: &str) -> Self {
        Self {
            discriminator: NEXT_ACTION.fetch_add(1, Relaxed),
            action: ActionInner::MakeSanMove(m.to_string()),
        }
    }

    pub fn revert_move() -> Action {
        Self {
            discriminator: NEXT_ACTION.fetch_add(1, Relaxed),
            action: ActionInner::RevertMove,
        }
    }

    pub fn set_position(fen: &str) -> Action {
        Self {
            discriminator: NEXT_ACTION.fetch_add(1, Relaxed),
            action: ActionInner::SetPosition {
                fen: fen.to_string(),
            },
        }
    }

    /// Returns true if action requires making a move.
    pub fn is_move(&self) -> bool {
        matches!(
            self.action,
            ActionInner::MakeSanMove(_) | ActionInner::RevertMove
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
/// List of action [Chessboard] can receive via its client.
pub(crate) enum ActionInner {
    MakeSanMove(String),
    RevertMove,
    SetPosition {
        /// String FEN representation of the position.
        fen: String,
    },
}

/// Complete properties with absent optional values of [ChessboardProps] filled with default values.
struct CompleteChessboardProps {
    is_interactive: bool,
    color: Color,
    single_player_mode: bool,
    /// Starting position in FEN notation.
    starting_position: String,
    pieces_set: PieceSet,
    action: Option<Action>,
    san_tx: Option<Coroutine<SanMove>>,
}

impl Debug for CompleteChessboardProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompleteChessboardProps")
            .field("is_interactive", &self.is_interactive)
            .field("color", &self.color)
            .field("single_player_mode", &self.single_player_mode)
            .field("starting position", &self.starting_position)
            .field("pieces_set", &self.pieces_set)
            .field("action", &self.action)
            .finish()
    }
}
