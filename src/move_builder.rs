use dioxus::hooks::Coroutine;
use owlchess::board::PrettyStyle;
use owlchess::moves::{uci, PromotePiece, Style};
use owlchess::{Board, Color, Coord, File, Move, MoveKind, Piece, Rank};
use std::ops::{Deref, DerefMut};
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
pub struct MoveBuilder {
    /// When the move reaches [State::ManualFinal],
    /// the corresponding uci will be sent out.
    uci_move_tx: Option<Coroutine<String>>,
    /// [State] of the builder.
    state: State,
}

impl Deref for MoveBuilder {
    type Target = State;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
impl DerefMut for MoveBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl MoveBuilder {
    pub fn new(uci_move_tx: Option<Coroutine<String>>) -> Self {
        Self {
            uci_move_tx,
            state: State::new(),
        }
    }

    pub fn src(&self) -> Option<Coord> {
        self.deref().src()
    }

    #[allow(dead_code)] // Maybe used in the future.
    pub fn dst(&self) -> Option<Coord> {
        self.deref().dst()
    }

    /// Puts a square into [State].
    pub fn put_square(&mut self, file: File, rank: Rank, board: &Board) {
        self.deref_mut().put_square(file, rank, board)
    }

    pub fn put_uci_move(&mut self, uci: &str, board: &Board) -> Result<(), uci::ParseError> {
        self.deref_mut().put_uci_move(uci, board)
    }

    pub fn check_promotion(&self) -> Option<(Coord, Coord)> {
        self.deref().check_promotion()
    }

    pub fn promote(&mut self, piece: PromotePiece, board: &Board) {
        self.deref_mut().promote(piece, board)
    }

    pub fn animations(&self) -> Vec<(Coord, Coord)> {
        self.deref().animations()
    }

    pub fn finalize(&mut self, board: &Board) -> Option<Move> {
        let m = self.deref_mut().finalize();

        if let (Some(ref m), Some(ref uci_move_tx)) = (m, self.uci_move_tx) {
            // There is a valid move and a coroutine to report it.
            let uci = m
                .styled(board, Style::Uci)
                .expect("Move must be correctly finalized")
                .to_string();
            uci_move_tx.send(uci);
        }

        m
    }
}

#[derive(Debug)]
pub enum State {
    None,
    Src(Coord),
    RegularMove {
        m: Move,
        /// Supporting animation.
        support: Vec<(Coord, Coord)>,
    },
    Promotion(Promotion),
    Final(Final),
}

#[derive(Debug)]
pub enum Promotion {
    PrePromotion {
        src: Coord,
        dst: Coord,
        // Supporting animation is just (src, dst).
    },
    Promotion {
        src: Coord,
        dst: Coord,
    },
}

impl Promotion {
    fn src(&self) -> Coord {
        match self {
            Self::PrePromotion { src, .. } => *src,
            Self::Promotion { src, .. } => *src,
        }
    }

    fn dst(&self) -> Coord {
        match self {
            Self::PrePromotion { dst, .. } => *dst,
            Self::Promotion { dst, .. } => *dst,
        }
    }

    fn animations(&self) -> Vec<(Coord, Coord)> {
        match self {
            Self::PrePromotion { src, dst } => vec![(*src, *dst)],
            _ => vec![],
        }
    }
}

#[derive(Debug)]
pub enum Final {
    Manual(Move),
    Automatic(Move),
}

impl Final {
    fn src(&self) -> Coord {
        match self {
            Self::Manual(m) => m.src(),
            Self::Automatic(m) => m.src(),
        }
    }

    fn dst(&self) -> Coord {
        match self {
            Self::Manual(m) => m.dst(),
            Self::Automatic(m) => m.dst(),
        }
    }

    fn animations(&self) -> Vec<(Coord, Coord)> {
        match self {
            Self::Automatic(m) => vec![(m.src(), m.dst())],
            _ => vec![],
        }
    }

    fn get_move(&self) -> Move {
        match self {
            Self::Manual(m) => *m,
            Self::Automatic(m) => *m,
        }
    }
}

impl State {
    fn new() -> Self {
        Self::None
    }

    fn src(&self) -> Option<Coord> {
        match self {
            Self::Src(src) => Some(*src),
            Self::RegularMove { m, .. } => Some(m.src()),
            Self::Promotion(manual) => Some(manual.src()),
            Self::Final(m) => Some(m.src()),
            _ => None,
        }
    }

    #[allow(dead_code)] // Maybe used in the future.
    fn dst(&self) -> Option<Coord> {
        match self {
            Self::RegularMove { m, .. } => Some(m.dst()),
            Self::Promotion(manual) => Some(manual.dst()),
            Self::Final(m) => Some(m.dst()),
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
    fn put_square(&mut self, file: File, rank: Rank, board: &Board) {
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

                            Self::RegularMove { m, support }
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

    fn put_uci_move(&mut self, uci: &str, board: &Board) -> Result<(), uci::ParseError> {
        *self = Self::Final(Final::Automatic(Move::from_uci_legal(uci, board)?));

        Ok(())
    }

    fn check_promotion(&self) -> Option<(Coord, Coord)> {
        match self {
            Self::Promotion(manual @ Promotion::Promotion { .. }) => {
                Some((manual.src(), manual.dst()))
            }
            _ => None,
        }
    }

    fn promote(&mut self, piece: PromotePiece, board: &Board) {
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
                    Ok(m) => Self::Final(Final::Manual(m)),
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

    fn animations(&self) -> Vec<(Coord, Coord)> {
        match self {
            Self::RegularMove { support, .. } => support.clone(),
            Self::Promotion(manual) => manual.animations(),
            Self::Final(final_move) => final_move.animations(),
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
            Self::Final(final_move) => {
                let m = final_move.get_move();
                *self = Self::None;
                Some(m)
            }
            Self::RegularMove { m, .. } => {
                let m = *m;
                *self = Self::None;
                Some(m)
            }
            _ => None,
        }
    }
}
