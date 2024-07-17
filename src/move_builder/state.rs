use crate::move_builder::applicable_move::ApplicableMove;
use crate::move_builder::promotion::Promotion;
use owlchess::board::PrettyStyle;
use owlchess::moves::{uci, PromotePiece};
use owlchess::{Board, Color, Coord, File, Move, MoveKind, Piece, Rank};
use tracing::{debug, warn};

/// Builder for [Move] structured as a [State] machine:
///
///                                               None <-----------------------------------------------------------+
///                                                ↓                                                               |
///                                          (Square selected)                                                     |
///                                                ↓                                                               |
///                 +---------------------------- Src ----------------------------------+                          |
///                 ↓                              ↓                                    ↓                          |
/// /     Valid square selected      \   /        Valid square selected      \   (Invalid square selected) --------+
/// \ Promotion info IS NOT required /   \ Promotion information IS required /                                     |
///                 ↓                                      ↓                                                       |
///          (Animation runs)                       (Animation runs)                                               |
///                 |                                      ↓                                                       |
///                 |                                 PromoRequired -------------------+                           |
///                 |                                      ↓                           ↓                           |
///                 |                        (Valid promotion info submitted)  (Invalid promotion info submitted) -+
///                 |                                      |                                                       |
///                 +-------------> FinalMove <------------+                                                       |
///                                     ↓                                                                          |
///                                (Finalize) --> (Move to be applied)                                             |
///                                     +--------------------------------------------------------------------------+

#[derive(Debug)]
pub enum State {
    None,
    Src(Coord),
    Move {
        m: Move,
        /// Supporting animation.
        support: Vec<(Coord, Coord)>,
    },
    Promotion(Promotion),
    ApplicableMove(ApplicableMove),
}

impl State {
    pub(crate) fn new() -> Self {
        Self::None
    }

    pub(crate) fn src(&self) -> Option<Coord> {
        match self {
            Self::Src(src) => Some(*src),
            Self::Move { m, .. } => Some(m.src()),
            Self::Promotion(manual) => Some(manual.src()),
            Self::ApplicableMove(m) => Some(m.src()),
            _ => None,
        }
    }

    #[allow(dead_code)] // Maybe used in the future.
    pub(crate) fn dst(&self) -> Option<Coord> {
        match self {
            Self::Move { m, .. } => Some(m.dst()),
            Self::Promotion(manual) => Some(manual.dst()),
            Self::ApplicableMove(m) => Some(m.dst()),
            _ => None,
        }
    }

    /// Promotion info is required when:
    /// - A White pawn reaches the 8th rank,
    /// - A Black pawn reaches the 1st rank.
    fn is_promotion_required(src: &Coord, dst: &Coord, board: &Board) -> bool {
        let cell = board.get(*src);

        matches!(
            (cell.color(), cell.piece(), dst.rank()),
            (Some(Color::White), Some(Piece::Pawn), Rank::R8)
                | (Some(Color::Black), Some(Piece::Pawn), Rank::R1)
        )
    }

    /// Puts a square into [State].
    pub(crate) fn put_square(&mut self, file: File, rank: Rank, board: &Board) {
        let coord = Coord::from_parts(file, rank);

        *self = match self {
            // Start building a move by selecting a piece.
            Self::None
                if board
                    .get(coord)
                    .color()
                    .map(|c| c == board.raw().side)
                    .unwrap_or_default() =>
            {
                // Selecting a piece of the right color to move.
                Self::Src(coord)
            }
            // Selecting the original source, cancels the move,
            // selecting a different square, sets the destination square.
            Self::Src(src) => {
                if *src == coord {
                    Self::None
                } else {
                    let src = *src;
                    let dst = coord;
                    // To validate a move, a promotion piece might be required if
                    // a pawn is being promoted.
                    let is_promo_required = Self::is_promotion_required(&src, &dst, board);
                    // Construct a hypothetical promotion piece,
                    // if the promotion information is required.
                    let hypothetical_promotion_piece = if is_promo_required { "q" } else { "" };

                    // Converting the move a UCI string is a shortcut
                    // enabling me to avoid dealing with move kind and
                    // let the engine figure it out by itself.
                    let uci = format!("{src}{dst}{hypothetical_promotion_piece}");
                    debug!(
                        "Testing hypothetical move: {uci}\n{}",
                        board.pretty(PrettyStyle::Utf8)
                    );

                    // Verify the legality of the move.
                    match Move::from_uci_legal(&uci, board) {
                        Ok(m) if !is_promo_required => {
                            fn coord(f: File, r: Rank) -> Coord {
                                Coord::from_parts(f, r)
                            }

                            // Compute supporting animations.
                            let support = match m.kind() {
                                MoveKind::CastlingKingside => {
                                    // King steps from the E to G file.
                                    // Rook steps from the H to F file.
                                    let rank = m.src().rank();
                                    vec![
                                        (coord(File::E, rank), coord(File::G, rank)),
                                        (coord(File::H, rank), coord(File::F, rank)),
                                    ]
                                }
                                MoveKind::CastlingQueenside => {
                                    // King steps from the E to C file.
                                    // Rook steps from the A to D file.
                                    let rank = m.src().rank();
                                    vec![
                                        (coord(File::E, rank), coord(File::C, rank)),
                                        (coord(File::A, rank), coord(File::D, rank)),
                                    ]
                                }
                                _ => vec![(m.src(), m.dst())],
                            };

                            Self::Move { m, support }
                        }
                        Ok(_) => Self::Promotion(Promotion::PrePromotion { src, dst }),
                        Err(_) => {
                            warn!("Illegal move. Cancelling the move");
                            Self::None
                        }
                    }
                }
            }
            _ => {
                debug!("Reverting the move builder to the initial state");
                Self::None
            }
        }
    }

    pub(crate) fn put_uci_move(&mut self, uci: &str, board: &Board) -> Result<(), uci::ParseError> {
        *self = Self::ApplicableMove(ApplicableMove::Automatic(Move::from_uci_legal(uci, board)?));

        Ok(())
    }

    pub(crate) fn check_promotion(&self) -> Option<(Coord, Coord)> {
        match self {
            Self::Promotion(promotion @ Promotion::Promotion { .. }) => {
                Some((promotion.src(), promotion.dst()))
            }
            _ => None,
        }
    }

    pub(crate) fn promote(&mut self, piece: PromotePiece, board: &Board) {
        *self = match self {
            Self::Promotion(Promotion::Promotion { src, dst }) => {
                // Converting the move a UCI string is a shortcut
                // enabling me to avoid dealing with move kind and
                // let the engine figure it out by itself.
                let promotion_piece = match piece {
                    PromotePiece::Knight => "n",
                    PromotePiece::Rook => "r",
                    PromotePiece::Bishop => "b",
                    PromotePiece::Queen => "q",
                };
                let uci = format!("{src}{dst}{promotion_piece}");
                debug!(
                    "Testing promotion move: {uci}\n{}",
                    board.pretty(PrettyStyle::Utf8)
                );

                let m = Move::from_uci_legal(&uci, board);
                debug!("Uci {uci}; build result {m:?}");

                match m {
                    Ok(m) => Self::ApplicableMove(ApplicableMove::Manual(m)),
                    Err(_) => {
                        warn!("Illegal promotion, cancelling the move");
                        Self::None
                    }
                }
            }
            _ => {
                warn!("Unexpected promotion of {self:?}. Cancelling the move");
                Self::None
            }
        }
    }

    pub(crate) fn animations(&self) -> Vec<(Coord, Coord)> {
        match self {
            Self::Move { support, .. } => support.clone(),
            Self::Promotion(promotion) => promotion.animations(),
            Self::ApplicableMove(applicable_move) => applicable_move.animations(),
            _ => vec![],
        }
    }

    pub(crate) fn finalize(&mut self) -> Option<Move> {
        match self {
            Self::Promotion(Promotion::PrePromotion { src, dst }) => {
                *self = Self::Promotion(Promotion::Promotion {
                    src: *src,
                    dst: *dst,
                });
                None
            }
            Self::ApplicableMove(final_move) => {
                let m = final_move.get_move();
                *self = Self::None;
                Some(m)
            }
            Self::Move { m, .. } => {
                let m = *m;
                *self = Self::None;
                Some(m)
            }
            _ => None,
        }
    }
}
