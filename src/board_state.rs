use std::collections::{HashMap, VecDeque};
use crate::piece::{PieceType};
use crate::bitboard::BitBoard;
use crate::position::Pos;
use serde::{Serialize, Deserialize};

/// Holds the current position of the pieces on the [Board] and the pieces
/// marked for a match check. BoardState is separate from the [Board] because
/// the [Board] is not (de)serializable. Thus, you can save the game by
/// saving the board state.
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct BoardState {
    pub(crate) width: u8,
    pub(crate) height: u8,
    pub(crate) pieces: HashMap<PieceType, BitBoard>,
    pub(crate) empties: BitBoard,
    pub(crate) movable_directions: [BitBoard; 4],
    pub(crate) last_changed: VecDeque<Pos>
}

impl BoardState {

    /// Creates a default board state with a given size.
    ///
    /// All the pieces on the board are walls by default,
    /// and no pieces are marked for a match check.
    ///
    /// # Arguments
    ///
    /// * `width` - the horizontal size of the board to create
    /// * `height` - the vertical size of the board to create
    pub fn new(width: u8, height: u8) -> BoardState {
        BoardState {
            width,
            height,
            pieces: HashMap::new(),
            empties: BitBoard::new(width, height),
            movable_directions: [
                BitBoard::new(width, height),
                BitBoard::new(width, height),
                BitBoard::new(width, height),
                BitBoard::new(width, height)
            ],
            last_changed: VecDeque::new()
        }
    }

}