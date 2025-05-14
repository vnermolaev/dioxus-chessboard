use owlchess::Move;

mod applicable_move;
mod promotion;
mod state;
pub use state::MoveBuilder;

#[derive(Clone, Copy)]
pub enum MoveAction {
    None,
    Apply(Move),
    Revert,
}
