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

/// Efficiently maintains the state of a board with bits.
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

#[cfg(test)]
mod tests {
    use crate::bitboard::{BoardSize, BitBoard};
    use crate::position::Pos;

    #[test]
    fn size_width_gets_width() {
        assert_eq!(8, BoardSize::EightByThirtyTwo.width());
    }

    #[test]
    fn size_height_gets_height() {
        assert_eq!(32, BoardSize::EightByThirtyTwo.height());
    }

    #[test]
    fn size_display_has_width_by_height() {
        assert_eq!("8x32", format!("{}", BoardSize::EightByThirtyTwo));
    }

    #[test]
    #[should_panic]
    fn bitboard_is_set_out_of_bounds_x_panics() {
        assert!(BitBoard::new(BoardSize::FifteenBySeventeen).is_set(Pos::new(15, 5)));
    }

    #[test]
    #[should_panic]
    fn bitboard_is_set_out_of_bounds_y_panics() {
        assert!(BitBoard::new(BoardSize::FifteenBySeventeen).is_set(Pos::new(5, 17)));
    }

    #[test]
    fn bitboard_set_previously_unset() {
        let pos = Pos::new(1, 3);
        assert!(BitBoard::new(BoardSize::SixteenBySixteen).set(pos).is_set(pos));
    }

    #[test]
    fn bitboard_set_previously_set() {
        let pos = Pos::new(1, 3);
        assert!(BitBoard::new(BoardSize::SixteenBySixteen).set(pos).set(pos).is_set(pos));
    }

    #[test]
    #[should_panic]
    fn bitboard_set_out_of_bounds_x_panics() {
        let pos = Pos::new(16, 5);
        BitBoard::new(BoardSize::SixteenBySixteen).set(pos);
    }

    #[test]
    #[should_panic]
    fn bitboard_set_out_of_bounds_y_panics() {
        let pos = Pos::new(5, 16);
        BitBoard::new(BoardSize::SixteenBySixteen).set(pos);
    }

    #[test]
    fn bitboard_unset_previously_unset() {
        let pos = Pos::new(1, 3);
        assert!(!BitBoard::new(BoardSize::SixteenBySixteen).unset(pos).is_set(pos));
    }

    #[test]
    fn bitboard_unset_previously_set() {
        let pos = Pos::new(1, 3);
        assert!(!BitBoard::new(BoardSize::SixteenBySixteen).set(pos).unset(pos).is_set(pos));
    }

    #[test]
    #[should_panic]
    fn bitboard_unset_out_of_bounds_x_panics() {
        let pos = Pos::new(15, 5);
        BitBoard::new(BoardSize::FifteenBySeventeen).unset(pos);
    }

    #[test]
    #[should_panic]
    fn bitboard_unset_out_of_bounds_y_panics() {
        let pos = Pos::new(5, 17);
        BitBoard::new(BoardSize::FifteenBySeventeen).unset(pos);
    }

    #[test]
    fn bitboard_swap_both_unset() {
        let pos1 = Pos::new(1, 3);
        let pos2 = Pos::new(0, 5);
        let board = BitBoard::new(BoardSize::SixteenBySixteen).swap(pos1, pos2);
        assert!(!board.is_set(pos1));
        assert!(!board.is_set(pos2));
    }

    #[test]
    fn bitboard_swap_both_set() {
        let pos1 = Pos::new(1, 3);
        let pos2 = Pos::new(0, 5);
        let board = BitBoard::new(BoardSize::SixteenBySixteen)
            .set(pos1)
            .set(pos2)
            .swap(pos1, pos2);
        assert!(board.is_set(pos1));
        assert!(board.is_set(pos2));
    }

    #[test]
    fn bitboard_swap_first_set() {
        let pos1 = Pos::new(1, 3);
        let pos2 = Pos::new(0, 5);
        let board = BitBoard::new(BoardSize::SixteenBySixteen)
            .set(pos1)
            .swap(pos1, pos2);
        assert!(!board.is_set(pos1));
        assert!(board.is_set(pos2));
    }

    #[test]
    fn bitboard_swap_second_set() {
        let pos1 = Pos::new(1, 3);
        let pos2 = Pos::new(0, 5);
        let board = BitBoard::new(BoardSize::SixteenBySixteen)
            .set(pos2)
            .swap(pos1, pos2);
        assert!(board.is_set(pos1));
        assert!(!board.is_set(pos2));
    }

    #[test]
    #[should_panic]
    fn bitboard_swap_out_of_bounds_first_x_panics() {
        let pos1 = Pos::new(15, 3);
        let pos2 = Pos::new(0, 5);
        BitBoard::new(BoardSize::FifteenBySeventeen).swap(pos1, pos2);
    }

    #[test]
    #[should_panic]
    fn bitboard_swap_out_of_bounds_first_y_panics() {
        let pos1 = Pos::new(1, 17);
        let pos2 = Pos::new(0, 5);
        BitBoard::new(BoardSize::FifteenBySeventeen).swap(pos1, pos2);
    }

    #[test]
    #[should_panic]
    fn bitboard_swap_out_of_bounds_second_x_panics() {
        let pos1 = Pos::new(1, 3);
        let pos2 = Pos::new(15, 5);
        BitBoard::new(BoardSize::FifteenBySeventeen).swap(pos1, pos2);
    }

    #[test]
    #[should_panic]
    fn bitboard_swap_out_of_bounds_second_y_panics() {
        let pos1 = Pos::new(1, 3);
        let pos2 = Pos::new(0, 17);
        BitBoard::new(BoardSize::FifteenBySeventeen).swap(pos1, pos2);
    }

    #[test]
    fn bitboard_and_both_unset() {
        let pos = Pos::new(1, 3);
        let board1 = BitBoard::new(BoardSize::SixteenBySixteen);
        let board2 = board1;
        assert!(!(board1 & board2).is_set(pos));
    }

    #[test]
    fn bitboard_and_both_set() {
        let pos = Pos::new(1, 3);
        let board1 = BitBoard::new(BoardSize::SixteenBySixteen).set(pos);
        let board2 = board1;
        assert!((board1 & board2).is_set(pos));
    }

    #[test]
    fn bitboard_and_first_set() {
        let pos = Pos::new(1, 3);
        let board1 = BitBoard::new(BoardSize::SixteenBySixteen).set(pos);
        let board2 = BitBoard::new(BoardSize::SixteenBySixteen);
        assert!(!(board1 & board2).is_set(pos));
    }

    #[test]
    fn bitboard_and_second_set() {
        let pos = Pos::new(1, 3);
        let board1 = BitBoard::new(BoardSize::SixteenBySixteen);
        let board2 = BitBoard::new(BoardSize::SixteenBySixteen).set(pos);
        assert!(!(board1 & board2).is_set(pos));
    }

    #[test]
    fn bitboard_or_both_unset() {
        let pos = Pos::new(1, 3);
        let board1 = BitBoard::new(BoardSize::SixteenBySixteen);
        let board2 = board1;
        assert!(!(board1 | board2).is_set(pos));
    }

    #[test]
    fn bitboard_or_both_set() {
        let pos = Pos::new(1, 3);
        let board1 = BitBoard::new(BoardSize::SixteenBySixteen).set(pos);
        let board2 = board1;
        assert!((board1 | board2).is_set(pos));
    }

    #[test]
    fn bitboard_or_first_set() {
        let pos = Pos::new(1, 3);
        let board1 = BitBoard::new(BoardSize::SixteenBySixteen).set(pos);
        let board2 = BitBoard::new(BoardSize::SixteenBySixteen);
        assert!((board1 | board2).is_set(pos));
    }

    #[test]
    fn bitboard_or_second_set() {
        let pos = Pos::new(1, 3);
        let board1 = BitBoard::new(BoardSize::SixteenBySixteen);
        let board2 = BitBoard::new(BoardSize::SixteenBySixteen).set(pos);
        assert!((board1 | board2).is_set(pos));
    }

    #[test]
    fn bitboard_xor_both_unset() {
        let pos = Pos::new(1, 3);
        let board1 = BitBoard::new(BoardSize::SixteenBySixteen);
        let board2 = board1;
        assert!(!(board1 ^ board2).is_set(pos));
    }

    #[test]
    fn bitboard_xor_both_set() {
        let pos = Pos::new(1, 3);
        let board1 = BitBoard::new(BoardSize::SixteenBySixteen).set(pos);
        let board2 = board1;
        assert!(!(board1 ^ board2).is_set(pos));
    }

    #[test]
    fn bitboard_xor_first_set() {
        let pos = Pos::new(1, 3);
        let board1 = BitBoard::new(BoardSize::SixteenBySixteen).set(pos);
        let board2 = BitBoard::new(BoardSize::SixteenBySixteen);
        assert!((board1 ^ board2).is_set(pos));
    }

    #[test]
    fn bitboard_xor_second_set() {
        let pos = Pos::new(1, 3);
        let board1 = BitBoard::new(BoardSize::SixteenBySixteen);
        let board2 = BitBoard::new(BoardSize::SixteenBySixteen).set(pos);
        assert!((board1 ^ board2).is_set(pos));
    }

    #[test]
    fn bitboard_not_unset() {
        let pos = Pos::new(1, 3);
        assert!((!BitBoard::new(BoardSize::SixteenBySixteen)).is_set(pos));
    }

    #[test]
    fn bitboard_not_set() {
        let pos = Pos::new(1, 3);
        assert!(!(!BitBoard::new(BoardSize::SixteenBySixteen).set(pos)).is_set(pos));
    }

    #[test]
    fn bitboard_display_shows_state() {
        let board = BitBoard::new(BoardSize::FifteenBySeventeen)
            .set(Pos::new(14, 16))
            .set(Pos::new(3, 4))
            .set(Pos::new(2, 2))
            .set(Pos::new(10, 4))
            .unset(Pos::new(10, 4))
            .set(Pos::new(11, 7));

        let expected = "\
        000000000000001\
        \n000000000000000\
        \n000000000000000\
        \n000000000000000\
        \n000000000000000\
        \n000000000000000\
        \n000000000000000\
        \n000000000000000\
        \n000000000000000\
        \n000000000001000\
        \n000000000000000\
        \n000000000000000\
        \n000100000000000\
        \n000000000000000\
        \n001000000000000\
        \n000000000000000\
        \n000000000000000\
        \n";

        assert_eq!(expected, format!("{}", board));
    }

}
