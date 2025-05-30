use owlchess::Move;

mod applicable_move;
mod promotion;
mod state;
pub use state::MoveBuilder;

#[derive(Clone, Copy)]
pub enum MoveAction {
    None,
    /// A game mode action.
    /// Apply a [`Move`] to the [`HistoricalBoard`].
    Apply(Move),
    /// A game mode action.
    /// Revert the last [`Move`] known to the [`HistoricalBoard`].
    Revert,
    /// An analysis mode action.
    /// Set the _previous_ move on the [`HistoricalBoard`], if any.
    StepBack,
    /// An analysis mode action.
    /// Set the _next_ move on the [`HistoricalBoard`], if any.
    StepForward,
}
