use std::collections::{HashMap, VecDeque};
use crate::piece::{Piece, Direction};
use crate::position::Pos;
use crate::matching::{MatchPattern, Match};

/// Contains zero or many and represents the current state
/// of the game pieces.
///
/// By default, the board is empty. It has no fixed bounds.
///
/// Users are responsible for updating the board state at the
/// start of a game and after each match.
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
///
/// Swap rules define which pieces can be changed. By default, the
/// only swap rule in place is that pieces marked unmovable in a
/// direction cannot be moved any amount in that direction. **This
/// means that pieces further than one space away can be swapped by
/// default.**
///
/// The board's lack of default restrictions allows games to implement
/// their own unique or non-standard rules.
pub struct Board {
    patterns: Vec<MatchPattern>,
    swap_rules: Vec<Box<dyn Fn(&Board, Pos, Pos) -> bool>>,
    pieces: HashMap<Pos, Piece>,
    last_changed: VecDeque<Pos>
}

impl Board {

    /// Creates a new board.
    ///
    /// # Arguments
    ///
    /// * `patterns` - the match patterns the board should use to detect matches. If
    ///                two patterns have the same rank, no order is guaranteed.
    /// * `swap_rules` - the swap rules that define whether two pieces can be swapped.
    ///                  If any rule returns false for two positions, the pieces are
    ///                  not swapped, and the swap method returns false. These rules
    ///                  are executed in the order provided after the default rule,
    ///                  so less expensive calculations should be done in earlier rules.
    pub fn new(mut patterns: Vec<MatchPattern>,
               mut swap_rules: Vec<Box<dyn Fn(&Board, Pos, Pos) -> bool>>) -> Board {
        patterns.sort_by(|a, b| b.get_rank().cmp(&a.get_rank()));
        swap_rules.insert(0, Box::from(Board::are_pieces_movable));
        Board { patterns, swap_rules, pieces: HashMap::new(), last_changed: VecDeque::new() }
    }

    /// Gets a piece at the given position on the board if one is present.
    ///
    /// # Arguments
    ///
    /// * `pos` - position of the piece to get
    pub fn get_piece(&self, pos: Pos) -> Option<&Piece> {
        self.pieces.get(&pos)
    }

    /// Attempts to swap two pieces on the board. If any swap rule is broken (i.e. it
    /// results false), then the pieces will not be swapped, and this method will
    /// return true.
    ///
    /// If the swap is successful, both swapped positions will be marked for a match check.
    ///
    /// Swapping a piece in a direction in which it is marked unmovable is automatically
    /// a violation of the swap rules.
    ///
    /// Swapping with a piece that is not present is considered valid. The existing piece
    /// moves into the empty space while the other space is cleared. It is also valid to
    /// swap a piece with itself, though this has no effect on the board besides marking
    /// the piece for a match check.
    ///
    /// The order of two positions provided does not matter.
    ///
    /// # Arguments
    ///
    /// * `first` - the first position of a piece to swap
    /// * `second` - the second position of a piece to swap
    #[must_use]
    pub fn swap_pieces(&mut self, first: Pos, second: Pos) -> bool {
        if !self.swap_rules.iter().all(|rule| rule(self, first, second)) {
            return false;
        }

        self.last_changed.push_back(first);
        self.last_changed.push_back(second);

        let original_first_piece = self.pieces.remove(&first);
        let original_second_piece = self.pieces.remove(&second);

        if let Some(piece) = original_first_piece {
            self.pieces.insert(second, piece);
        }

        if let Some(piece) = original_second_piece {
            self.pieces.insert(first, piece);
        }

        true
    }

    /// Replaces a piece at the given position and returns the previous piece
    /// if one was present. The space is marked as needing a match check. Swap
    /// rules do not apply and the replacement is always successful.
    ///
    /// # Arguments
    ///
    /// * `pos` - the position of the piece to replace
    /// * `piece` - the piece to put at the given position
    pub fn set_piece(&mut self, pos: Pos, piece: Piece) -> Option<Piece> {
        self.last_changed.push_back(pos);
        self.pieces.insert(pos, piece)
    }

    /// Gets the next match on the board. Matches from pieces that were changed
    /// earlier are returned first. Matches are always based on the current board
    /// state, not the board state when the match occurred.
    ///
    /// Pieces that were changed but did not create a match are skipped.
    ///
    /// Regardless of whether a match is found, each piece is unmarked for a
    /// match check, unless it has been marked multiple times.
    pub fn next_match(&mut self) -> Option<Match> {
        let mut next_pos;
        let mut next_match = None;

        while next_match.is_none() {
            next_pos = self.last_changed.pop_front()?;
            next_match = self.patterns.iter().find_map(|pattern| {
                let positions = self.find_match(pattern, next_pos)?;
                Some(Match::new(pattern, next_pos, positions))
            });
        }

        next_match
    }

    /// Looks for a match for a specific pattern and changed position. All variants
    /// of the match that contain the changed position are checked.
    ///
    /// # Arguments
    ///
    /// * `pattern` - the match pattern to look for
    /// * `pos` - the changed position to check around
    fn find_match(&self, pattern: &MatchPattern, pos: Pos) -> Option<HashMap<Pos, Pos>> {
        pattern.get_spaces().keys().into_iter().find_map(|&original|
            self.check_variant_match(pattern, pos - original)
        )
    }

    /// Checks a specific variant of a match pattern, that is, with the changed position
    /// at a specific place in the pattern. A space with no piece is automatically not a
    /// match.
    ///
    /// # Arguments
    ///
    /// * `pattern` - the pattern to check if it matches
    /// * `new_origin` - the origin to use for the match pattern so that its positions
    ///                  correspond to positions on the board
    fn check_variant_match(&self, pattern: &MatchPattern, new_origin: Pos) -> Option<HashMap<Pos, Pos>> {
        let original_to_board_pos = Board::change_origin(pattern, new_origin);
        let is_match = original_to_board_pos.iter().all(|(original_pos, board_pos)|
            match self.get_piece(*board_pos) {
                None => false,
                Some(piece) => piece.get_type() == *pattern.get_spaces().get(original_pos)
                    .expect("Known piece wasn't found in pattern!")
            }
        );

        match is_match {
            true => Some(original_to_board_pos),
            false => None
        }
    }

    /// Maps the positions of a match pattern to have a new origin.
    ///
    /// # Arguments
    ///
    /// * `pattern` - the pattern whose positions to convert
    /// * `origin` - the new origin to use for the pattern's positions
    fn change_origin(pattern: &MatchPattern, origin: Pos) -> HashMap<Pos, Pos> {
        let mut original_positions: Vec<Pos> = pattern.get_spaces().keys().map(|&pos| pos).collect();
        let mut changed_positions: Vec<Pos> = original_positions.iter().map(
            |&original| original + origin
        ).collect();

        original_positions.drain(..).zip(changed_positions.drain(..)).collect()
    }

    /// Checks if the pieces at two positions on the board are both movable in the
    /// direction in which they would be swapped.
    ///
    /// # Arguments
    ///
    /// * `first` - the position of the first piece to check
    /// * `second` - the position of the second piece to check
    fn are_pieces_movable(&self, first: Pos, second: Pos) -> bool {
        let is_first_movable = match self.get_piece(first) {
            None => true,
            Some(piece) => Board::is_movable(first, second, piece)
        };

        let is_second_movable = match self.get_piece(second) {
            None => true,
            Some(piece) => Board::is_movable(second, first, piece)
        };

        is_first_movable && is_second_movable
    }

    /// Checks if a piece is movable vertically and horizontally.
    ///
    /// # Arguments
    ///
    /// * `from` - the current position of the piece
    /// * `to` - the position where the piece will be moved
    /// * `piece` - the at the "from" position
    fn is_movable(from: Pos, to: Pos, piece: &Piece) -> bool {
        Board::is_vertically_movable(from, to, piece)
            && Board::is_horizontally_movable(from, to, piece)
    }

    /// Checks if a piece is vertically movable from one position to another.
    /// If there is no vertical change between the two positions, the piece
    /// is considered movable.
    ///
    /// # Arguments
    ///
    /// * `from` - the current position of the piece
    /// * `to` - the position where the piece will be moved
    /// * `piece` - the at the "from" position
    fn is_vertically_movable(from: Pos, to: Pos, piece: &Piece) -> bool {
        let vertical_change = to.get_y() - from.get_y();
        if vertical_change > 0 {
            return piece.is_movable(Direction::North);
        } else if vertical_change < 0 {
            return piece.is_movable(Direction::South);
        }

        true
    }

    /// Checks if a piece is horizontally movable from one position to another.
    /// If there is no horizontal change between the two positions, the piece
    /// is considered movable.
    ///
    /// # Arguments
    ///
    /// * `from` - the current position of the piece
    /// * `to` - the position where the piece will be moved
    /// * `piece` - the at the "from" position
    fn is_horizontally_movable(from: Pos, to: Pos, piece: &Piece) -> bool {
        let horizontal_change = to.get_x() - from.get_x();
        if horizontal_change > 0 {
            return piece.is_movable(Direction::East);
        } else if horizontal_change < 0 {
            return piece.is_movable(Direction::West);
        }

        true
    }

}

#[cfg(test)]
mod tests {
    use crate::board::Board;
    use crate::position::Pos;
    use crate::piece::{PieceType, Piece, Direction};
    use std::collections::HashMap;
    use crate::matching::MatchPattern;

    #[test]
    fn swap_adjacent_all_rules_passed_swapped() {
        let mut board = Board::new(Vec::new(), vec![
            Box::new(|_, _, _| true),
            Box::new(|_, _, _| true)
        ]);
        let type1 = PieceType::new("first");
        let type2 = PieceType::new("second");
        let piece1 = Piece::new(type1);
        let piece2 = Piece::new(type2);

        board.set_piece(Pos::new(1, 2), piece1);
        board.set_piece(Pos::new(1, 3), piece2);

        assert!(board.swap_pieces(Pos::new(1, 2), Pos::new(1, 3)));
        assert_eq!(PieceType::new("second"), board.get_piece(Pos::new(1, 2)).unwrap().get_type());
        assert_eq!(PieceType::new("first"), board.get_piece(Pos::new(1, 3)).unwrap().get_type());
    }

    #[test]
    fn swap_non_adjacent_all_rules_passed_swapped() {
        let mut board = Board::new(Vec::new(), vec![
            Box::new(|_, _, _| true),
            Box::new(|_, _, _| true)
        ]);
        let type1 = PieceType::new("first");
        let type2 = PieceType::new("second");
        let piece1 = Piece::new(type1);
        let piece2 = Piece::new(type2);

        board.set_piece(Pos::new(1, 2), piece1);
        board.set_piece(Pos::new(20, 31), piece2);

        assert!(board.swap_pieces(Pos::new(1, 2), Pos::new(20, 31)));
        assert_eq!(PieceType::new("second"), board.get_piece(Pos::new(1, 2)).unwrap().get_type());
        assert_eq!(PieceType::new("first"), board.get_piece(Pos::new(20, 31)).unwrap().get_type());
    }

    #[test]
    fn swap_rules_violated_not_swapped() {
        let mut board = Board::new(Vec::new(), vec![
            Box::new(|_, _, _| true),
            Box::new(|_, _, _| false)
        ]);
        let type1 = PieceType::new("first");
        let type2 = PieceType::new("second");
        let piece1 = Piece::new(type1);
        let piece2 = Piece::new(type2);

        board.set_piece(Pos::new(1, 2), piece1);
        board.set_piece(Pos::new(1, 3), piece2);

        assert!(!board.swap_pieces(Pos::new(1, 2), Pos::new(1, 3)));
        assert_eq!(PieceType::new("first"), board.get_piece(Pos::new(1, 2)).unwrap().get_type());
        assert_eq!(PieceType::new("second"), board.get_piece(Pos::new(1, 3)).unwrap().get_type());
    }

    #[test]
    fn swap_rules_violated_short_circuits() {
        let mut board = Board::new(Vec::new(), vec![
            Box::new(|_, _, _| false),
            Box::new(|_, _, _| { panic!("Should short circuit before this") })
        ]);
        let type1 = PieceType::new("first");
        let type2 = PieceType::new("second");
        let piece1 = Piece::new(type1);
        let piece2 = Piece::new(type2);

        board.set_piece(Pos::new(1, 2), piece1);
        board.set_piece(Pos::new(1, 3), piece2);

        assert!(!board.swap_pieces(Pos::new(1, 2), Pos::new(1, 3)));
    }

    #[test]
    fn swap_empty_all_rules_passed_swapped() {
        let mut board = Board::new(Vec::new(), vec![
            Box::new(|_, _, _| true),
            Box::new(|_, _, _| true)
        ]);
        let type1 = PieceType::new("first");
        let piece1 = Piece::new(type1);

        board.set_piece(Pos::new(1, 2), piece1);

        assert!(board.swap_pieces(Pos::new(1, 2), Pos::new(1, 3)));
        assert!(board.get_piece(Pos::new(1, 2)).is_none());
        assert_eq!(PieceType::new("first"), board.get_piece(Pos::new(1, 3)).unwrap().get_type());
    }

    #[test]
    fn swap_self_all_rules_passed_swapped() {
        let mut board = Board::new(Vec::new(), vec![
            Box::new(|_, _, _| true),
            Box::new(|_, _, _| true)
        ]);
        let type1 = PieceType::new("first");
        let piece1 = Piece::new(type1);

        board.set_piece(Pos::new(1, 2), piece1);

        assert!(board.swap_pieces(Pos::new(1, 2), Pos::new(1, 2)));
        assert_eq!(PieceType::new("first"), board.get_piece(Pos::new(1, 2)).unwrap().get_type());
    }

    #[test]
    fn swap_same_vertical_not_vertically_movable_swapped() {
        let mut board = Board::new(Vec::new(), vec![
            Box::new(|_, _, _| true),
            Box::new(|_, _, _| true)
        ]);
        let type1 = PieceType::new("first");
        let type2 = PieceType::new("second");
        let mut piece1 = Piece::new(type1);
        piece1.make_unmovable(Direction::North);
        piece1.make_unmovable(Direction::South);

        let piece2 = Piece::new(type2);

        board.set_piece(Pos::new(1, 2), piece1);
        board.set_piece(Pos::new(2, 2), piece2);

        assert!(board.swap_pieces(Pos::new(1, 2), Pos::new(2, 2)));
        assert_eq!(PieceType::new("second"), board.get_piece(Pos::new(1, 2)).unwrap().get_type());
        assert_eq!(PieceType::new("first"), board.get_piece(Pos::new(2, 2)).unwrap().get_type());
    }

    #[test]
    fn swap_same_horizontal_not_horizontally_movable_swapped() {
        let mut board = Board::new(Vec::new(), vec![
            Box::new(|_, _, _| true),
            Box::new(|_, _, _| true)
        ]);
        let type1 = PieceType::new("first");
        let type2 = PieceType::new("second");
        let mut piece1 = Piece::new(type1);
        piece1.make_unmovable(Direction::East);
        piece1.make_unmovable(Direction::West);

        let piece2 = Piece::new(type2);

        board.set_piece(Pos::new(1, 2), piece1);
        board.set_piece(Pos::new(1, 3), piece2);

        assert!(board.swap_pieces(Pos::new(1, 2), Pos::new(1, 3)));
        assert_eq!(PieceType::new("second"), board.get_piece(Pos::new(1, 2)).unwrap().get_type());
        assert_eq!(PieceType::new("first"), board.get_piece(Pos::new(1, 3)).unwrap().get_type());
    }

    #[test]
    fn swap_north_not_movable_north_not_swapped() {
        let mut board = Board::new(Vec::new(), vec![
            Box::new(|_, _, _| true),
            Box::new(|_, _, _| true)
        ]);
        let type1 = PieceType::new("first");
        let type2 = PieceType::new("second");
        let mut piece1 = Piece::new(type1);
        piece1.make_unmovable(Direction::North);

        let piece2 = Piece::new(type2);

        board.set_piece(Pos::new(1, 2), piece1);
        board.set_piece(Pos::new(1, 3), piece2);

        assert!(!board.swap_pieces(Pos::new(1, 2), Pos::new(1, 3)));
        assert_eq!(PieceType::new("first"), board.get_piece(Pos::new(1, 2)).unwrap().get_type());
        assert_eq!(PieceType::new("second"), board.get_piece(Pos::new(1, 3)).unwrap().get_type());
    }

    #[test]
    fn swap_south_not_movable_south_not_swapped() {
        let mut board = Board::new(Vec::new(), vec![
            Box::new(|_, _, _| true),
            Box::new(|_, _, _| true)
        ]);
        let type1 = PieceType::new("first");
        let type2 = PieceType::new("second");
        let mut piece1 = Piece::new(type1);
        piece1.make_unmovable(Direction::South);

        let piece2 = Piece::new(type2);

        board.set_piece(Pos::new(1, 2), piece1);
        board.set_piece(Pos::new(1, -3), piece2);

        assert!(!board.swap_pieces(Pos::new(1, 2), Pos::new(1, -3)));
        assert_eq!(PieceType::new("first"), board.get_piece(Pos::new(1, 2)).unwrap().get_type());
        assert_eq!(PieceType::new("second"), board.get_piece(Pos::new(1, -3)).unwrap().get_type());
    }

    #[test]
    fn swap_east_not_movable_east_not_swapped() {
        let mut board = Board::new(Vec::new(), vec![
            Box::new(|_, _, _| true),
            Box::new(|_, _, _| true)
        ]);
        let type1 = PieceType::new("first");
        let type2 = PieceType::new("second");
        let mut piece1 = Piece::new(type1);
        piece1.make_unmovable(Direction::East);

        let piece2 = Piece::new(type2);

        board.set_piece(Pos::new(1, 2), piece1);
        board.set_piece(Pos::new(2, 3), piece2);

        assert!(!board.swap_pieces(Pos::new(1, 2), Pos::new(2, 3)));
        assert_eq!(PieceType::new("first"), board.get_piece(Pos::new(1, 2)).unwrap().get_type());
        assert_eq!(PieceType::new("second"), board.get_piece(Pos::new(2, 3)).unwrap().get_type());
    }

    #[test]
    fn swap_west_not_movable_west_not_swapped() {
        let mut board = Board::new(Vec::new(), vec![
            Box::new(|_, _, _| true),
            Box::new(|_, _, _| true)
        ]);
        let type1 = PieceType::new("first");
        let type2 = PieceType::new("second");
        let mut piece1 = Piece::new(type1);
        piece1.make_unmovable(Direction::West);

        let piece2 = Piece::new(type2);

        board.set_piece(Pos::new(1, 2), piece1);
        board.set_piece(Pos::new(-2, 3), piece2);

        assert!(!board.swap_pieces(Pos::new(1, 2), Pos::new(-2, 3)));
        assert_eq!(PieceType::new("first"), board.get_piece(Pos::new(1, 2)).unwrap().get_type());
        assert_eq!(PieceType::new("second"), board.get_piece(Pos::new(-2, 3)).unwrap().get_type());
    }

    #[test]
    fn set_piece_not_present_none_returned() {
        let mut board = Board::new(Vec::new(), vec![
            Box::new(|_, _, _| true),
            Box::new(|_, _, _| true)
        ]);
        let type1 = PieceType::new("first");
        let piece1 = Piece::new(type1);

        assert!(board.set_piece(Pos::new(1, 2), piece1).is_none());
        assert_eq!(PieceType::new("first"), board.get_piece(Pos::new(1, 2)).unwrap().get_type());
    }

    #[test]
    fn set_piece_present_old_piece_returned() {
        let mut board = Board::new(Vec::new(), vec![
            Box::new(|_, _, _| true),
            Box::new(|_, _, _| true)
        ]);
        let type1 = PieceType::new("first");
        let type2 = PieceType::new("second");
        let piece1 = Piece::new(type1);
        let piece2 = Piece::new(type2);

        board.set_piece(Pos::new(1, 2), piece1);
        assert_eq!(PieceType::new("first"), board.set_piece(Pos::new(1, 2), piece2).unwrap().get_type());
        assert_eq!(PieceType::new("second"),
                   board.get_piece(Pos::new(1, 2)).unwrap().get_type());
    }

    #[test]
    fn next_match_no_patterns_none() {
        let mut board = Board::new(Vec::new(), vec![
            Box::new(|_, _, _| true),
            Box::new(|_, _, _| true)
        ]);
        let type1 = PieceType::new("first");
        let piece1 = Piece::new(type1);
        let piece2 = Piece::new(type1);

        board.set_piece(Pos::new(1, 2), piece1);
        board.set_piece(Pos::new(2, 3), piece2);
        assert!(board.next_match().is_none());
    }

    #[test]
    fn next_match_set_pieces_match_found() {
        let mut pattern_pos = HashMap::new();
        pattern_pos.insert(Pos::new(2, 3), PieceType::new("first"));
        pattern_pos.insert(Pos::new(3, 3), PieceType::new("first"));
        pattern_pos.insert(Pos::new(-2, -2), PieceType::new("first"));

        let mut board = Board::new(vec![MatchPattern::new(pattern_pos, 1)], Vec::new());
        let type1 = PieceType::new("first");
        let type2 = PieceType::new("first");
        let type3 = PieceType::new("first");
        let piece1 = Piece::new(type1);
        let piece2 = Piece::new(type2);
        let piece3 = Piece::new(type3);

        board.set_piece(Pos::new(0, 1), piece1);
        board.set_piece(Pos::new(1, 1), piece2);
        board.set_piece(Pos::new(-4, -4), piece3);

        let next_match = board.next_match().unwrap();
        assert_eq!(Pos::new(0, 1), next_match.get_changed_pos());
        assert_eq!(Pos::new(0, 1), next_match.convert_to_board_pos(Pos::new(2, 3)));
        assert_eq!(Pos::new(1, 1), next_match.convert_to_board_pos(Pos::new(3, 3)));
        assert_eq!(Pos::new(-4, -4), next_match.convert_to_board_pos(Pos::new(-2, -2)));
    }

    #[test]
    fn next_match_swap_pieces_match_found() {
        let mut pattern_pos = HashMap::new();
        pattern_pos.insert(Pos::new(2, 3), PieceType::new("first"));
        pattern_pos.insert(Pos::new(3, 3), PieceType::new("first"));
        pattern_pos.insert(Pos::new(-2, -2), PieceType::new("first"));

        let mut board = Board::new(vec![MatchPattern::new(pattern_pos, 1)], Vec::new());
        let type1 = PieceType::new("first");
        let type2 = PieceType::new("first");
        let type3 = PieceType::new("first");
        let piece1 = Piece::new(type1);
        let piece2 = Piece::new(type2);
        let piece3 = Piece::new(type3);

        board.set_piece(Pos::new(0, 1), piece1);
        board.set_piece(Pos::new(1, 1), piece2);
        board.set_piece(Pos::new(-3, -3), piece3);
        board.next_match();
        board.next_match();
        board.next_match();

        assert!(board.swap_pieces(Pos::new(-3, -3), Pos::new(-4, -4)));

        let next_match = board.next_match().unwrap();
        assert_eq!(Pos::new(-4, -4), next_match.get_changed_pos());
        assert_eq!(Pos::new(0, 1), next_match.convert_to_board_pos(Pos::new(2, 3)));
        assert_eq!(Pos::new(1, 1), next_match.convert_to_board_pos(Pos::new(3, 3)));
        assert_eq!(Pos::new(-4, -4), next_match.convert_to_board_pos(Pos::new(-2, -2)));
    }

    #[test]
    fn next_match_matches_all_variants() {
        let piece_type = PieceType::new("first");
        let mut pattern_pos = HashMap::new();
        pattern_pos.insert(Pos::new(2, 2), piece_type);
        pattern_pos.insert(Pos::new(3, 3), piece_type);
        pattern_pos.insert(Pos::new(4, 4), piece_type);

        let mut board = Board::new(vec![MatchPattern::new(pattern_pos, 1)], Vec::new());
        let piece1 = Piece::new(piece_type);
        let piece2 = Piece::new(piece_type);
        let piece3 = Piece::new(piece_type);

        board.set_piece(Pos::new(0, 0), piece1);
        board.set_piece(Pos::new(1, 1), piece2);
        board.set_piece(Pos::new(2, 2), piece3);

        let next_match1 = board.next_match().unwrap();
        assert_eq!(Pos::new(0, 0), next_match1.get_changed_pos());
        assert_eq!(Pos::new(0, 0), next_match1.convert_to_board_pos(Pos::new(2, 2)));
        assert_eq!(Pos::new(1, 1), next_match1.convert_to_board_pos(Pos::new(3, 3)));
        assert_eq!(Pos::new(2, 2), next_match1.convert_to_board_pos(Pos::new(4, 4)));

        let next_match2 = board.next_match().unwrap();
        assert_eq!(Pos::new(1, 1), next_match2.get_changed_pos());
        assert_eq!(Pos::new(0, 0), next_match2.convert_to_board_pos(Pos::new(2, 2)));
        assert_eq!(Pos::new(1, 1), next_match2.convert_to_board_pos(Pos::new(3, 3)));
        assert_eq!(Pos::new(2, 2), next_match2.convert_to_board_pos(Pos::new(4, 4)));

        let next_match3 = board.next_match().unwrap();
        assert_eq!(Pos::new(2, 2), next_match3.get_changed_pos());
        assert_eq!(Pos::new(0, 0), next_match3.convert_to_board_pos(Pos::new(2, 2)));
        assert_eq!(Pos::new(1, 1), next_match3.convert_to_board_pos(Pos::new(3, 3)));
        assert_eq!(Pos::new(2, 2), next_match3.convert_to_board_pos(Pos::new(4, 4)));
    }

    #[test]
    fn next_match_matches_different_types() {
        let type1 = PieceType::new("first");
        let type2 = PieceType::new("second");
        let type3 = PieceType::new("third");
        let mut pattern_pos = HashMap::new();
        pattern_pos.insert(Pos::new(2, 2), type1);
        pattern_pos.insert(Pos::new(3, 3), type2);
        pattern_pos.insert(Pos::new(4, 4), type3);

        let mut board = Board::new(vec![MatchPattern::new(pattern_pos, 1)], Vec::new());
        let piece1 = Piece::new(type1);
        let piece2 = Piece::new(type2);
        let piece3 = Piece::new(type3);

        board.set_piece(Pos::new(0, 0), piece1);
        board.set_piece(Pos::new(1, 1), piece2);
        board.set_piece(Pos::new(2, 2), piece3);

        let next_match1 = board.next_match().unwrap();
        assert_eq!(Pos::new(0, 0), next_match1.get_changed_pos());
        assert_eq!(Pos::new(0, 0), next_match1.convert_to_board_pos(Pos::new(2, 2)));
        assert_eq!(Pos::new(1, 1), next_match1.convert_to_board_pos(Pos::new(3, 3)));
        assert_eq!(Pos::new(2, 2), next_match1.convert_to_board_pos(Pos::new(4, 4)));
    }

    #[test]
    fn next_match_does_not_match_wrong_types() {
        let type1 = PieceType::new("first");
        let type2 = PieceType::new("second");
        let type3 = PieceType::new("third");
        let mut pattern_pos = HashMap::new();
        pattern_pos.insert(Pos::new(2, 2), type1);
        pattern_pos.insert(Pos::new(3, 3), type2);
        pattern_pos.insert(Pos::new(4, 4), type3);

        let mut board = Board::new(vec![MatchPattern::new(pattern_pos, 1)], Vec::new());
        let piece1 = Piece::new(type1);
        let piece2 = Piece::new(type3);
        let piece3 = Piece::new(type2);

        board.set_piece(Pos::new(0, 0), piece1);
        board.set_piece(Pos::new(1, 1), piece2);
        board.set_piece(Pos::new(2, 2), piece3);

        assert!(board.next_match().is_none());
        assert!(board.next_match().is_none());
        assert!(board.next_match().is_none());
    }

    #[test]
    fn next_match_matches_when_not_all_in_queue() {
        let type1 = PieceType::new("first");
        let type2 = PieceType::new("first");
        let type3 = PieceType::new("first");
        let mut pattern_pos = HashMap::new();
        pattern_pos.insert(Pos::new(2, 2), type1);
        pattern_pos.insert(Pos::new(3, 3), type2);
        pattern_pos.insert(Pos::new(4, 4), type3);

        let mut board = Board::new(vec![MatchPattern::new(pattern_pos, 1)], Vec::new());
        let piece1 = Piece::new(type1);
        let piece2 = Piece::new(type2);
        let piece3 = Piece::new(type3);

        board.set_piece(Pos::new(0, 0), piece1);
        board.set_piece(Pos::new(1, 1), piece2);

        board.next_match();
        board.next_match();

        board.set_piece(Pos::new(2, 2), piece3);

        let next_match = board.next_match().unwrap();
        assert_eq!(Pos::new(2, 2), next_match.get_changed_pos());
        assert_eq!(Pos::new(0, 0), next_match.convert_to_board_pos(Pos::new(2, 2)));
        assert_eq!(Pos::new(1, 1), next_match.convert_to_board_pos(Pos::new(3, 3)));
        assert_eq!(Pos::new(2, 2), next_match.convert_to_board_pos(Pos::new(4, 4)));
    }

    #[test]
    fn next_match_board_state_changed_after_match_still_matches() {
        let type1 = PieceType::new("first");
        let type2 = PieceType::new("first");
        let type3 = PieceType::new("first");
        let mut pattern_pos = HashMap::new();
        pattern_pos.insert(Pos::new(2, 2), type1);
        pattern_pos.insert(Pos::new(3, 3), type2);
        pattern_pos.insert(Pos::new(4, 4), type3);

        let mut board = Board::new(vec![MatchPattern::new(pattern_pos, 1)], Vec::new());
        let piece1 = Piece::new(type1);
        let piece2 = Piece::new(type2);
        let piece3 = Piece::new(type3);
        let piece4 = Piece::new(type1);

        board.set_piece(Pos::new(0, 0), piece1);
        board.set_piece(Pos::new(1, 1), piece2);

        board.next_match();
        board.next_match();

        board.set_piece(Pos::new(2, 2), piece3);
        board.set_piece(Pos::new(2, 3), piece4);

        let next_match = board.next_match().unwrap();
        assert_eq!(Pos::new(2, 2), next_match.get_changed_pos());
        assert_eq!(Pos::new(0, 0), next_match.convert_to_board_pos(Pos::new(2, 2)));
        assert_eq!(Pos::new(1, 1), next_match.convert_to_board_pos(Pos::new(3, 3)));
        assert_eq!(Pos::new(2, 2), next_match.convert_to_board_pos(Pos::new(4, 4)));
    }

    #[test]
    fn next_match_match_overwritten_does_not_match() {
        let type1 = PieceType::new("first");
        let type2 = PieceType::new("first");
        let type3 = PieceType::new("first");
        let mut pattern_pos = HashMap::new();
        pattern_pos.insert(Pos::new(2, 2), type1);
        pattern_pos.insert(Pos::new(3, 3), type2);
        pattern_pos.insert(Pos::new(4, 4), type3);

        let mut board = Board::new(vec![MatchPattern::new(pattern_pos, 1)], Vec::new());
        let piece1 = Piece::new(type1);
        let piece2 = Piece::new(type2);
        let piece3 = Piece::new(type3);
        let piece4 = Piece::new(type1);

        board.set_piece(Pos::new(0, 0), piece1);
        board.set_piece(Pos::new(1, 1), piece2);

        board.next_match();
        board.next_match();

        board.set_piece(Pos::new(2, 2), piece3);
        assert!(board.swap_pieces(Pos::new(2, 2), Pos::new(2, 3)));
        assert!(board.next_match().is_none());
    }

    #[test]
    fn next_match_position_in_queue_twice_matches_twice() {
        let piece_type = PieceType::new("first");
        let mut pattern_pos = HashMap::new();
        pattern_pos.insert(Pos::new(2, 2), piece_type);
        pattern_pos.insert(Pos::new(3, 3), piece_type);
        pattern_pos.insert(Pos::new(4, 4), piece_type);

        let mut board = Board::new(vec![MatchPattern::new(pattern_pos, 1)], Vec::new());
        let piece1 = Piece::new(piece_type);
        let piece2 = Piece::new(piece_type);
        let piece3 = Piece::new(piece_type);
        let piece4 = Piece::new(piece_type);

        board.set_piece(Pos::new(0, 0), piece1);
        board.set_piece(Pos::new(1, 1), piece2);

        board.next_match();
        board.next_match();

        board.set_piece(Pos::new(2, 2), piece3);
        board.set_piece(Pos::new(2, 2), piece4);

        let next_match = board.next_match().unwrap();
        assert_eq!(Pos::new(2, 2), next_match.get_changed_pos());
        assert_eq!(Pos::new(0, 0), next_match.convert_to_board_pos(Pos::new(2, 2)));
        assert_eq!(Pos::new(1, 1), next_match.convert_to_board_pos(Pos::new(3, 3)));
        assert_eq!(Pos::new(2, 2), next_match.convert_to_board_pos(Pos::new(4, 4)));
    }

    #[test]
    fn next_match_two_patterns_same_rank_matching_picked() {
        let type1 = PieceType::new("first");
        let type2 = PieceType::new("second");

        let mut pattern_pos1 = HashMap::new();
        pattern_pos1.insert(Pos::new(2, 2), type2);
        pattern_pos1.insert(Pos::new(3, 3), type2);
        pattern_pos1.insert(Pos::new(4, 4), type2);

        let mut pattern_pos2 = HashMap::new();
        pattern_pos2.insert(Pos::new(2, 2), type1);
        pattern_pos2.insert(Pos::new(3, 3), type1);
        pattern_pos2.insert(Pos::new(4, 4), type1);

        let mut board = Board::new(vec![
            MatchPattern::new(pattern_pos1, 1),
            MatchPattern::new(pattern_pos2, 1)
        ], Vec::new());
        let piece1 = Piece::new(type1);
        let piece2 = Piece::new(type1);
        let piece3 = Piece::new(type1);

        board.set_piece(Pos::new(0, 0), piece1);
        board.set_piece(Pos::new(1, 1), piece2);

        board.next_match();
        board.next_match();

        board.set_piece(Pos::new(2, 2), piece3);

        let next_match = board.next_match().unwrap();
        assert_eq!(Pos::new(2, 2), next_match.get_changed_pos());
        assert_eq!(Pos::new(0, 0), next_match.convert_to_board_pos(Pos::new(2, 2)));
        assert_eq!(Pos::new(1, 1), next_match.convert_to_board_pos(Pos::new(3, 3)));
        assert_eq!(Pos::new(2, 2), next_match.convert_to_board_pos(Pos::new(4, 4)));
    }

    #[test]
    fn next_match_two_patterns_different_rank_higher_picked() {
        let type1 = PieceType::new("first");

        let mut pattern_pos1 = HashMap::new();
        pattern_pos1.insert(Pos::new(2, 2), type1);
        pattern_pos1.insert(Pos::new(3, 3), type1);
        pattern_pos1.insert(Pos::new(4, 4), type1);

        let mut pattern_pos2 = HashMap::new();
        pattern_pos2.insert(Pos::new(1, 1), type1);
        pattern_pos2.insert(Pos::new(2, 2), type1);
        pattern_pos2.insert(Pos::new(3, 3), type1);
        pattern_pos2.insert(Pos::new(4, 4), type1);

        let mut board = Board::new(vec![
            MatchPattern::new(pattern_pos1, 1),
            MatchPattern::new(pattern_pos2, 2)
        ], Vec::new());
        let piece1 = Piece::new(type1);
        let piece2 = Piece::new(type1);
        let piece3 = Piece::new(type1);
        let piece4 = Piece::new(type1);

        board.set_piece(Pos::new(0, 0), piece1);
        board.set_piece(Pos::new(1, 1), piece2);
        board.set_piece(Pos::new(2, 2), piece3);

        board.next_match();
        board.next_match();
        board.next_match();

        board.set_piece(Pos::new(3, 3), piece4);

        let next_match = board.next_match().unwrap();
        assert_eq!(Pos::new(3, 3), next_match.get_changed_pos());
        assert_eq!(Pos::new(0, 0), next_match.convert_to_board_pos(Pos::new(1, 1)));
        assert_eq!(Pos::new(1, 1), next_match.convert_to_board_pos(Pos::new(2, 2)));
        assert_eq!(Pos::new(2, 2), next_match.convert_to_board_pos(Pos::new(3, 3)));
        assert_eq!(Pos::new(3, 3), next_match.convert_to_board_pos(Pos::new(4, 4)));
    }
}
