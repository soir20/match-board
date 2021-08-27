use crate::position::Pos;
use std::ops::{BitAnd, BitOr, BitXor, Not};
use primitive_types::U256;
use std::fmt::{Display, Formatter};
use serde::{Serialize, Deserialize};

/// The size of a board as width by height.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub enum BoardSize {

    /* These are hard-coded so we know they fit in a 256-bit integer.
       (Width x height <= 256.) Validating provided dimensions may
       be confusing to users, so this enum makes the sizes explicit.
       They are as large as possible because extra space can simply be
       filled by walls. */
    EightByThirtyTwo,
    NineByTwentyEight,
    TenByTwentyFive,
    ElevenByTwentyThree,
    TwelveByTwentyOne,
    ThirteenByNineteen,
    FourteenByEighteen,
    FifteenBySeventeen,
    SixteenBySixteen

}

impl BoardSize {

    /// Gets the width of the board for this size.
    pub fn width(&self) -> u8 {
        match *self {
            BoardSize::EightByThirtyTwo => 8,
            BoardSize::NineByTwentyEight => 9,
            BoardSize::TenByTwentyFive => 10,
            BoardSize::ElevenByTwentyThree => 11,
            BoardSize::TwelveByTwentyOne => 12,
            BoardSize::ThirteenByNineteen => 13,
            BoardSize::FourteenByEighteen => 14,
            BoardSize::FifteenBySeventeen => 15,
            BoardSize::SixteenBySixteen => 16
        }
    }

    /// Gets the height of the board for this size.
    pub fn height(&self) -> u8 {
        match *self {
            BoardSize::EightByThirtyTwo => 32,
            BoardSize::NineByTwentyEight => 28,
            BoardSize::TenByTwentyFive => 25,
            BoardSize::ElevenByTwentyThree => 23,
            BoardSize::TwelveByTwentyOne => 21,
            BoardSize::ThirteenByNineteen => 19,
            BoardSize::FourteenByEighteen => 18,
            BoardSize::FifteenBySeventeen => 17,
            BoardSize::SixteenBySixteen => 16
        }
    }

}

impl Display for BoardSize {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width(), self.height())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Serialize, Deserialize)]
pub(crate) struct BitBoard {
    board: U256,
    width: u8,
    height: u8
}

/// A bitboard represents game state in binary. Board operations copy the board
/// with the new state.
impl BitBoard {

    /// Creates a new bitboard with a given size.
    ///
    /// # Arguments
    ///
    /// * `size` - the size of the bitboard to create
    pub fn new(size: BoardSize) -> BitBoard {
        BitBoard {
            board: U256::from(0),
            width: size.width(),
            height: size.height()
        }
    }

    /// Checks if a coordinate is set in this bitboard.
    ///
    /// # Arguments
    ///
    /// * `pos` - the coordinate to check
    ///
    /// # Panics
    ///
    /// Panics if the position is outside the bitboard.
    pub fn is_set(&self, pos: Pos) -> bool {
        self.board.bit(self.bit_pos(pos))
    }

    /// Sets a coordinate in this bitboard.
    ///
    /// # Arguments
    ///
    /// * `pos` - the coordinate to set
    ///
    /// # Panics
    ///
    /// Panics if the position is outside the bitboard.
    pub fn set(&self, pos: Pos) -> BitBoard {
        self.change_board(self.board | (U256::one() << self.bit_pos(pos)))
    }

    /// Clears a coordinate in this bitboard.
    ///
    /// # Arguments
    ///
    /// * `pos` - the coordinate to unset
    ///
    /// # Panics
    ///
    /// Panics if the position is outside the bitboard.
    pub fn unset(&self, pos: Pos) -> BitBoard {
        self.change_board(self.board & !(U256::one() << self.bit_pos(pos)))
    }

    /// Swaps two coordinates in this bitboard.
    ///
    /// # Arguments
    ///
    /// * `pos` - the coordinate to set
    ///
    /// # Panics
    ///
    /// Panics if either position is outside the bitboard.
    pub fn swap(&self, first: Pos, second: Pos) -> BitBoard {
        let bit1: U256 = self.bit(first);
        let bit2: U256 = self.bit(second);

        let xor_single = bit1 ^ bit2;
        let xor_in_pos = (xor_single << self.bit_pos(first)) | (xor_single << self.bit_pos(second));

        self.change_board(self.board ^ xor_in_pos)
    }

    /// Creates a new board with the same width and height.
    ///
    /// # Arguments
    ///
    /// * `board` - the integer backing the new board to create
    fn change_board(&self, board: U256) -> BitBoard {
        BitBoard {
            board,
            width: self.width,
            height: self.height
        }
    }

    /// Converts a coordinate into the position of the corresponding bit.
    ///
    /// # Arguments
    ///
    /// * `pos` - the coordinate to convert
    ///
    /// # Panics
    ///
    /// Panics if the position is outside the bitboard.
    fn bit_pos(&self, pos: Pos) -> usize {
        if pos.x() >= self.width || pos.y() >= self.height {
            panic!("Attempted to access position outside the bitboard: {}. \
            \nPlease report this to https://github.com/soir20/swap-and-match-engine/issues!", pos);
        }

        usize::from(pos.x() * self.width + pos.y())
    }

    /// Gets the bit at a specific position as a 256-bit integer.
    ///
    /// # Arguments
    ///
    /// * `pos` - the position to convert
    ///
    /// # Panics
    ///
    /// Panics if the position is outside the bitboard.
    fn bit(&self, pos: Pos) -> U256 {
        match self.is_set(pos) {
            true => U256::one(),
            false => U256::zero()
        }
    }
}

impl BitAnd for BitBoard {
    type Output = BitBoard;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.change_board(self.board & rhs.board)
    }
}

impl BitOr for BitBoard {
    type Output = BitBoard;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.change_board(self.board | rhs.board)
    }
}

impl BitXor for BitBoard {
    type Output = BitBoard;

    fn bitxor(self, rhs: Self) -> Self::Output {
        self.change_board(self.board ^ rhs.board)
    }
}

impl Not for BitBoard {
    type Output = BitBoard;

    fn not(self) -> Self::Output {
        self.change_board(!self.board)
    }
}

impl Display for BitBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut str = String::new();
        for y in (0..self.height).rev() {
            for x in 0..self.width {
                str.push(match self.is_set(Pos::new(x, y)) {
                    true => '1',
                    false => '0'
                })
            }

            str.push('\n');
        }

        write!(f, "{}", str)
    }
}