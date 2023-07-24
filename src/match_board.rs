use std::collections::{HashSet, VecDeque};
use crate::{BoardState, CloseMatch, Match, MatchPattern, Piece, Pos};

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
    const BOARD_WIDTH: usize,
    const BOARD_HEIGHT: usize
> {
    board: BoardState<P, BOARD_WIDTH, BOARD_HEIGHT>,
    patterns: Vec<&'a MatchPattern<M, BOARD_WIDTH, BOARD_HEIGHT>>,
    matches: VecDeque<Match<'a, M, BOARD_WIDTH, BOARD_HEIGHT>>,
    close_matches: VecDeque<CloseMatch<'a, M, BOARD_WIDTH, BOARD_HEIGHT>>
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
    pub fn new(board: BoardState<P, W, H>, patterns: Vec<&MatchPattern<M, W, H>>) -> MatchBoard<M, P, W, H> {
        let mut match_board = MatchBoard {
            board,
            patterns,
            matches: VecDeque::new(),
            close_matches: VecDeque::new()
        };

        match_board.add_initial_matches();

        match_board
    }

    /// Ends the current game by returning the final board state.
    pub fn end_game(self) -> BoardState<P, W, H> {
        self.board
    }

    /// Gets the type of a piece at a certain position.
    ///
    /// # Arguments
    ///
    /// * `pos` - the position of the piece whose type to find
    pub fn piece(&self, pos: Pos<W, H>) -> P {
        self.board.piece(pos)
    }

    /// Replaces a piece at the given position and returns the previous piece.
    /// The space is marked as needing a match check.
    ///
    /// # Arguments
    ///
    /// * `pos` - the position of the piece to replace
    /// * `piece` - the piece to put at the given position
    pub fn set_piece(&mut self, pos: Pos<W, H>, piece: P) -> P {
        let old_piece = self.board.set_piece(pos, piece);
        self.recompute_matches(pos);
        old_piece
    }

    /// Swap two pieces on the board. The order of two positions provided does not matter.
    /// The space is marked as needing a match check.
    ///
    /// # Arguments
    ///
    /// * `first` - the first position of a piece to swap
    /// * `second` - the second position of a piece to swap
    pub fn swap(&mut self, first: Pos<W, H>, second: Pos<W, H>) {
        if first == second {
            return;
        }

        self.board.swap(first, second);

        self.recompute_matches(first);
        self.recompute_matches(second);
    }

    /// Gets the next match on the board. Matches from pieces that were changed
    /// earlier are returned first. Matches are always based on the current board
    /// state.
    ///
    /// All positions are checked for a match the first time this method is run.
    ///
    /// Pieces that were changed but did not create a match are skipped.
    pub fn next_match(&mut self) -> Option<Match<M, W, H>> {
        self.matches.pop_front()
    }

    /// Gets the next close match on the board. A close match occurs when one piece needs to
    /// change to create a match. However, the missing piece does not have to be adjacent to
    /// the missing position or even on the board. Close matches from pieces that were changed
    /// earlier are returned first. Close matches are always based on the current board
    /// state.
    ///
    /// Close matches are still computed even when their positions are part of a match. However,
    /// a close match will not be returned when a complete match exists for the exact same
    /// positions.
    ///
    /// All positions are checked for a close match the first time this method is run.
    ///
    /// Pieces that were changed but did not create a close match are skipped.
    pub fn next_close_match(&mut self) -> Option<CloseMatch<M, W, H>> {
        self.close_matches.pop_front()
    }

    /// Checks if a given position is inside the board.
    ///
    /// # Arguments
    ///
    /// * `pos` - the position to check
    pub fn is_in_bounds(&self, pos: Pos<W, H>) -> bool {
        pos.x() < W && pos.y() < H
    }


    /// Scans the initial state of the board for matches and close matches.
    fn add_initial_matches(&mut self) {
        for x in 0..W {
            for y in 0..H {
                let possible_match = self.patterns.iter().find_map(|pattern| {
                    self.check_pattern(
                        pattern,
                        Pos::new(x, y)
                    )
                });

                if let Some(initial_match) = possible_match {
                    let is_new_match = initial_match.iter().all(|pos| pos.x() >= x && pos.y() >= y);
                    if is_new_match {
                        self.matches.push_back(initial_match);
                    }
                }

                let possible_close_match = self.patterns.iter().find_map(|pattern| {
                    self.check_close_pattern(
                        pattern,
                        Pos::new(x, y)
                    )
                });

                if let Some(initial_close_match) = possible_close_match {
                    let is_new_match = initial_close_match.iter().all(|pos| pos.x() >= x && pos.y() >= y);
                    if is_new_match {
                        self.close_matches.push_back(initial_close_match);
                    }
                }
            }
        };
    }

    /// Recomputes the current set of matches when a position on the board is changed.
    ///
    /// # Arguments
    ///
    /// * `changed_pos` - the position on the board that changed
    fn recompute_matches(&mut self, changed_pos: Pos<W, H>) {

        // TODO: replace with drain_filter() once it is stable
        self.matches = self.matches.clone().into_iter()
            .filter(|prev_match| !prev_match.contains(changed_pos))
            .collect();

        let possible_new_match = self.patterns.iter().find_map(
            |&pattern| self.check_pattern(
                pattern,
                changed_pos
            )
        );

        if let Some(new_match) = possible_new_match {
            self.matches.push_back(new_match);
        }

        // Both a match and a close match can be added, in case we want to process
        // the close match and not the match for some reason.
        self.close_matches = self.close_matches.clone().into_iter()
            .filter(|prev_match| !prev_match.contains(changed_pos))
            .collect();

        let possible_close_match = self.patterns.iter().find_map(
            |&pattern| self.check_close_pattern(
                pattern,
                changed_pos
            )
        );

        if let Some(new_close_match) = possible_close_match {
            self.close_matches.push_back(new_close_match);
        }
    }

    /// Checks for a pattern that includes a specific position on the board. Looks
    /// for all variants of a pattern (all possible patterns that include the required
    /// position). Returns the positions on the board that correspond to that pattern
    /// if there is a match.
    ///
    /// # Arguments
    ///
    /// * `pattern` - the match pattern to check
    /// * `pos` - the position that must be included in a match
    fn check_pattern<'a>(&self, pattern: &'a MatchPattern<M, W, H>, pos: Pos<W, H>) -> Option<Match<'a, M, W, H>> {
        pattern.iter().find_map(
            |&original| match pos - original {
                Ok(origin) => self.check_variant(pattern, origin),
                Err(_) => None
            }
        ).map(|positions| Match::new(pattern, pos, positions))
    }

    /// Checks for a single variant of a pattern and returns the corresponding positions
    /// on the board if found.
    ///
    /// # Arguments
    ///
    /// * `pattern` - the match pattern to check
    /// * `new_origin` - the origin to use for the pattern positions so that they
    ///                  correspond to actual positions on the board
    fn check_variant(&self, pattern: &MatchPattern<M, W, H>, new_origin: Pos<W, H>) -> Option<HashSet<Pos<W, H>>> {
        let grid_pos = MatchBoard::<M, P, W, H>::change_origin(pattern.iter(), new_origin)?;
        let all_match = grid_pos.iter().all(
            |&pos| self.is_in_bounds(pos)
                && MatchBoard::<M, P, W, H>::matches(pattern.match_type(), self.board.piece(pos))
        );
        match all_match {
            true => Some(grid_pos),
            false => None
        }
    }

    /// Checks for a close match on a pattern that includes a specific position on the board.
    /// Looks for all variants of a pattern (all possible patterns that include the required
    /// position). Returns the positions on the board that correspond to that pattern
    /// if there is a match.
    ///
    /// # Arguments
    ///
    /// * `pattern` - the match pattern to check
    /// * `pos` - the position that must be included in a match
    fn check_close_pattern<'a>(&self, pattern: &'a MatchPattern<M, W, H>, pos: Pos<W, H>) -> Option<CloseMatch<'a, M, W, H>> {
        pattern.iter().find_map(
            |&original| match pos - original {
                Ok(origin) => self.check_close_variant(pattern, origin),
                Err(_) => None
            }
        ).map(|(positions, missing_pos)| CloseMatch::new(pattern, missing_pos, positions))
    }

    /// Checks for a close match of a single variant of a pattern and returns the
    /// corresponding positions on the board if found.
    ///
    /// # Arguments
    ///
    /// * `pattern` - the match pattern to check
    /// * `new_origin` - the origin to use for the pattern positions so that they
    ///                  correspond to actual positions on the board
    fn check_close_variant(&self, pattern: &MatchPattern<M, W, H>, new_origin: Pos<W, H>) -> Option<(HashSet<Pos<W, H>>, Pos<W, H>)> {
        let grid_pos = MatchBoard::<M, P, W, H>::change_origin(pattern.iter(), new_origin)?;

        let (matched, unmatched): (HashSet<Pos<W, H>>, HashSet<Pos<W, H>>) = grid_pos.iter().partition(
            |&&pos| MatchBoard::<M, P, W, H>::matches(pattern.match_type(), self.board.piece(pos))
        );

        match unmatched.len() {
            1 => Some((matched, unmatched.into_iter().next().unwrap())),
            _ => None
        }
    }

    /// Changes the origin of a set of points.
    ///
    /// # Arguments
    ///
    /// * `positions` - the positions to change the origin of
    /// * `origin` - the new origin to use for the positions
    fn change_origin<'a>(positions: impl Iterator<Item=&'a Pos<W, H>>, origin: Pos<W, H>) -> Option<HashSet<Pos<W, H>>> {
        let mut new_positions = HashSet::new();

        for &pos in positions {
            if let Ok(new_pos) = pos + origin {
                new_positions.insert(new_pos);
            } else {
                return None;
            }
        }

        Some(new_positions)
    }

    /// Checks if the given piece has the given match type.
    ///
    /// # Arguments
    ///
    /// * `match_type` - match type to compare to the piece
    /// * `piece` - piece to compare to the match type
    fn matches(match_type: M, piece: P) -> bool {
        let type_piece: P = match_type.into();
        (type_piece & piece) != P::AIR
    }

}

#[cfg(test)]
mod tests {
    use std::ops::BitAnd;
    use crate::{BoardState, MatchBoard, MatchPattern, Piece, Pos};

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
        const AIR: Self = Self::None;
    }

    #[test]
    fn get_piece_in_bounds_returns_piece() {
        let board = BoardState::<TestPiece, 15, 16>::new();

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(6, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let match_board = MatchBoard::new(board, vec![&pattern]);

        assert_eq!(TestPiece::None, match_board.piece(Pos::new(0, 0)));
    }

    #[test]
    fn next_match_no_patterns_none() {
        let mut board: MatchBoard<TestMatchType, TestPiece, 15, 16> = MatchBoard::new(
            BoardState::new(),
            Vec::new()
        );

        board.set_piece(Pos::new(1, 2), TestPiece::First);
        board.set_piece(Pos::new(2, 3), TestPiece::First);
        assert!(board.next_match().is_none());
    }

    #[test]
    fn next_match_checks_initial_board() {
        let mut board = BoardState::<TestPiece, 15, 16>::new();

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
        assert!(next_match.contains(Pos::new(0, 1)));
        assert!(next_match.contains(Pos::new(1, 1)));
        assert!(next_match.contains(Pos::new(4, 6)));
    }

    #[test]
    fn next_match_swap_pieces_match_found_at_first() {
        let mut board = BoardState::<TestPiece, 15, 16>::new();

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
        assert!(next_match.contains(Pos::new(0, 1)));
        assert!(next_match.contains(Pos::new(1, 1)));
        assert!(next_match.contains(Pos::new(6, 6)));
    }

    #[test]
    fn next_match_swap_pieces_match_found_at_second() {
        let mut board = BoardState::<TestPiece, 15, 16>::new();

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
        assert!(next_match.contains(Pos::new(0, 1)));
        assert!(next_match.contains(Pos::new(1, 1)));
        assert!(next_match.contains(Pos::new(6, 6)));
    }

    #[test]
    fn next_match_swap_self_no_match() {
        let mut board = BoardState::<TestPiece, 15, 16>::new();

        board.set_piece(Pos::new(0, 1), TestPiece::Second);
        board.set_piece(Pos::new(1, 1), TestPiece::Both);
        board.set_piece(Pos::new(6, 6), TestPiece::Second);

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(8, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let mut match_board = MatchBoard::new(board, vec![&pattern]);

        // Empty initial positions from last modified queue
        match_board.next_match();

        match_board.swap(Pos::new(6, 6), Pos::new(6, 6));

        assert!(match_board.next_match().is_none())
    }

    #[test]
    fn next_match_wrong_match_type_none_found() {
        let board = BoardState::<TestPiece, 15, 16>::new();

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
        let mut board = BoardState::<TestPiece, 15, 16>::new();
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
        assert!(next_match.contains(Pos::new(0, 1)));
        assert!(next_match.contains(Pos::new(1, 1)));
        assert!(next_match.contains(Pos::new(4, 6)));

        assert!(match_board.next_match().is_none());
    }

    #[test]
    fn next_match_matches_when_changed_twice() {
        let board = BoardState::<TestPiece, 15, 16>::new();

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(6, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let mut match_board = MatchBoard::new(board, vec![&pattern]);

        // Empty initial positions from last modified queue
        match_board.next_match();

        match_board.set_piece(Pos::new(0, 1), TestPiece::Second);
        match_board.set_piece(Pos::new(1, 1), TestPiece::Both);
        match_board.set_piece(Pos::new(4, 6), TestPiece::Second);
        match_board.set_piece(Pos::new(0, 1), TestPiece::Both);

        let next_match = match_board.next_match().unwrap();
        assert_eq!(Pos::new(0, 1), next_match.changed_pos());
        assert!(next_match.contains(Pos::new(0, 1)));
        assert!(next_match.contains(Pos::new(1, 1)));
        assert!(next_match.contains(Pos::new(4, 6)));

        assert!(match_board.next_match().is_none());
    }

    #[test]
    fn next_match_never_matches_when_match_overwritten() {
        let board = BoardState::<TestPiece, 15, 16>::new();

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(6, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let mut match_board = MatchBoard::new(board, vec![&pattern]);

        // Empty initial positions from last modified queue
        match_board.next_match();

        match_board.set_piece(Pos::new(0, 1), TestPiece::Second);
        match_board.set_piece(Pos::new(1, 1), TestPiece::Both);
        match_board.set_piece(Pos::new(4, 6), TestPiece::Second);
        match_board.set_piece(Pos::new(0, 1), TestPiece::First);

        assert!(match_board.next_match().is_none());
    }

    #[test]
    fn next_match_set_pieces_matches_earlier_pattern() {
        let board = BoardState::<TestPiece, 15, 16>::new();

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

        let next_match = match_board.next_match().unwrap();
        assert_eq!(Pos::new(4, 6), next_match.changed_pos());
        assert!(next_match.contains(Pos::new(0, 1)));
        assert!(next_match.contains(Pos::new(1, 1)));
        assert!(next_match.contains(Pos::new(4, 6)));
        assert!(!next_match.contains(Pos::new(5, 6)));

        let next_next_match = match_board.next_match().unwrap();
        assert_eq!(Pos::new(5, 6), next_next_match.changed_pos());
        assert!(next_next_match.contains(Pos::new(0, 1)));
        assert!(next_next_match.contains(Pos::new(1, 1)));
        assert!(next_next_match.contains(Pos::new(4, 6)));
        assert!(next_next_match.contains(Pos::new(5, 6)));
    }

    #[test]
    fn next_close_match_no_patterns_none() {
        let mut board: MatchBoard<TestMatchType, TestPiece, 15, 16> = MatchBoard::new(
            BoardState::new(),
            Vec::new()
        );

        board.set_piece(Pos::new(1, 2), TestPiece::First);
        board.set_piece(Pos::new(2, 3), TestPiece::First);
        assert!(board.next_close_match().is_none());
    }

    #[test]
    fn next_close_match_checks_initial_board() {
        let mut board = BoardState::<TestPiece, 15, 16>::new();

        board.set_piece(Pos::new(0, 1), TestPiece::First);
        board.set_piece(Pos::new(4, 6), TestPiece::First);

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(6, 8)];
        let pattern = MatchPattern::new(TestMatchType::First, &pattern_pos[..]);

        let mut match_board = MatchBoard::new(
            board,
            vec![&pattern]
        );

        let next_match = match_board.next_close_match().unwrap();
        assert_eq!(Pos::new(1, 1), next_match.missing_pos());
        assert!(next_match.contains(Pos::new(0, 1)));
        assert!(!next_match.contains(Pos::new(1, 1)));
        assert!(next_match.contains(Pos::new(4, 6)));
    }

    #[test]
    fn next_close_match_swap_pieces_match_found_at_first() {
        let mut board = BoardState::<TestPiece, 15, 16>::new();

        board.set_piece(Pos::new(0, 1), TestPiece::Second);
        board.set_piece(Pos::new(1, 1), TestPiece::Both);
        board.set_piece(Pos::new(6, 6), TestPiece::Second);

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(8, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let mut match_board = MatchBoard::new(board, vec![&pattern]);

        // Empty initial positions from last modified queue
        match_board.next_close_match();

        match_board.swap(Pos::new(6, 6), Pos::new(8, 8));

        let next_match = match_board.next_close_match().unwrap();
        assert_eq!(Pos::new(6, 6), next_match.missing_pos());
        assert!(next_match.contains(Pos::new(0, 1)));
        assert!(next_match.contains(Pos::new(1, 1)));
        assert!(!next_match.contains(Pos::new(6, 6)));
    }

    #[test]
    fn next_close_match_swap_pieces_match_found_at_second() {
        let mut board = BoardState::<TestPiece, 15, 16>::new();

        board.set_piece(Pos::new(0, 1), TestPiece::Second);
        board.set_piece(Pos::new(1, 1), TestPiece::Both);
        board.set_piece(Pos::new(6, 6), TestPiece::Second);

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(8, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let mut match_board = MatchBoard::new(board, vec![&pattern]);

        // Empty initial positions from last modified queue
        match_board.next_close_match();

        match_board.swap(Pos::new(8, 8), Pos::new(6, 6));

        let next_match = match_board.next_close_match().unwrap();
        assert_eq!(Pos::new(6, 6), next_match.missing_pos());
        assert!(next_match.contains(Pos::new(0, 1)));
        assert!(next_match.contains(Pos::new(1, 1)));
        assert!(!next_match.contains(Pos::new(6, 6)));
    }

    #[test]
    fn next_close_match_swap_self_no_match() {
        let mut board = BoardState::<TestPiece, 15, 16>::new();

        board.set_piece(Pos::new(0, 1), TestPiece::Second);
        board.set_piece(Pos::new(6, 6), TestPiece::Second);

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(8, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let mut match_board = MatchBoard::new(board, vec![&pattern]);

        // Empty initial positions from last modified queue
        match_board.next_close_match();

        match_board.swap(Pos::new(6, 6), Pos::new(6, 6));

        assert!(match_board.next_close_match().is_none())
    }

    #[test]
    fn next_close_match_wrong_match_type_none_found() {
        let board = BoardState::<TestPiece, 15, 16>::new();

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(6, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let mut match_board = MatchBoard::new(board, vec![&pattern]);

        // Empty initial positions from last modified queue
        match_board.next_close_match();

        match_board.set_piece(Pos::new(0, 1), TestPiece::Second);
        match_board.set_piece(Pos::new(4, 6), TestPiece::First);

        assert!(match_board.next_close_match().is_none())
    }

    #[test]
    fn next_close_match_matches_when_not_all_in_queue() {
        let mut board = BoardState::<TestPiece, 15, 16>::new();
        board.set_piece(Pos::new(0, 1), TestPiece::Second);

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(6, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let mut match_board = MatchBoard::new(board, vec![&pattern]);

        // Empty initial positions from last modified queue
        match_board.next_close_match();

        match_board.set_piece(Pos::new(4, 6), TestPiece::Second);

        let next_match = match_board.next_close_match().unwrap();
        assert_eq!(Pos::new(1, 1), next_match.missing_pos());
        assert!(next_match.contains(Pos::new(0, 1)));
        assert!(!next_match.contains(Pos::new(1, 1)));
        assert!(next_match.contains(Pos::new(4, 6)));

        assert!(match_board.next_close_match().is_none());
    }

    #[test]
    fn next_close_match_matches_when_changed_twice() {
        let board = BoardState::<TestPiece, 15, 16>::new();

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(6, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let mut match_board = MatchBoard::new(board, vec![&pattern]);

        // Empty initial positions from last modified queue
        match_board.next_close_match();

        match_board.set_piece(Pos::new(0, 1), TestPiece::Second);
        match_board.set_piece(Pos::new(4, 6), TestPiece::Second);
        match_board.set_piece(Pos::new(0, 1), TestPiece::Both);

        let next_match = match_board.next_close_match().unwrap();
        assert_eq!(Pos::new(1, 1), next_match.missing_pos());
        assert!(next_match.contains(Pos::new(0, 1)));
        assert!(!next_match.contains(Pos::new(1, 1)));
        assert!(next_match.contains(Pos::new(4, 6)));

        assert!(match_board.next_close_match().is_none());
    }

    #[test]
    fn next_close_match_never_matches_when_match_overwritten() {
        let board = BoardState::<TestPiece, 15, 16>::new();

        let pattern_pos = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(6, 8)];
        let pattern = MatchPattern::new(TestMatchType::Second, &pattern_pos[..]);

        let mut match_board = MatchBoard::new(board, vec![&pattern]);

        // Empty initial positions from last modified queue
        match_board.next_close_match();

        match_board.set_piece(Pos::new(0, 1), TestPiece::Second);
        match_board.set_piece(Pos::new(1, 1), TestPiece::Both);
        match_board.set_piece(Pos::new(0, 1), TestPiece::First);

        assert!(match_board.next_close_match().is_none());
    }

    #[test]
    fn next_close_match_set_pieces_matches_earlier_pattern() {
        let board = BoardState::<TestPiece, 15, 16>::new();

        let pattern_pos1 = vec![Pos::new(2, 3), Pos::new(3, 3), Pos::new(6, 8)];
        let pattern_pos2 = vec![
            Pos::new(2, 3), Pos::new(3, 3),
            Pos::new(6, 8), Pos::new(7, 8)
        ];
        let pattern1 = MatchPattern::new(TestMatchType::Second, &pattern_pos1[..]);
        let pattern2 = MatchPattern::new(TestMatchType::Second, &pattern_pos2[..]);

        let mut match_board = MatchBoard::new(board, vec![&pattern1, &pattern2]);

        // Empty initial positions from last modified queue
        match_board.next_close_match();

        match_board.set_piece(Pos::new(0, 1), TestPiece::Second);
        match_board.set_piece(Pos::new(1, 1), TestPiece::Both);
        match_board.set_piece(Pos::new(5, 6), TestPiece::Both);

        let next_match = match_board.next_close_match().unwrap();
        assert_eq!(Pos::new(4, 6), next_match.missing_pos());
        assert!(next_match.contains(Pos::new(0, 1)));
        assert!(next_match.contains(Pos::new(1, 1)));
        assert!(!next_match.contains(Pos::new(4, 6)));
        assert!(!next_match.contains(Pos::new(5, 6)));

        let next_next_match = match_board.next_close_match().unwrap();
        assert_eq!(Pos::new(2, 1), next_next_match.missing_pos());
        assert!(!next_next_match.contains(Pos::new(0, 1)));
        assert!(next_next_match.contains(Pos::new(1, 1)));
        assert!(!next_next_match.contains(Pos::new(4, 6)));
        assert!(next_next_match.contains(Pos::new(5, 6)));
    }

    #[test]
    fn end_game_returns_board() {
        let board = BoardState::<TestPiece, 15, 16>::new();

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