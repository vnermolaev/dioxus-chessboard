use crate::chessboard::action::{Action, ActionInner, PROCESSED_ACTION};
use crate::chessboard::properties::ChessboardProps;
use crate::files::Files;
use crate::historical_board::HistoricalBoard;
use crate::move_builder::MoveBuilder;
use crate::promotion::Promotion;
use crate::ranks::Ranks;
use crate::square::Square;
use dioxus::prelude::*;
use owlchess::board::PrettyStyle;
use owlchess::{Color, Coord, File, Rank};
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
            HistoricalBoard::initialize(&props.starting_position, props.san_tx)
                .expect("Valid FEN position description is expected"),
        )
    });

    // Initialize the move builder.
    use_context_provider(|| Signal::new(MoveBuilder::new()));

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
        debug!("Action {action:?} has already been processed");
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
            if let Some(m) = historical_board.read().last_move() {
                move_builder.write().revert_move(m);
            }
        }

        ActionInner::SetPosition { fen } => {
            let move_tx = historical_board.write().move_tx.take();

            historical_board.set(
                HistoricalBoard::initialize(&fen, move_tx)
                    .expect("Valid FEN position description is expected"),
            );
        }

        ActionInner::StepBack => {
            if let Some(m) = historical_board.read().get_previous_move() {
                move_builder.write().step_back(m);
            }
        }

        ActionInner::StepForward => {
            if let Some(m) = historical_board.read().get_next_move() {
                move_builder.write().step_forward(m);
            }
        }
        ActionInner::SetStartPosition => historical_board.write().set_start(),
        ActionInner::SetEndPosition => historical_board.write().set_end(),
    }
}
