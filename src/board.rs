use crate::position::Pos;

use std::ops::BitAnd;

/// A piece with one or more match types. Two pieces should be equal when they match the same
/// match types.
pub trait Piece: Copy + From<Self::MatchType> + Default + BitAnd<Output=Self> + Eq {

    /// A type that all pieces in a pattern must have to create a match.
    type MatchType;

    /// A piece that matches no match types.
    const UNMATCHABLE: Self;

}

/// Contains zero or many pieces and represents the current state
/// of the game.
///
/// Positions with larger y values are higher on the board. Positions
/// with larger x values are further right on the board. Positions start
/// at (0, 0), so a position at (16, 16) would be outside a 16x16 board
/// horizontally and vertically.
///
/// The board's lack of default restrictions allows games to implement
/// their own unique or non-standard rules.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BoardState<
    P,
    const WIDTH: usize,
    const HEIGHT: usize
> {
    pieces: [[P; HEIGHT]; WIDTH]
}

impl<P: Piece, const W: usize, const H: usize> BoardState<P, W, H> {

    /// Creates a new board filled with default pieces (according to the [Default] trait).
    pub fn new() -> BoardState<P, W, H> {
        BoardState {
            pieces: [[P::default(); H]; W]
        }
    }

    /// Gets the type of a piece at a certain position.
    ///
    /// # Arguments
    ///
    /// * `pos` - the position of the piece whose type to find
    ///
    /// # Panics
    ///
    /// Panics if the provided position is outside the board.
    pub fn piece(&self, pos: Pos) -> P {
        if !self.is_within_board(pos) {
            panic!("Tried to access piece outside board: {}", pos);
        }

        self.pieces[pos.x()][pos.y()]
    }

    /// Replaces a piece at the given position and returns the previous piece.
    ///
    /// # Arguments
    ///
    /// * `pos` - the position of the piece to replace
    /// * `piece` - the piece to put at the given position
    ///
    /// # Panics
    ///
    /// Panics if the provided position is outside the board.
    pub fn set_piece(&mut self, pos: Pos, piece: P) -> P {
        if !self.is_within_board(pos) {
            panic!("Tried to set piece out of bounds: {}", pos);
        }

        let old_piece = self.pieces[pos.x()][pos.y()];
        self.pieces[pos.x()][pos.y()] = piece;
        old_piece
    }

    /// Swap two pieces on the board. The order of two positions provided does not matter.
    ///
    /// # Arguments
    ///
    /// * `first` - the first position of a piece to swap
    /// * `second` - the second position of a piece to swap
    ///
    /// # Panics
    ///
    /// Panics if either position is outside the board.
    pub fn swap(&mut self, first: Pos, second: Pos) {
        let old_first = self.set_piece(first, self.piece(second));
        self.set_piece(second, old_first);
    }

    /// Checks if a given position is inside the board.
    ///
    /// # Arguments
    ///
    /// * `pos` - the position to check
    pub(crate) fn is_within_board(&self, pos: Pos) -> bool {
        pos.x() < W && pos.y() < H
    }

}

impl<P: Piece, const W: usize, const H: usize> Default for BoardState<P, W, H> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::ops::BitAnd;
    use crate::{BoardState, Piece, Pos};

    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
    enum TestPiece {
        #[default]
        None = 0b00,
        First = 0b01,
        Second = 0b10
    }

    impl From<u8> for TestPiece {
        fn from(value: u8) -> Self {
            match value {
                0 => TestPiece::First,
                1 => TestPiece::Second,
                _ => TestPiece::None
            }
        }
    }

    impl BitAnd for TestPiece {
        type Output = Self;

        fn bitand(self, rhs: Self) -> Self::Output {
            match self as u8 & rhs as u8 {
                0b00 => TestPiece::None,
                0b01 => TestPiece::First,
                0b10 => TestPiece::Second,
                _ => panic!("impossible")
            }
        }
    }

    impl Piece for TestPiece {
        type MatchType = u8;
        const UNMATCHABLE: Self = Self::None;
    }

    #[test]
    fn get_piece_zero_zero_default_retrieved() {
        let board: BoardState<TestPiece, 15, 16> = BoardState::new();
        assert_eq!(TestPiece::None, board.piece(Pos::new(0, 0)));
    }

    #[test]
    fn get_piece_never_set_default_retrieved() {
        let board: BoardState<TestPiece, 15, 16> = BoardState::new();
        assert_eq!(TestPiece::None, board.piece(Pos::new(5, 10)));
    }

    #[test]
    #[should_panic]
    fn get_piece_out_of_bounds_x_panics() {
        let board: BoardState<TestPiece, 15, 16> = BoardState::new();
        board.piece(Pos::new(15, 15));
    }

    #[test]
    #[should_panic]
    fn get_piece_out_of_bounds_y_panics() {
        let board: BoardState<TestPiece, 15, 16> = BoardState::new();
        board.piece(Pos::new(14, 16));
    }

    #[test]
    fn swap_adjacent_swapped() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        let piece1 = TestPiece::First;
        let piece2 = TestPiece::Second;

        board.set_piece(Pos::new(1, 2), piece1);
        board.set_piece(Pos::new(1, 3), piece2);

        board.swap(Pos::new(1, 2), Pos::new(1, 3));
        assert_eq!(piece2, board.piece(Pos::new(1, 2)));
        assert_eq!(piece1, board.piece(Pos::new(1, 3)));
    }

    #[test]
    fn swap_non_adjacent_swapped() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        let piece1 = TestPiece::First;
        let piece2 = TestPiece::Second;

        board.set_piece(Pos::new(1, 2), piece1);
        board.set_piece(Pos::new(14, 15), piece2);

        board.swap(Pos::new(1, 2), Pos::new(14, 15));
        assert_eq!(piece2, board.piece(Pos::new(1, 2)));
        assert_eq!(piece1, board.piece(Pos::new(14, 15)));
    }

    #[test]
    #[should_panic]
    fn swap_first_pos_outside_board_panics() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        let piece1 = TestPiece::First;

        board.set_piece(Pos::new(1, 2), piece1);

        board.swap(Pos::new(15, 15), Pos::new(1, 2));
    }

    #[test]
    #[should_panic]
    fn swap_first_pos_very_large_panics() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        let piece1 = TestPiece::First;

        board.set_piece(Pos::new(1, 2), piece1);

        board.swap(Pos::new(usize::MAX, usize::MAX), Pos::new(1, 2));
    }

    #[test]
    #[should_panic]
    fn swap_second_pos_outside_board_panics() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        let piece1 = TestPiece::First;

        board.set_piece(Pos::new(1, 2), piece1);

        board.swap(Pos::new(1, 2), Pos::new(14, 16));
    }

    #[test]
    #[should_panic]
    fn swap_second_pos_very_large_panics() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        let piece1 = TestPiece::First;

        board.set_piece(Pos::new(1, 2), piece1);

        board.swap(Pos::new(1, 2), Pos::new(usize::MAX, usize::MAX));
    }

    #[test]
    fn swap_self_no_change() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        let piece1 = TestPiece::First;

        board.set_piece(Pos::new(1, 2), piece1);

        board.swap(Pos::new(1, 2), Pos::new(1, 2));
        assert_eq!(piece1, board.piece(Pos::new(1, 2)));
    }

    #[test]
    fn set_piece_none_previous_default_returned() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        let piece1 = TestPiece::First;

        assert_eq!(TestPiece::default(), board.set_piece(Pos::new(1, 2), piece1));
        assert_eq!(piece1, board.piece(Pos::new(1, 2)));
    }

    #[test]
    fn set_piece_one_previous_old_returned() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        let piece1 = TestPiece::First;
        let piece2 = TestPiece::Second;

        assert_eq!(TestPiece::default(), board.set_piece(Pos::new(1, 2), piece1));
        assert_eq!(piece1, board.set_piece(Pos::new(1, 2), piece2));
        assert_eq!(piece2, board.piece(Pos::new(1, 2)));
    }

    #[test]
    fn set_piece_duplicate_old_returned() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        let piece1 = TestPiece::First;

        assert_eq!(TestPiece::default(), board.set_piece(Pos::new(1, 2), piece1));
        assert_eq!(piece1, board.set_piece(Pos::new(1, 2), piece1));
        assert_eq!(piece1, board.piece(Pos::new(1, 2)));
    }

    #[test]
    #[should_panic]
    fn set_piece_out_of_bounds_x_panics() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        let piece1 = TestPiece::First;

        board.set_piece(Pos::new(15, 15), piece1);
    }

    #[test]
    #[should_panic]
    fn set_piece_out_of_bounds_y_panics() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        let piece1 = TestPiece::First;

        board.set_piece(Pos::new(14, 16), piece1);
    }

    #[test]
    #[should_panic]
    fn set_piece_very_large_pos_panics() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        let piece1 = TestPiece::First;

        board.set_piece(Pos::new(usize::MAX, usize::MAX), piece1);
    }
}
