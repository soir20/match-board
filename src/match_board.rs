use std::collections::VecDeque;
use crate::{Board, Match, MatchPattern, Piece, Pos};

/// Keeps track of the current board state and computes matches.
///
/// The board detects matches based on user-provided match patterns.
/// It does not have any match patterns by default. Patterns with
/// higher rank are preferred over those with lower rank.
///
/// The whole board is not scanned to check for matches. When a
/// piece is changed, either because it is set/overwritten or it
/// is swapped, it is marked as having changed. Then the changed
/// pieces are selectively checked for matches. Users should update
/// the board based on the positions provided in the match.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct MatchBoard<
    'a,
    M,
    P,
    const WIDTH: usize,
    const HEIGHT: usize
> {
    board: Board<P, WIDTH, HEIGHT>,
    last_changed: VecDeque<Pos>,
    patterns: Vec<&'a MatchPattern<M>>
}

impl<M: Copy, P: Piece<MatchType=M>, const W: usize, const H: usize> MatchBoard<'_, M, P, W, H> {

    /// Creates a new match board.
    ///
    /// # Arguments
    ///
    /// * `board` - initial board state of the game (or the previous state if the game is
    ///             being resumed after a game shutdown)
    /// * `patterns` - match patterns to use to check for matches. Patterns will be checked in the
    ///                order provided. For example, if one pattern matches a column of five pieces
    ///                and another matches a column of three pieces, the column of five pattern
    ///                should probably be first.
    pub fn new(board: Board<P, W, H>, patterns: Vec<&MatchPattern<M>>) -> MatchBoard<M, P, W, H> {
        let mut last_changed = VecDeque::with_capacity(W * H);
        for x in 0..W {
            for y in 0..H {
                last_changed.push_back(Pos::new(x, y));
            }
        }

        MatchBoard {
            board,
            last_changed,
            patterns
        }
    }

    /// Ends the current game by returning the final board state.
    pub fn end_game(self) -> Board<P, W, H> {
        self.board
    }

    /// Gets the type of a piece at a certain position.
    ///
    /// # Arguments
    ///
    /// * `pos` - the position of the piece whose type to find
    pub fn piece(&self, pos: Pos) -> P {
        self.board.piece(pos)
    }

    /// Replaces a piece at the given position and returns the previous piece.
    /// The space is marked as needing a match check.
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
        self.last_changed.push_back(pos);
        self.board.set_piece(pos, piece)
    }

    /// Swap two pieces on the board. The order of two positions provided does not matter.
    /// The space is marked as needing a match check.
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
        if first == second {
            return;
        }

        self.last_changed.push_back(first);
        self.last_changed.push_back(second);

        self.board.swap(first, second)
    }

    /// Gets the next match on the board. Matches from pieces that were changed
    /// earlier are returned first. Matches are always based on the current board
    /// state, not the board state when the match occurred.
    ///
    /// All positions are checked for a match the first time this method is run.
    ///
    /// Pieces that were changed but did not create a match are skipped.
    ///
    /// Regardless of whether a match is found, each piece is unmarked for a
    /// match check, unless it has been marked multiple times.
    pub fn next_match(&mut self) -> Option<Match<M>> {
        let mut next_pos;
        let mut next_match = None;

        while next_match.is_none() {
            next_pos = self.last_changed.pop_front()?;

            next_match = self.patterns.iter().find_map(|pattern| {
                let positions = self.check_pattern(
                    pattern.match_type(),
                    pattern.spaces(),
                    next_pos
                )?;

                return Some(Match::new(pattern, next_pos, positions));
            });
        }

        next_match
    }

    /// Checks for a pattern that includes a specific position on the board. Looks
    /// for all variants of a pattern (all possible patterns that include the required
    /// position). Returns the positions on the board that correspond to that pattern
    /// if there is a match.
    ///
    /// # Arguments
    ///
    /// * `match_type` - the match type of the pattern
    /// * `pattern` - the set of relative positions that represent a pattern
    /// * `pos` - the position that must be included in a match
    fn check_pattern(&self, match_type: M, pattern: &[Pos], pos: Pos) -> Option<Vec<Pos>> {
        pattern.iter().find_map(|&original| {

            // Don't check variants outside the board
            if original.x() > pos.x() || original.y() > pos.y() {
                return None;
            }

            self.check_variant(match_type, pattern, pos - original)
        })
    }

    /// Checks for a single variant of a pattern and returns the corresponding positions
    /// on the board if found.
    ///
    /// # Arguments
    ///
    /// * `match_type` - the match type of the pattern
    /// * `pattern` - the set of relative positions that represent a variant
    /// * `new_origin` - the origin to use for the pattern positions so that they
    ///                  correspond to actual positions on the board
    fn check_variant(&self, match_type: M, pattern: &[Pos], new_origin: Pos) -> Option<Vec<Pos>> {
        let grid_pos = MatchBoard::<M, P, W, H>::change_origin(pattern, new_origin);
        let all_match = grid_pos.iter().all(
            |&pos| MatchBoard::<M, P, W, H>::matches(match_type, self.board.piece(pos))
        );
        match all_match {
            true => Some(grid_pos),
            false => None
        }
    }

    /// Checks if the given piece has the given match type.
    ///
    /// # Arguments
    ///
    /// * `match_type` - match type to compare to the piece
    /// * `piece` - piece to compare to the match type
    fn matches(match_type: M, piece: P) -> bool {
        let type_piece: P = match_type.into();
        (type_piece & piece).matches_any()
    }

    /// Changes the origin of a set of points.
    ///
    /// # Arguments
    ///
    /// * `positions` - the positions to change the origin of
    /// * `origin` - the new origin to use for the positions
    fn change_origin(positions: &[Pos], origin: Pos) -> Vec<Pos> {
        positions.iter().map(|&original| original + origin).collect()
    }

}

#[cfg(test)]
mod tests {
    use std::ops::BitAnd;
    use crate::{Board, MatchBoard, MatchPattern, Piece, Pos};

    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
    enum TestMatchType {
        First = 0b0,
        Second = 0b1
    }

    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
    enum TestPiece {
        #[default]
        None = 0b00,
        First = 0b01,
        Second = 0b10,
        Both = 0b11
    }

    impl From<TestMatchType> for TestPiece {
        fn from(value: TestMatchType) -> Self {
            match value {
                TestMatchType::First => TestPiece::First,
                TestMatchType::Second => TestPiece::Second
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
                0b11 => TestPiece::Both,
                _ => panic!("impossible")
            }
        }
    }

    impl Piece for TestPiece {
        type MatchType = TestMatchType;

        fn matches_any(&self) -> bool {
            *self != TestPiece::None
        }
    }

    #[test]
    #[should_panic]
    fn get_piece_out_of_bounds_x_panics() {
        let board = Board::<TestPiece, 15, 16>::new();

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(6, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let match_board = MatchBoard::new(board, vec![&pattern]);

        match_board.piece(Pos::new(15, 15));
    }

    #[test]
    fn get_piece_in_bounds_returns_piece() {
        let board = Board::<TestPiece, 15, 16>::new();

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(6, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let match_board = MatchBoard::new(board, vec![&pattern]);

        assert_eq!(TestPiece::None, match_board.piece(Pos::new(0, 0)));
    }

    #[test]
    #[should_panic]
    fn get_piece_out_of_bounds_y_panics() {
        let board = Board::<TestPiece, 15, 16>::new();

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(6, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let match_board = MatchBoard::new(board, vec![&pattern]);

        match_board.piece(Pos::new(14, 16));
    }


    #[test]
    #[should_panic]
    fn set_piece_out_of_bounds_x_panics() {
        let board = Board::<TestPiece, 15, 16>::new();

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(6, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let mut match_board = MatchBoard::new(board, vec![&pattern]);

        match_board.set_piece(Pos::new(15, 15), TestPiece::First);
    }

    #[test]
    #[should_panic]
    fn set_piece_out_of_bounds_y_panics() {
        let board = Board::<TestPiece, 15, 16>::new();

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(6, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let mut match_board = MatchBoard::new(board, vec![&pattern]);

        match_board.set_piece(Pos::new(14, 16), TestPiece::First);
    }

    #[test]
    #[should_panic]
    fn swap_piece_first_out_of_bounds_panics() {
        let board = Board::<TestPiece, 15, 16>::new();

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(6, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let mut match_board = MatchBoard::new(board, vec![&pattern]);

        match_board.swap(Pos::new(15, 15), Pos::new(0, 0));
    }

    #[test]
    #[should_panic]
    fn swap_piece_second_out_of_bounds_panics() {
        let board = Board::<TestPiece, 15, 16>::new();

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(6, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let mut match_board = MatchBoard::new(board, vec![&pattern]);

        match_board.swap(Pos::new(0, 0), Pos::new(15, 15));
    }

    #[test]
    fn next_match_no_patterns_none() {
        let mut board: MatchBoard<TestMatchType, TestPiece, 15, 16> = MatchBoard::new(
            Board::new(),
            Vec::new()
        );

        board.set_piece(Pos::new(1, 2), TestPiece::First);
        board.set_piece(Pos::new(2, 3), TestPiece::First);
        assert!(board.next_match().is_none());
    }

    #[test]
    fn next_match_checks_initial_board() {
        let mut board = Board::<TestPiece, 15, 16>::new();

        board.set_piece(Pos::new(0, 1), TestPiece::First);
        board.set_piece(Pos::new(1, 1), TestPiece::Both);
        board.set_piece(Pos::new(4, 6), TestPiece::First);

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(6, 8)];
        let pattern = MatchPattern::new(TestMatchType::First, &pattern_pos[..]);

        let mut match_board = MatchBoard::new(
            board,
            vec![&pattern]
        );

        let next_match = match_board.next_match().unwrap();
        assert_eq!(Pos::new(0, 1), next_match.changed_pos());
        assert!(next_match.board_pos().contains(&Pos::new(0, 1)));
        assert!(next_match.board_pos().contains(&Pos::new(1, 1)));
        assert!(next_match.board_pos().contains(&Pos::new(4, 6)));
    }

    #[test]
    fn next_match_set_pieces_matches_for_all_pieces() {
        let board = Board::<TestPiece, 15, 16>::new();

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(6, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let mut match_board = MatchBoard::new(board, vec![&pattern]);

        // Empty initial positions from last modified queue
        match_board.next_match();

        match_board.set_piece(Pos::new(0, 1), TestPiece::Second);
        match_board.set_piece(Pos::new(1, 1), TestPiece::Both);
        match_board.set_piece(Pos::new(4, 6), TestPiece::Second);

        let expected_changed_pos = vec![Pos::new(0, 1), Pos::new(1, 1), Pos::new(4, 6)];
        for i in 0..3 {
            let next_match = match_board.next_match().unwrap();
            assert_eq!(*expected_changed_pos.get(i).unwrap(), next_match.changed_pos());
            assert!(next_match.board_pos().contains(&Pos::new(0, 1)));
            assert!(next_match.board_pos().contains(&Pos::new(1, 1)));
            assert!(next_match.board_pos().contains(&Pos::new(4, 6)));
        }
    }

    #[test]
    fn next_match_swap_pieces_match_found_at_first() {
        let mut board = Board::<TestPiece, 15, 16>::new();

        board.set_piece(Pos::new(0, 1), TestPiece::Second);
        board.set_piece(Pos::new(1, 1), TestPiece::Both);
        board.set_piece(Pos::new(8, 8), TestPiece::Second);

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(8, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let mut match_board = MatchBoard::new(board, vec![&pattern]);

        // Empty initial positions from last modified queue
        match_board.next_match();

        match_board.swap(Pos::new(6, 6), Pos::new(8, 8));

        let next_match = match_board.next_match().unwrap();
        assert_eq!(Pos::new(6, 6), next_match.changed_pos());
        assert!(next_match.board_pos().contains(&Pos::new(0, 1)));
        assert!(next_match.board_pos().contains(&Pos::new(1, 1)));
        assert!(next_match.board_pos().contains(&Pos::new(6, 6)));
    }

    #[test]
    fn next_match_swap_pieces_match_found_at_second() {
        let mut board = Board::<TestPiece, 15, 16>::new();

        board.set_piece(Pos::new(0, 1), TestPiece::Second);
        board.set_piece(Pos::new(1, 1), TestPiece::Both);
        board.set_piece(Pos::new(8, 8), TestPiece::Second);

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(8, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let mut match_board = MatchBoard::new(board, vec![&pattern]);

        // Empty initial positions from last modified queue
        match_board.next_match();

        match_board.swap(Pos::new(8, 8), Pos::new(6, 6));

        let next_match = match_board.next_match().unwrap();
        assert_eq!(Pos::new(6, 6), next_match.changed_pos());
        assert!(next_match.board_pos().contains(&Pos::new(0, 1)));
        assert!(next_match.board_pos().contains(&Pos::new(1, 1)));
        assert!(next_match.board_pos().contains(&Pos::new(6, 6)));
    }

    #[test]
    fn next_match_swap_self_no_match() {
        let mut board = Board::<TestPiece, 15, 16>::new();

        board.set_piece(Pos::new(0, 1), TestPiece::Second);
        board.set_piece(Pos::new(1, 1), TestPiece::Both);
        board.set_piece(Pos::new(6, 6), TestPiece::Second);

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(8, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let mut match_board = MatchBoard::new(board, vec![&pattern]);

        // Empty initial positions from last modified queue
        match_board.next_match();
        match_board.next_match();
        match_board.next_match();

        match_board.swap(Pos::new(6, 6), Pos::new(6, 6));

        assert!(match_board.next_match().is_none())
    }

    #[test]
    fn next_match_wrong_match_type_none_found() {
        let board = Board::<TestPiece, 15, 16>::new();

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(6, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let mut match_board = MatchBoard::new(board, vec![&pattern]);

        // Empty initial positions from last modified queue
        match_board.next_match();

        match_board.set_piece(Pos::new(0, 1), TestPiece::Second);
        match_board.set_piece(Pos::new(1, 1), TestPiece::Both);
        match_board.set_piece(Pos::new(4, 6), TestPiece::First);

        assert!(match_board.next_match().is_none())
    }

    #[test]
    fn next_match_matches_when_not_all_in_queue() {
        let mut board = Board::<TestPiece, 15, 16>::new();
        board.set_piece(Pos::new(0, 1), TestPiece::Second);
        board.set_piece(Pos::new(1, 1), TestPiece::Both);

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(6, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let mut match_board = MatchBoard::new(board, vec![&pattern]);

        // Empty initial positions from last modified queue
        match_board.next_match();

        match_board.set_piece(Pos::new(4, 6), TestPiece::Second);

        let next_match = match_board.next_match().unwrap();
        assert_eq!(Pos::new(4, 6), next_match.changed_pos());
        assert!(next_match.board_pos().contains(&Pos::new(0, 1)));
        assert!(next_match.board_pos().contains(&Pos::new(1, 1)));
        assert!(next_match.board_pos().contains(&Pos::new(4, 6)));

        assert!(match_board.next_match().is_none());
    }

    #[test]
    fn next_match_matches_when_in_queue_twice() {
        let board = Board::<TestPiece, 15, 16>::new();

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(6, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let mut match_board = MatchBoard::new(board, vec![&pattern]);

        // Empty initial positions from last modified queue
        match_board.next_match();

        match_board.set_piece(Pos::new(0, 1), TestPiece::Second);
        match_board.set_piece(Pos::new(1, 1), TestPiece::Both);
        match_board.set_piece(Pos::new(4, 6), TestPiece::Second);
        match_board.set_piece(Pos::new(0, 1), TestPiece::Both);

        let expected_changed_pos = vec![
            Pos::new(0, 1), Pos::new(1, 1),
            Pos::new(4, 6), Pos::new(0, 1)
        ];
        for i in 0..4 {
            let next_match = match_board.next_match().unwrap();
            assert_eq!(*expected_changed_pos.get(i).unwrap(), next_match.changed_pos());
            assert!(next_match.board_pos().contains(&Pos::new(0, 1)));
            assert!(next_match.board_pos().contains(&Pos::new(1, 1)));
            assert!(next_match.board_pos().contains(&Pos::new(4, 6)));
        }
    }

    #[test]
    fn next_match_never_matches_when_match_overwritten() {
        let board = Board::<TestPiece, 15, 16>::new();

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(6, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let mut match_board = MatchBoard::new(board, vec![&pattern]);

        // Empty initial positions from last modified queue
        match_board.next_match();

        match_board.set_piece(Pos::new(0, 1), TestPiece::Second);
        match_board.set_piece(Pos::new(1, 1), TestPiece::Both);
        match_board.set_piece(Pos::new(4, 6), TestPiece::Second);
        match_board.set_piece(Pos::new(0, 1), TestPiece::First);

        for _ in 0..4 {
            assert!(match_board.next_match().is_none());
        }
    }

    #[test]
    fn next_match_set_pieces_matches_earlier_pattern() {
        let board = Board::<TestPiece, 15, 16>::new();

        let pattern_pos1 = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(6, 8)];
        let pattern_pos2 = vec![
            Pos::new(2, 3), Pos::new(3, 3),
            Pos::new(6, 8), Pos::new(7, 8)
        ];
        let pattern1 = MatchPattern::new(TestMatchType::Second, &pattern_pos1[..]);
        let pattern2 = MatchPattern::new(TestMatchType::Second, &pattern_pos2[..]);

        let mut match_board = MatchBoard::new(board, vec![&pattern1, &pattern2]);

        // Empty initial positions from last modified queue
        match_board.next_match();

        match_board.set_piece(Pos::new(0, 1), TestPiece::Second);
        match_board.set_piece(Pos::new(1, 1), TestPiece::Both);
        match_board.set_piece(Pos::new(4, 6), TestPiece::Second);
        match_board.set_piece(Pos::new(5, 6), TestPiece::Both);

        let expected_changed_pos = vec![Pos::new(0, 1), Pos::new(1, 1), Pos::new(4, 6)];
        for i in 0..3 {
            let next_match = match_board.next_match().unwrap();
            assert_eq!(*expected_changed_pos.get(i).unwrap(), next_match.changed_pos());
            assert!(next_match.board_pos().contains(&Pos::new(0, 1)));
            assert!(next_match.board_pos().contains(&Pos::new(1, 1)));
            assert!(next_match.board_pos().contains(&Pos::new(4, 6)));
            assert!(!next_match.board_pos().contains(&Pos::new(5, 6)));
        }

        let next_match = match_board.next_match().unwrap();
        assert_eq!(Pos::new(5, 6), next_match.changed_pos());
        assert!(next_match.board_pos().contains(&Pos::new(0, 1)));
        assert!(next_match.board_pos().contains(&Pos::new(1, 1)));
        assert!(next_match.board_pos().contains(&Pos::new(4, 6)));
        assert!(next_match.board_pos().contains(&Pos::new(5, 6)));
    }

    #[test]
    fn next_match_end_game_returns_board() {
        let board = Board::<TestPiece, 15, 16>::new();

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(6, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let mut match_board = MatchBoard::new(board, vec![&pattern]);

        // Empty initial positions from last modified queue
        match_board.next_match();

        match_board.set_piece(Pos::new(0, 1), TestPiece::First);
        match_board.set_piece(Pos::new(1, 1), TestPiece::Both);
        match_board.set_piece(Pos::new(4, 6), TestPiece::Second);

        match_board.swap(Pos::new(4, 6), Pos::new(5, 6));

        let final_board = match_board.end_game();

        for x in 0..15 {
            for y in 0..16 {
                if x == 0 && y == 1 {
                    assert_eq!(TestPiece::First, final_board.piece(Pos::new(x, y)));
                } else if x == 1 && y == 1 {
                    assert_eq!(TestPiece::Both, final_board.piece(Pos::new(x, y)));
                } else if x == 5 && y == 6 {
                    assert_eq!(TestPiece::Second, final_board.piece(Pos::new(x, y)));
                } else {
                    assert_eq!(TestPiece::None, final_board.piece(Pos::new(x, y)));
                }
            }
        }
    }
}