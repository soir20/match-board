use std::collections::HashMap;
use crate::position::Pos;
use crate::piece::PieceType;

/// A pattern of piece types that represents a valid match on a board.
#[derive(Debug, Eq, PartialEq)]
pub struct MatchPattern {
    spaces: HashMap<Pos, PieceType>,
    rank: u32
}

impl MatchPattern {

    /// Creates a new pattern.
    ///
    /// # Arguments
    ///
    /// * `spaces_to_types` - a mapping of positions to piece types. The values of the positions
    ///                       do not matter: only their relative positions matter.
    /// * `rank`            - the rank of a match. A higher ranked match takes precedence over
    ///                       a lower ranked one.
    pub fn new(spaces_to_types: HashMap<Pos, PieceType>, rank: u32) -> MatchPattern {
        MatchPattern { spaces: spaces_to_types, rank }
    }

    /// Gets the relative position to type mapping for this pattern.
    pub fn spaces(&self) -> &HashMap<Pos, PieceType> {
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
    pattern_to_board_pos: HashMap<Pos, Pos>
}

impl Match<'_> {

    /// Creates a new match.
    ///
    /// # Arguments
    ///
    /// * `pattern` - the pattern of the found match
    /// * `changed_pos` - the position that was changed and triggered the match
    /// * `pattern_to_board_pos` - a mapping from relative positions in the pattern to
    ///                            actual positions on the board
    pub(crate) fn new(pattern: &MatchPattern, changed_pos: Pos,
                      pattern_to_board_pos: HashMap<Pos, Pos>) -> Match {
        Match { pattern, changed_pos, pattern_to_board_pos }
    }

    /// Gets the pattern associated with this match.
    pub fn pattern(&self) -> &MatchPattern {
        self.pattern
    }

    /// Gets the changed position that triggered this match.
    pub fn changed_pos(&self) -> Pos {
        self.changed_pos
    }

    /// Converts a relative position in the pattern to its actual position on the board.
    ///
    /// # Arguments
    ///
    /// `pattern_pos` - a position in the pattern to convert to its actual board position.
    ///                 This **must** be in the pattern. Otherwise, this method will panic
    ///                 as passing a non-pattern position is a bug.
    pub fn convert_to_board_pos(&self, pattern_pos: Pos) -> Pos {
        *self.pattern_to_board_pos.get(&pattern_pos).expect(
            &*format!("The position {} is not in the pattern", pattern_pos)
        )
    }

}

#[cfg(test)]
mod tests {
    use crate::matching::{MatchPattern, Match};
    use std::collections::HashMap;
    use crate::piece::PieceType;
    use crate::position::Pos;

    #[test]
    fn new_pattern_empty_map_works() {
        let spaces_to_types = HashMap::new();
        let pattern = MatchPattern::new(spaces_to_types, 10);
        assert!(pattern.spaces().is_empty());
    }

    #[test]
    fn new_pattern_filled_map_works() {
        let mut spaces_to_types = HashMap::new();
        spaces_to_types.insert(Pos::new(0, 1), PieceType::new("first"));
        spaces_to_types.insert(Pos::new(1, 0), PieceType::new("second"));
        spaces_to_types.insert(Pos::new(5, 5), PieceType::new("first"));

        let pattern = MatchPattern::new(spaces_to_types, 10);
        assert_eq!(3, pattern.spaces().len());
        assert_eq!(&PieceType::new("first"), pattern.spaces().get(&Pos::new(0, 1)).unwrap());
        assert_eq!(&PieceType::new("second"), pattern.spaces().get(&Pos::new(1, 0)).unwrap());
        assert_eq!(&PieceType::new("first"), pattern.spaces().get(&Pos::new(5, 5)).unwrap());
    }

    #[test]
    fn new_pattern_created_with_rank_has_rank() {
        let mut spaces_to_types = HashMap::new();
        spaces_to_types.insert(Pos::new(0, 1), PieceType::new("first"));
        spaces_to_types.insert(Pos::new(1, 0), PieceType::new("second"));
        spaces_to_types.insert(Pos::new(5, 5), PieceType::new("first"));

        let pattern = MatchPattern::new(spaces_to_types, 10);
        assert_eq!(10, pattern.rank());
    }

    #[test]
    fn new_match_created_with_pattern_has_pattern() {
        let mut spaces_to_types = HashMap::new();
        spaces_to_types.insert(Pos::new(0, 1), PieceType::new("first"));
        spaces_to_types.insert(Pos::new(1, 0), PieceType::new("second"));
        spaces_to_types.insert(Pos::new(5, 5), PieceType::new("first"));
        let pattern = MatchPattern::new(spaces_to_types, 10);

        let mut pattern_to_board = HashMap::new();
        pattern_to_board.insert(Pos::new(0, 1) , Pos::new(5, -1));
        pattern_to_board.insert(Pos::new(1, 0), Pos::new(6, -2));
        pattern_to_board.insert(Pos::new(5, 5), Pos::new(10, 3));

        let match1 = Match::new(&pattern, Pos::new(6, -2), pattern_to_board);
        assert_eq!(pattern, *match1.pattern());
    }

    #[test]
    fn new_match_created_with_changed_pos_has_changed_pos() {
        let mut spaces_to_types = HashMap::new();
        spaces_to_types.insert(Pos::new(0, 1), PieceType::new("first"));
        spaces_to_types.insert(Pos::new(1, 0), PieceType::new("second"));
        spaces_to_types.insert(Pos::new(5, 5), PieceType::new("first"));
        let pattern = MatchPattern::new(spaces_to_types, 10);

        let mut pattern_to_board = HashMap::new();
        pattern_to_board.insert(Pos::new(0, 1) , Pos::new(5, -1));
        pattern_to_board.insert(Pos::new(1, 0), Pos::new(6, -2));
        pattern_to_board.insert(Pos::new(5, 5), Pos::new(10, 3));

        let match1 = Match::new(&pattern, Pos::new(6, -2), pattern_to_board);
        assert_eq!(Pos::new(6, -2), match1.changed_pos());
    }

    #[test]
    fn convert_to_board_pos_in_pattern_gets_board_pos() {
        let mut spaces_to_types = HashMap::new();
        spaces_to_types.insert(Pos::new(0, 1), PieceType::new("first"));
        spaces_to_types.insert(Pos::new(1, 0), PieceType::new("second"));
        spaces_to_types.insert(Pos::new(5, 5), PieceType::new("first"));
        let pattern = MatchPattern::new(spaces_to_types, 10);

        let mut pattern_to_board = HashMap::new();
        pattern_to_board.insert(Pos::new(0, 1) , Pos::new(5, -1));
        pattern_to_board.insert(Pos::new(1, 0), Pos::new(6, -2));
        pattern_to_board.insert(Pos::new(5, 5), Pos::new(10, 3));

        let match1 = Match::new(&pattern, Pos::new(6, -2), pattern_to_board);
        assert_eq!(Pos::new(5, -1), match1.convert_to_board_pos(Pos::new(0, 1)));
        assert_eq!(Pos::new(6, -2), match1.convert_to_board_pos(Pos::new(1, 0)));
        assert_eq!(Pos::new(10, 3), match1.convert_to_board_pos(Pos::new(5, 5)));
    }

    #[test]
    #[should_panic]
    fn convert_to_board_pos_not_in_pattern_panics() {
        let mut spaces_to_types = HashMap::new();
        spaces_to_types.insert(Pos::new(0, 1), PieceType::new("first"));
        spaces_to_types.insert(Pos::new(1, 0), PieceType::new("second"));
        spaces_to_types.insert(Pos::new(5, 5), PieceType::new("first"));
        let pattern = MatchPattern::new(spaces_to_types, 10);

        let mut pattern_to_board = HashMap::new();
        pattern_to_board.insert(Pos::new(0, 1) , Pos::new(5, -1));
        pattern_to_board.insert(Pos::new(1, 0), Pos::new(6, -2));
        pattern_to_board.insert(Pos::new(5, 5), Pos::new(10, 3));

        let match1 = Match::new(&pattern, Pos::new(6, -2), pattern_to_board);
        match1.convert_to_board_pos(Pos::new(1, 1));
    }
}