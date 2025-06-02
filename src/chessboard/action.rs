use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;

/// Action counter to make every [`ActionInner`] unique, i.e., [`Action`].
pub static NEXT_ACTION: AtomicU32 = AtomicU32::new(0);

/// Keeps track of injected [`ActionInner`]'s have been processed.
/// At initialization, this value must be different from the one in [`NEXT_ACTION`].
pub static PROCESSED_ACTION: AtomicU32 = AtomicU32::new(1);

#[derive(Debug, Clone, PartialEq)]
pub struct Action {
    /// Value allowing to discriminate instances of this variant.
    pub(crate) discriminator: u32,
    pub(crate) action: ActionInner,
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

    pub fn set_start_position() -> Action {
        Self {
            discriminator: NEXT_ACTION.fetch_add(1, Relaxed),
            action: ActionInner::SetStartPosition,
        }
    }

    pub fn set_end_position() -> Action {
        Self {
            discriminator: NEXT_ACTION.fetch_add(1, Relaxed),
            action: ActionInner::SetEndPosition,
        }
    }

    pub fn prev() -> Action {
        Self {
            discriminator: NEXT_ACTION.fetch_add(1, Relaxed),
            action: ActionInner::StepBack,
        }
    }

    pub fn next() -> Action {
        Self {
            discriminator: NEXT_ACTION.fetch_add(1, Relaxed),
            action: ActionInner::StepForward,
        }
    }
}

/// List of actions the [`Chessboard`] can receive via its client.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ActionInner {
    MakeSanMove(String),
    RevertMove,
    SetPosition {
        /// String FEN representation of the position.
        fen: String,
    },
    StepBack,
    StepForward,
    SetStartPosition,
    SetEndPosition,
}
