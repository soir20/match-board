use crate::position::Pos;
use crate::bitboard::PosSet;
use crate::piece::PieceType;

/// A pattern of piece positions that represents a valid match on a board.
#[derive(Debug, Eq, PartialEq)]
pub struct MatchPattern {
    piece_type: PieceType,
    spaces: PosSet,
    rank: u32
}

impl MatchPattern {

    /// Creates a new pattern.
    ///
    /// # Arguments
    ///
    /// * `spaces` - a set of unique positions that represents a pattern.
    ///              The values of the positions do not matter: only their
    ///              relative positions matter.
    /// * `rank`    - the rank of a match. A higher ranked match takes precedence over
    ///               a lower ranked one.
    pub fn new(piece_type: PieceType, spaces: PosSet, rank: u32) -> MatchPattern {
        MatchPattern { piece_type, spaces, rank }
    }

    pub fn piece_type(&self) -> PieceType {
        self.piece_type
    }

    /// Gets the relative position list for this pattern.
    pub fn spaces(&self) -> &PosSet {
        &self.spaces
    }

    // Gets the rank of this pattern.
    pub fn rank(&self) -> u32 {
        self.rank
    }

}

// A match found in a board.
#[derive(Debug, Eq, PartialEq)]
pub struct Match<'a> {
    pattern: &'a MatchPattern,
    changed_pos: Pos,
    board_pos: PosSet
}

impl Match<'_> {

    /// Creates a new match.
    ///
    /// # Arguments
    ///
    /// * `pattern` - the pattern of the found match
    /// * `changed_pos` - the position that was changed and triggered the match
    /// * `board_pos` - actual positions on the board
    pub(crate) fn new(pattern: &MatchPattern, changed_pos: Pos, board_pos: PosSet) -> Match {
        Match { pattern, changed_pos, board_pos }
    }

    /// Gets the pattern associated with this match.
    pub fn pattern(&self) -> &MatchPattern {
        self.pattern
    }

    /// Gets the changed position that triggered this match.
    pub fn changed_pos(&self) -> Pos {
        self.changed_pos
    }

    /// Gets all of the board positions where this pattern is located.
    pub fn board_pos(&self) -> &PosSet {
        &self.board_pos
    }

}

#[cfg(test)]
mod tests {
    use crate::matching::{MatchPattern, Match};
    use std::collections::{HashSet};
    use crate::position::Pos;
    use crate::piece::PieceType;

    #[test]
    fn new_pattern_empty_set_works() {
        let spaces = HashSet::new();
        let pattern = MatchPattern::new(PieceType::new("test"), spaces, 10);
        assert!(pattern.spaces().is_empty());
    }

    #[test]
    fn new_pattern_filled_set_works() {
        let mut spaces = HashSet::new();
        spaces.insert(Pos::new(0, 1));
        spaces.insert(Pos::new(1, 0));
        spaces.insert(Pos::new(5, 5));

        let pattern = MatchPattern::new(PieceType::new("test"), spaces, 10);

        let mut expected_spaces = HashSet::new();
        expected_spaces.insert(Pos::new(0, 1));
        expected_spaces.insert(Pos::new(1, 0));
        expected_spaces.insert(Pos::new(5, 5));

        assert_eq!(expected_spaces, *pattern.spaces());
    }

    #[test]
    fn new_pattern_created_with_rank_has_rank() {
        let mut spaces = HashSet::new();
        spaces.insert(Pos::new(0, 1));
        spaces.insert(Pos::new(1, 0));
        spaces.insert(Pos::new(5, 5));

        let pattern = MatchPattern::new(PieceType::new("test"), spaces, 10);
        assert_eq!(10, pattern.rank());
    }

    #[test]
    fn new_match_created_with_pattern_has_pattern() {
        let mut spaces = HashSet::new();
        spaces.insert(Pos::new(0, 1));
        spaces.insert(Pos::new(1, 0));
        spaces.insert(Pos::new(5, 5));

        let pattern = MatchPattern::new(PieceType::new("test"), spaces, 10);

        let mut board_pos = HashSet::new();
        board_pos.insert(Pos::new(5, 1));
        board_pos.insert(Pos::new(6, 0));
        board_pos.insert(Pos::new(10, 5));

        let match1 = Match::new(&pattern, Pos::new(6, 0), board_pos);
        assert_eq!(pattern, *match1.pattern());
    }

    #[test]
    fn new_match_created_with_changed_pos_has_changed_pos() {
        let mut spaces = HashSet::new();
        spaces.insert(Pos::new(0, 1));
        spaces.insert(Pos::new(1, 0));
        spaces.insert(Pos::new(5, 5));

        let pattern = MatchPattern::new(PieceType::new("test"), spaces, 10);

        let mut board_pos = HashSet::new();
        board_pos.insert(Pos::new(5, 1));
        board_pos.insert(Pos::new(6, 0));
        board_pos.insert(Pos::new(10, 5));

        let match1 = Match::new(&pattern, Pos::new(6, 0), board_pos);
        assert_eq!(Pos::new(6, 0), match1.changed_pos());
    }

    #[test]
    fn new_match_created_with_board_pos_has_board_pos() {
        let mut spaces = HashSet::new();
        spaces.insert(Pos::new(0, 1));
        spaces.insert(Pos::new(1, 0));
        spaces.insert(Pos::new(5, 5));

        let pattern = MatchPattern::new(PieceType::new("test"), spaces, 10);

        let mut board_pos = HashSet::new();
        board_pos.insert(Pos::new(5, 1));
        board_pos.insert(Pos::new(6, 0));
        board_pos.insert(Pos::new(10, 5));

        let mut expected_board_pos = HashSet::new();
        expected_board_pos.insert(Pos::new(5, 1));
        expected_board_pos.insert(Pos::new(6, 0));
        expected_board_pos.insert(Pos::new(10, 5));

        let match1 = Match::new(&pattern, Pos::new(6, 0), board_pos);
        assert_eq!(expected_board_pos, *match1.board_pos());
    }
}