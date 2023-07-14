use std::collections::VecDeque;
use crate::position::Pos;

use std::ops::BitAnd;

/// A piece with one or more match types. Two pieces should be equal when they match the same
/// match types.
pub trait Piece: Copy + From<Self::MatchType> + Default + BitAnd<Output=Self> + Eq {

    /// A type that all pieces in a pattern must have to create a match.
    type MatchType;

    /// A piece that matches no match types and is treated as empty.
    const AIR: Self;

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
    pieces: [[P; HEIGHT]; WIDTH],

    // TODO: The inner array is one element too large, but generic const exprs aren't stable yet
    horizontal_barriers: [[bool; HEIGHT]; WIDTH],
    vertical_barriers: [[bool; WIDTH]; HEIGHT]
}

impl<P: Piece, const W: usize, const H: usize> BoardState<P, W, H> {

    /// Creates a new board filled with default pieces (according to the [Default] trait).
    pub fn new() -> BoardState<P, W, H> {
        BoardState {
            pieces: [[P::default(); H]; W],
            horizontal_barriers: [[false; H]; W],
            vertical_barriers: [[false; W]; H]
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
        if !self.is_in_bounds(pos) {
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
        if !self.is_in_bounds(pos) {
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

    /// Finds the y position of a space with air that represents the "surface" of the given column.
    /// The surface is the position where a piece would be if it was dropped into the column from
    /// the top of the board.
    ///
    /// If the entire column is filled with air and has no barriers, and the board has height
    /// greater than zero, the surface is the bottommost position in the column (y = 0). Otherwise,
    /// the surface is immediately above the topmost piece or barrier, whichever is higher. If
    /// there is no space above the topmost piece, there is no surface.
    ///
    /// # Arguments
    ///
    /// `x` - x-coordinate of the column to find the surface of
    ///
    /// # Panics
    ///
    /// Panics if the given column does not exist.
    pub fn surface(&self, x: usize) -> Option<usize> {
        if x >= W {
            panic!("Column index {} is not within the board", x);
        }

        let column = &self.pieces[x];

        for y in (1..=H).rev() {
            let is_pos_below_filled = column[y - 1] != P::AIR;
            let pos = Pos::new(x, y);
            let has_barrier_below = self.is_in_bounds(pos)
                && self.has_barrier_between(pos, Pos::new(x, y - 1));

            if is_pos_below_filled || has_barrier_below {
                return match y == H {
                    true => None,
                    false => Some(y)
                };
            }
        }

        match H == 0 {
            true => None,
            false => Some(0)
        }
    }

    /// Moves all pieces in the given column as if they were falling due to gravity. The bottom of
    /// the board, horizontal barriers, and other pieces will block the fall of pieces in the given
    /// column. This method returns (before, after) pairs of y-coordinates that describe how the
    /// pieces were moved.
    ///
    /// # Arguments
    ///
    /// `x` - x-coordinate of the column to apply gravity to
    ///
    /// # Panics
    ///
    /// Panics if the given column does not exist.
    pub fn apply_gravity_to_column(&mut self, x: usize) -> Vec<(usize, usize)> {
        if x >= W {
            panic!("Column index {} is not within the board", x);
        }

        let mut air_ys = VecDeque::new();
        let mut moves = Vec::new();

        for y in 0..H {
            let pos = Pos::new(x, y);

            if self.pieces[x][y] == P::AIR {
                air_ys.push_back(y);
            } else if let Some(air_y) = air_ys.pop_front() {
                self.swap(pos, Pos::new(x, air_y));
                moves.push((y, air_y));
                air_ys.push_back(y);
            }

            let pos_above = Pos::new(x, y + 1);
            if self.is_in_bounds(pos_above) && self.has_barrier_between(pos, pos_above) {
                air_ys.clear();
            }
        }

        moves
    }

    /// Checks if a given position is inside the board.
    ///
    /// # Arguments
    ///
    /// * `pos` - the position to check
    pub fn is_in_bounds(&self, pos: Pos) -> bool {
        pos.x() < W && pos.y() < H
    }

    /// Checks whether there is a barrier between two positions.
    ///
    /// # Arguments
    ///
    /// * `first` - first position to check whether there is a barrier between
    /// * `second` - second position to check whether there is a barrier between
    ///
    /// # Panics
    ///
    /// Panics if the two positions are not vertically or horizontally adjacent.
    pub fn has_barrier_between(&self, first: Pos, second: Pos) -> bool {
        if !self.is_in_bounds(first) || !self.is_in_bounds(second) {
            panic!("Tried to check barrier with piece out of bounds: {} and {}", first, second);
        }

        if BoardState::<P, W, H>::vertically_adjacent(first, second) {
            let x = first.x();
            let barrier_index = BoardState::<P, W, H>::horizontal_barrier_index(first, second);
            return self.horizontal_barriers[x][barrier_index]
        } else if BoardState::<P, W, H>::horizontally_adjacent(first, second) {
            let y = first.y();
            let barrier_index = BoardState::<P, W, H>::vertical_barrier_index(first, second);
            return self.vertical_barriers[y][barrier_index]
        }

        panic!("Barriers only exist between adjacent positions, but {} and {} were provided",
               first, second)
    }

    /// Sets whether there is a barrier between two positions.
    ///
    /// # Arguments
    ///
    /// * `first` - first position to set a barrier between
    /// * `second` - second position to set a barrier between
    /// * `has_barrier` - whether there should be a barrier between the two positions
    ///
    /// # Panics
    ///
    /// Panics if the two positions are not vertically or horizontally adjacent.
    pub fn set_barrier_between(&mut self, first: Pos, second: Pos, has_barrier: bool) {
        if !self.is_in_bounds(first) || !self.is_in_bounds(second) {
            panic!("Tried to set barrier with piece out of bounds: {} and {}", first, second);
        }

        if BoardState::<P, W, H>::vertically_adjacent(first, second) {
            let x = first.x();
            let barrier_index = BoardState::<P, W, H>::horizontal_barrier_index(first, second);
            self.horizontal_barriers[x][barrier_index] = has_barrier
        } else if BoardState::<P, W, H>::horizontally_adjacent(first, second) {
            let y = first.y();
            let barrier_index = BoardState::<P, W, H>::vertical_barrier_index(first, second);
            self.vertical_barriers[y][barrier_index] = has_barrier
        } else {
            panic!("Barriers only exist between adjacent positions, but {} and {} were provided",
                   first, second)
        }
    }

    /// Checks if two positions are horizontally adjacent.
    ///
    /// # Arguments
    ///
    /// * `first` - first position to compare
    /// * `second` - second position to compare
    fn horizontally_adjacent(first: Pos, second: Pos) -> bool {
        usize::abs_diff(first.x(), second.x()) == 1
            && usize::abs_diff(first.y(), second.y()) == 0
    }

    /// Checks if two positions are vertically adjacent.
    ///
    /// # Arguments
    ///
    /// * `first` - first position to compare
    /// * `second` - second position to compare
    fn vertically_adjacent(first: Pos, second: Pos) -> bool {
        usize::abs_diff(first.x(), second.x()) == 0
            && usize::abs_diff(first.y(), second.y()) == 1
    }

    /// Retrieves the index of a horizontal barrier within an array of horizontal barriers for
    /// the same column.
    ///
    /// # Arguments
    ///
    /// * `first` - first position that the barrier exists between
    /// * `second` - second position that the barrier exists between
    fn horizontal_barrier_index(first: Pos, second: Pos) -> usize {
        first.y().min(second.y())
    }

    /// Retrieves the index of a vertical barrier within an array of vertical barriers for
    /// the same row.
    ///
    /// # Arguments
    ///
    /// * `first` - first position that the barrier exists between
    /// * `second` - second position that the barrier exists between
    fn vertical_barrier_index(first: Pos, second: Pos) -> usize {
        first.x().min(second.x())
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
        const AIR: Self = Self::None;
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

    #[test]
    fn has_barrier_barriers_unset_defaults_false() {
        let board: BoardState<TestPiece, 15, 16> = BoardState::new();
        for x in 0..14 {
            for y in 0..15 {
                assert!(!board.has_barrier_between(Pos::new(x, y), Pos::new(x, y + 1)));
                assert!(!board.has_barrier_between(Pos::new(x, y), Pos::new(x + 1, y)));
            }
        }
    }

    #[test]
    fn has_barrier_barrier_set_true() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        for x in 0..14 {
            for y in 0..15 {
                board.set_barrier_between(Pos::new(x, y), Pos::new(x, y + 1), true);
                assert!(board.has_barrier_between(Pos::new(x, y), Pos::new(x, y + 1)));
                board.set_barrier_between(Pos::new(x, y), Pos::new(x, y + 1), false);

                board.set_barrier_between(Pos::new(x, y), Pos::new(x + 1, y), true);
                assert!(board.has_barrier_between(Pos::new(x, y), Pos::new(x + 1, y)));
                board.set_barrier_between(Pos::new(x, y), Pos::new(x + 1, y), false);
            }
        }
    }

    #[test]
    fn has_barrier_barrier_set_twice_true() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        for x in 0..14 {
            for y in 0..15 {
                board.set_barrier_between(Pos::new(x, y), Pos::new(x, y + 1), true);
                board.set_barrier_between(Pos::new(x, y), Pos::new(x, y + 1), true);
                assert!(board.has_barrier_between(Pos::new(x, y), Pos::new(x, y + 1)));
                board.set_barrier_between(Pos::new(x, y), Pos::new(x, y + 1), false);

                board.set_barrier_between(Pos::new(x, y), Pos::new(x + 1, y), true);
                board.set_barrier_between(Pos::new(x, y), Pos::new(x + 1, y), true);
                assert!(board.has_barrier_between(Pos::new(x, y), Pos::new(x + 1, y)));
                board.set_barrier_between(Pos::new(x, y), Pos::new(x + 1, y), false);
            }
        }
    }

    #[test]
    fn has_barrier_barrier_overwritten_false() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        for x in 0..14 {
            for y in 0..15 {
                board.set_barrier_between(Pos::new(x, y), Pos::new(x, y + 1), true);
                board.set_barrier_between(Pos::new(x, y), Pos::new(x, y + 1), false);
                assert!(!board.has_barrier_between(Pos::new(x, y), Pos::new(x, y + 1)));

                board.set_barrier_between(Pos::new(x, y), Pos::new(x + 1, y), true);
                board.set_barrier_between(Pos::new(x, y), Pos::new(x + 1, y), false);
                assert!(!board.has_barrier_between(Pos::new(x, y), Pos::new(x + 1, y)));
            }
        }
    }

    #[test]
    #[should_panic]
    fn set_barrier_first_pos_out_of_bounds_panics() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        board.set_barrier_between(Pos::new(14, 16), Pos::new(14, 15), true);
    }

    #[test]
    #[should_panic]
    fn set_barrier_second_pos_out_of_bounds_panics() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        board.set_barrier_between(Pos::new(14, 15), Pos::new(14, 16), true);
    }

    #[test]
    #[should_panic]
    fn set_barrier_first_pos_very_large_panics() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        board.set_barrier_between(Pos::new(usize::MAX, usize::MAX), Pos::new(14, 15), true);
    }

    #[test]
    #[should_panic]
    fn set_barrier_second_pos_very_large_panics() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        board.set_barrier_between(Pos::new(14, 15), Pos::new(usize::MAX, usize::MAX), true);
    }

    #[test]
    #[should_panic]
    fn set_barrier_horizontally_separated_panics() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        board.set_barrier_between(Pos::new(1, 2), Pos::new(3, 2), true);
    }

    #[test]
    #[should_panic]
    fn set_barrier_vertically_separated_panics() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        board.set_barrier_between(Pos::new(1, 2), Pos::new(1, 4), true);
    }

    #[test]
    #[should_panic]
    fn set_barrier_diagonally_adjacent_panics() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        board.set_barrier_between(Pos::new(1, 2), Pos::new(2, 3), true);
    }

    #[test]
    #[should_panic]
    fn has_barrier_first_pos_out_of_bounds_panics() {
        let board: BoardState<TestPiece, 15, 16> = BoardState::new();
        board.has_barrier_between(Pos::new(14, 16), Pos::new(14, 15));
    }

    #[test]
    #[should_panic]
    fn has_barrier_second_pos_out_of_bounds_panics() {
        let board: BoardState<TestPiece, 15, 16> = BoardState::new();
        board.has_barrier_between(Pos::new(14, 15), Pos::new(14, 16));
    }

    #[test]
    #[should_panic]
    fn has_barrier_first_pos_very_large_panics() {
        let board: BoardState<TestPiece, 15, 16> = BoardState::new();
        board.has_barrier_between(Pos::new(usize::MAX, usize::MAX), Pos::new(14, 15));
    }

    #[test]
    #[should_panic]
    fn has_barrier_second_pos_very_large_panics() {
        let board: BoardState<TestPiece, 15, 16> = BoardState::new();
        board.has_barrier_between(Pos::new(14, 15), Pos::new(usize::MAX, usize::MAX));
    }

    #[test]
    #[should_panic]
    fn has_barrier_horizontally_separated_panics() {
        let board: BoardState<TestPiece, 15, 16> = BoardState::new();
        board.has_barrier_between(Pos::new(1, 2), Pos::new(3, 2));
    }

    #[test]
    #[should_panic]
    fn has_barrier_vertically_separated_panics() {
        let board: BoardState<TestPiece, 15, 16> = BoardState::new();
        board.has_barrier_between(Pos::new(1, 2), Pos::new(1, 4));
    }

    #[test]
    #[should_panic]
    fn has_barrier_diagonally_adjacent_panics() {
        let board: BoardState<TestPiece, 15, 16> = BoardState::new();
        board.has_barrier_between(Pos::new(1, 2), Pos::new(2, 3));
    }

    #[test]
    fn surface_zero_height_none() {
        let board: BoardState<TestPiece, 15, 0> = BoardState::new();
        assert!(board.surface(1).is_none());
    }

    #[test]
    fn surface_all_air_zero() {
        let board: BoardState<TestPiece, 15, 16> = BoardState::new();
        assert_eq!(0, board.surface(1).unwrap());
    }

    #[test]
    fn surface_all_filled_none() {
        let mut board: BoardState<TestPiece, 15, 8> = BoardState::new();
        let x = 1;

        for y in 0..8 {
            board.set_piece(Pos::new(x, y), TestPiece::First);
        }

        assert!(board.surface(x).is_none());
    }

    #[test]
    fn surface_top_filled_none() {
        let mut board: BoardState<TestPiece, 15, 8> = BoardState::new();
        let x = 1;

        for y in 4..8 {
            board.set_piece(Pos::new(x, y), TestPiece::First);
        }

        assert!(board.surface(x).is_none());
    }

    #[test]
    fn surface_air_pockets_finds_topmost() {
        let mut board: BoardState<TestPiece, 15, 8> = BoardState::new();
        let x = 1;

        board.set_piece(Pos::new(x, 1), TestPiece::First);
        board.set_piece(Pos::new(x, 2), TestPiece::Second);
        board.set_piece(Pos::new(x, 5), TestPiece::First);

        board.set_barrier_between(Pos::new(x, 2), Pos::new(x, 3), true);

        assert_eq!(6, board.surface(x).unwrap());
    }

    #[test]
    fn surface_barrier_below_top_finds_barrier() {
        let mut board: BoardState<TestPiece, 15, 8> = BoardState::new();
        let x = 1;

        board.set_piece(Pos::new(x, 1), TestPiece::First);
        board.set_piece(Pos::new(x, 2), TestPiece::Second);
        board.set_piece(Pos::new(x, 5), TestPiece::First);

        board.set_barrier_between(Pos::new(x, 2), Pos::new(x, 3), true);
        board.set_barrier_between(Pos::new(x, 6), Pos::new(x, 7), true);

        assert_eq!(7, board.surface(x).unwrap());
    }

    #[test]
    #[should_panic]
    fn surface_column_index_out_of_bounds_panics() {
        let board: BoardState<TestPiece, 15, 16> = BoardState::new();
        board.surface(15);
    }

    #[test]
    #[should_panic]
    fn surface_column_index_very_large_panics() {
        let board: BoardState<TestPiece, 15, 16> = BoardState::new();
        board.surface(usize::MAX);
    }

    #[test]
    fn column_gravity_zero_height_no_exception() {
        let mut board: BoardState<TestPiece, 15, 0> = BoardState::new();
        assert!(board.apply_gravity_to_column(1).is_empty());
    }

    #[test]
    fn column_gravity_all_air_unchanged() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        assert!(board.apply_gravity_to_column(1).is_empty());
        for y in 0..16 {
            assert_eq!(TestPiece::None, board.piece(Pos::new(1, y)));
        }
    }

    #[test]
    fn column_gravity_all_filled_unchanged() {
        let mut board: BoardState<TestPiece, 15, 8> = BoardState::new();
        let x = 1;

        for y in 0..8 {
            board.set_piece(Pos::new(x, y), TestPiece::First);
        }

        assert!(board.apply_gravity_to_column(1).is_empty());
        for y in 0..8 {
            assert_eq!(TestPiece::First, board.piece(Pos::new(1, y)));
        }
    }

    #[test]
    fn column_gravity_air_pockets_pieces_fall() {
        let mut board: BoardState<TestPiece, 15, 8> = BoardState::new();
        let x = 1;

        board.set_piece(Pos::new(x, 0), TestPiece::First);
        board.set_piece(Pos::new(x, 2), TestPiece::Second);
        board.set_piece(Pos::new(x, 5), TestPiece::First);
        board.set_piece(Pos::new(x, 7), TestPiece::First);

        board.set_barrier_between(Pos::new(x, 3), Pos::new(x, 4), true);

        assert_eq!(vec![(2, 1), (5, 4), (7, 5)], board.apply_gravity_to_column(1));

        assert_eq!(TestPiece::First, board.piece(Pos::new(x, 0)));
        assert_eq!(TestPiece::Second, board.piece(Pos::new(x, 1)));
        assert_eq!(TestPiece::None, board.piece(Pos::new(x, 2)));
        assert_eq!(TestPiece::None, board.piece(Pos::new(x, 3)));
        assert_eq!(TestPiece::First, board.piece(Pos::new(x, 4)));
        assert_eq!(TestPiece::First, board.piece(Pos::new(x, 5)));
        assert_eq!(TestPiece::None, board.piece(Pos::new(x, 6)));
        assert_eq!(TestPiece::None, board.piece(Pos::new(x, 7)));
    }

    #[test]
    #[should_panic]
    fn column_gravity_column_index_out_of_bounds_panics() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        board.apply_gravity_to_column(15);
    }

    #[test]
    #[should_panic]
    fn column_gravity_column_index_very_large_panics() {
        let mut board: BoardState<TestPiece, 15, 16> = BoardState::new();
        board.apply_gravity_to_column(usize::MAX);
    }
}
