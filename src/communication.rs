use crate::chessboard::Action;
use std::sync::{Mutex, OnceLock};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

#[derive(Clone)]
pub struct ChessboardClient;

impl ChessboardClient {
    fn send(action: Action) {
        initialize().0.send(action).expect("Must send");
    }

    pub fn make_move(uci: &str) {
        Self::send(Action::MakeUciMove(uci.to_string()));
    }

    pub fn revert_move() {
        Self::send(Action::RevertMove);
    }
}

type Communication = (
    UnboundedSender<Action>,
    Mutex<Option<UnboundedReceiver<Action>>>,
);

static COMMUNICATION: OnceLock<Communication> = OnceLock::new();

/// Builds [COMMUNICATION] channels.
fn initialize() -> &'static Communication {
    COMMUNICATION.get_or_init(|| {
        let (tx, rx) = unbounded_channel();
        (tx, Mutex::new(Some(rx)))
    })
}

pub(crate) fn get_chessboard_receiver() -> Option<UnboundedReceiver<Action>> {
    initialize().1.lock().expect("Must lock").take()
}
