use crate::position::Pos;
use std::fmt::{Display, Formatter};
use serde::{Serialize, Deserialize};
use bitvec::bitvec;
use bitvec::vec::BitVec;

/// Efficiently maintains the state of a board with bits.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Serialize, Deserialize)]
pub(crate) struct BitBoard {
    board: BitVec,
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
    /// * `width` - the horizontal size of the bitboard to create
    /// * `height` - the vertical size of the bitboard to create
    pub fn new(width: u8, height: u8) -> BitBoard {
        BitBoard {
            board: bitvec![0; usize::from(width) * usize::from(height)],
            width,
            height
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
        self.board[self.bit_pos(pos)]
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
    pub fn set(&mut self, pos: Pos) {
        let bit_pos = self.bit_pos(pos);
        self.board.set(bit_pos, true)
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
    pub fn unset(&mut self, pos: Pos) {
        let bit_pos = self.bit_pos(pos);
        self.board.set(bit_pos, false)
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
    pub fn swap(&mut self, first: Pos, second: Pos) {
        let first_bit_pos = self.bit_pos(first);
        let second_bit_pos = self.bit_pos(second);
        self.board.swap(first_bit_pos, second_bit_pos)
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

        usize::from(pos.y()) * usize::from(self.width) + usize::from(pos.x())
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
    use crate::bitboard::BitBoard;
    use crate::position::Pos;

    #[test]
    fn bitboard_new_no_size_does_not_panic() {
        BitBoard::new(0, 0);
    }
    #[test]
    fn bitboard_new_no_width_does_not_panic() {
        BitBoard::new(0, 10);
    }
    #[test]
    fn bitboard_new_no_height_does_not_panic() {
        BitBoard::new(10, 0);
    }

    #[test]
    #[should_panic]
    fn bitboard_is_set_out_of_bounds_x_panics() {
        assert!(BitBoard::new(15, 17).is_set(Pos::new(15, 5)));
    }

    #[test]
    #[should_panic]
    fn bitboard_is_set_out_of_bounds_y_panics() {
        assert!(BitBoard::new(15, 17).is_set(Pos::new(5, 17)));
    }

    #[test]
    fn bitboard_modify_indices_do_not_collide() {
        let pos = Pos::new(1, 0);

        /* Collision occurs if index is calculated incorrectly: 1 * 15 + 0 == 0 * 15 + 15 ;
           x * width + y should be y * width + x */
        let poss_colliding_pos = Pos::new(0, 15);

        let mut board = BitBoard::new(15, 17);
        board.set(pos);

        assert!(!board.is_set(poss_colliding_pos));
    }

    #[test]
    fn bitboard_set_previously_unset() {
        let pos = Pos::new(1, 3);
        let mut board = BitBoard::new(16, 16);
        board.set(pos);
        assert!(board.is_set(pos));
    }

    #[test]
    fn bitboard_set_previously_set() {
        let pos = Pos::new(1, 3);
        let mut board = BitBoard::new(16, 16);
        board.set(pos);
        board.set(pos);
        assert!(board.is_set(pos));
    }

    #[test]
    fn bitboard_set_large_board() {
        let pos = Pos::new(254, 254);
        let mut board = BitBoard::new(255, 255);
        board.set(pos);
        board.set(pos);
        assert!(board.is_set(pos));
    }

    #[test]
    #[should_panic]
    fn bitboard_set_out_of_bounds_x_panics() {
        let pos = Pos::new(16, 5);
        BitBoard::new(16, 16).set(pos);
    }

    #[test]
    #[should_panic]
    fn bitboard_set_out_of_bounds_y_panics() {
        let pos = Pos::new(5, 16);
        BitBoard::new(16, 16).set(pos);
    }

    #[test]
    fn bitboard_unset_previously_unset() {
        let pos = Pos::new(1, 3);
        let mut board = BitBoard::new(16, 16);
        board.unset(pos);
        assert!(!board.is_set(pos));
    }

    #[test]
    fn bitboard_unset_previously_set() {
        let pos = Pos::new(1, 3);
        let mut board = BitBoard::new(16, 16);
        board.set(pos);
        board.unset(pos);
        assert!(!board.is_set(pos));
    }

    #[test]
    fn bitboard_unset_large_board() {
        let pos = Pos::new(254, 254);
        let mut board = BitBoard::new(255, 255);
        board.set(pos);
        board.unset(pos);
        assert!(!board.is_set(pos));
    }

    #[test]
    #[should_panic]
    fn bitboard_unset_out_of_bounds_x_panics() {
        let pos = Pos::new(15, 5);
        BitBoard::new(15, 17).unset(pos);
    }

    #[test]
    #[should_panic]
    fn bitboard_unset_out_of_bounds_y_panics() {
        let pos = Pos::new(5, 17);
        BitBoard::new(15, 17).unset(pos);
    }

    #[test]
    fn bitboard_swap_both_unset() {
        let pos1 = Pos::new(1, 3);
        let pos2 = Pos::new(0, 5);
        let mut board = BitBoard::new(16, 16);
        board.swap(pos1, pos2);
        assert!(!board.is_set(pos1));
        assert!(!board.is_set(pos2));
    }

    #[test]
    fn bitboard_swap_both_set() {
        let pos1 = Pos::new(1, 3);
        let pos2 = Pos::new(0, 5);
        let mut board = BitBoard::new(16, 16);
        board.set(pos1);
        board.set(pos2);
        board.swap(pos1, pos2);
        assert!(board.is_set(pos1));
        assert!(board.is_set(pos2));
    }

    #[test]
    fn bitboard_swap_first_set() {
        let pos1 = Pos::new(1, 3);
        let pos2 = Pos::new(0, 5);
        let mut board = BitBoard::new(16, 16);
        board.set(pos1);
        board.swap(pos1, pos2);
        assert!(!board.is_set(pos1));
        assert!(board.is_set(pos2));
    }

    #[test]
    fn bitboard_swap_second_set() {
        let pos1 = Pos::new(1, 3);
        let pos2 = Pos::new(0, 5);
        let mut board = BitBoard::new(16, 16);
        board.set(pos2);
        board.swap(pos1, pos2);
        assert!(board.is_set(pos1));
        assert!(!board.is_set(pos2));
    }

    #[test]
    fn bitboard_swap_large_board() {
        let pos1 = Pos::new(254, 254);
        let pos2 = Pos::new(0, 5);
        let mut board = BitBoard::new(255, 255);
        board.set(pos2);
        board.swap(pos1, pos2);
        assert!(board.is_set(pos1));
        assert!(!board.is_set(pos2));
    }

    #[test]
    #[should_panic]
    fn bitboard_swap_out_of_bounds_first_x_panics() {
        let pos1 = Pos::new(15, 3);
        let pos2 = Pos::new(0, 5);
        BitBoard::new(15, 17).swap(pos1, pos2);
    }

    #[test]
    #[should_panic]
    fn bitboard_swap_out_of_bounds_first_y_panics() {
        let pos1 = Pos::new(1, 17);
        let pos2 = Pos::new(0, 5);
        BitBoard::new(15, 17).swap(pos1, pos2);
    }

    #[test]
    #[should_panic]
    fn bitboard_swap_out_of_bounds_second_x_panics() {
        let pos1 = Pos::new(1, 3);
        let pos2 = Pos::new(15, 5);
        BitBoard::new(15, 17).swap(pos1, pos2);
    }

    #[test]
    #[should_panic]
    fn bitboard_swap_out_of_bounds_second_y_panics() {
        let pos1 = Pos::new(1, 3);
        let pos2 = Pos::new(0, 17);
        BitBoard::new(15, 17).swap(pos1, pos2);
    }

    #[test]
    fn bitboard_display_shows_state() {
        let mut board = BitBoard::new(15, 17);
        board.set(Pos::new(14, 16));
        board.set(Pos::new(3, 4));
        board.set(Pos::new(2, 2));
        board.set(Pos::new(10, 4));
        board.unset(Pos::new(10, 4));
        board.set(Pos::new(11, 7));

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
