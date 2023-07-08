use std::collections::HashSet;
use crate::position::Pos;

/// A pattern of piece positions that represents a valid match on a board.
#[derive(Clone, Eq, PartialEq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchPattern<M> {
    match_type: M,
    spaces: HashSet<Pos>
}

impl<M: Copy> MatchPattern<M> {

    /// Creates a new pattern.
    ///
    /// # Arguments
    ///
    /// * `match_type` - match type all pieces must have for this pattern to apply
    /// * `spaces` - unique positions that represents a pattern. The values of the
    ///              positions do not matter: only their relative positions matter.
    pub fn new(match_type: M, spaces: &[Pos]) -> MatchPattern<M> {
        let min_x = spaces.iter().map(|space | space.x()).min().unwrap_or(0);
        let min_y = spaces.iter().map(|space | space.y()).min().unwrap_or(0);

        let spaces_around_origin = spaces.iter().map(
            |space| Pos::new(space.x() - min_x, space.y() - min_y)
        ).collect();

        MatchPattern { match_type, spaces: spaces_around_origin }
    }

    /// Gets the type of pieces in this pattern.
    pub fn match_type(&self) -> M {
        self.match_type
    }

    /// Returns an iterator of all of the relative positions in this pattern.
    pub fn iter(&self) -> impl Iterator<Item=&Pos> {
        self.spaces.iter()
    }

}

// A match found in a board.
#[derive(Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Match<'a, M> {
    pattern: &'a MatchPattern<M>,
    changed_pos: Pos,
    board_pos: HashSet<Pos>
}

impl<M> Match<'_, M> {

    /// Creates a new match.
    ///
    /// # Arguments
    ///
    /// * `pattern` - the pattern of the found match
    /// * `changed_pos` - the position that was changed and triggered the match
    /// * `board_pos` - actual positions on the board
    pub(crate) fn new(pattern: &MatchPattern<M>, changed_pos: Pos, board_pos: HashSet<Pos>) -> Match<M> {
        Match { pattern, changed_pos, board_pos }
    }

    /// Gets the pattern associated with this match.
    pub fn pattern(&self) -> &MatchPattern<M> {
        self.pattern
    }

    /// Gets the changed position that triggered this match.
    pub fn changed_pos(&self) -> Pos {
        self.changed_pos
    }

    /// Checks if the given position on the board is part of the match.
    ///
    /// # Arguments
    ///
    /// * `pos` - position to check for in this match
    pub fn contains(&self, pos: Pos) -> bool {
        self.board_pos.contains(&pos)
    }

    /// Returns an iterator of all of the board positions where this pattern is located.
    pub fn iter(&self) -> impl Iterator<Item=&Pos> {
        self.board_pos.iter()
    }

}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use crate::matching::{MatchPattern, Match};
    use crate::position::Pos;

    #[test]
    fn new_pattern_empty_set_works() {
        let spaces = Vec::new();
        let pattern = MatchPattern::new(0, &spaces[..]);
        assert!(pattern.iter().next().is_none());
    }

    #[test]
    fn new_pattern_filled_set_works() {
        let mut spaces = Vec::new();
        spaces.push(Pos::new(0, 1));
        spaces.push(Pos::new(1, 0));
        spaces.push(Pos::new(5, 5));

        let pattern = MatchPattern::new(0, &spaces[..]);

        let mut expected_spaces = HashSet::new();
        expected_spaces.insert(Pos::new(0, 1));
        expected_spaces.insert(Pos::new(1, 0));
        expected_spaces.insert(Pos::new(5, 5));

        assert_eq!(expected_spaces, pattern.iter().map(|&pos| pos).collect());
    }

    #[test]
    fn new_pattern_not_at_origin_set_moved_horizontally() {
        let mut spaces = Vec::new();
        spaces.push(Pos::new(4, 1));
        spaces.push(Pos::new(5, 0));
        spaces.push(Pos::new(9, 5));

        let pattern = MatchPattern::new(0, &spaces[..]);

        let mut expected_spaces = HashSet::new();
        expected_spaces.insert(Pos::new(0, 1));
        expected_spaces.insert(Pos::new(1, 0));
        expected_spaces.insert(Pos::new(5, 5));

        assert_eq!(expected_spaces, pattern.iter().map(|&pos| pos).collect());
    }

    #[test]
    fn new_pattern_not_at_origin_set_moved_vertically() {
        let mut spaces = Vec::new();
        spaces.push(Pos::new(0, 6));
        spaces.push(Pos::new(1, 5));
        spaces.push(Pos::new(5, 10));

        let pattern = MatchPattern::new(0, &spaces[..]);

        let mut expected_spaces = HashSet::new();
        expected_spaces.insert(Pos::new(0, 1));
        expected_spaces.insert(Pos::new(1, 0));
        expected_spaces.insert(Pos::new(5, 5));

        assert_eq!(expected_spaces, pattern.iter().map(|&pos| pos).collect());
    }

    #[test]
    fn new_pattern_not_at_origin_set_moved_horizontally_vertically() {
        let mut spaces = Vec::new();
        spaces.push(Pos::new(4, 6));
        spaces.push(Pos::new(5, 5));
        spaces.push(Pos::new(9, 10));

        let pattern = MatchPattern::new(0, &spaces[..]);

        let mut expected_spaces = HashSet::new();
        expected_spaces.insert(Pos::new(0, 1));
        expected_spaces.insert(Pos::new(1, 0));
        expected_spaces.insert(Pos::new(5, 5));

        assert_eq!(expected_spaces, pattern.iter().map(|&pos| pos).collect());
    }

    #[test]
    fn new_pattern_at_large_val_set_moved_horizontally_vertically() {
        let mut spaces = Vec::new();
        spaces.push(Pos::new(usize::MAX, usize::MAX));
        spaces.push(Pos::new(usize::MAX, usize::MAX - 1));
        spaces.push(Pos::new(usize::MAX - 1, usize::MAX));

        let pattern = MatchPattern::new(0, &spaces[..]);

        let mut expected_spaces = HashSet::new();
        expected_spaces.insert(Pos::new(1, 1));
        expected_spaces.insert(Pos::new(1, 0));
        expected_spaces.insert(Pos::new(0, 1));

        assert_eq!(expected_spaces, pattern.iter().map(|&pos| pos).collect());
    }

    #[test]
    fn new_pattern_created_with_type_has_type() {
        let mut spaces = Vec::new();
        spaces.push(Pos::new(0, 1));
        spaces.push(Pos::new(1, 0));
        spaces.push(Pos::new(5, 5));

        let pattern = MatchPattern::new(0, &spaces[..]);
        assert_eq!(0, pattern.match_type());
    }

    #[test]
    fn new_match_created_with_pattern_has_pattern() {
        let mut spaces = Vec::new();
        spaces.push(Pos::new(0, 1));
        spaces.push(Pos::new(1, 0));
        spaces.push(Pos::new(5, 5));

        let pattern = MatchPattern::new(0, &spaces[..]);

        let mut board_pos = HashSet::new();
        board_pos.insert(Pos::new(5, 1));
        board_pos.insert(Pos::new(6, 0));
        board_pos.insert(Pos::new(10, 5));

        let match1 = Match::new(&pattern, Pos::new(6, 0), board_pos);
        assert_eq!(pattern, *match1.pattern());
    }

    #[test]
    fn new_match_created_with_changed_pos_has_changed_pos() {
        let mut spaces = Vec::new();
        spaces.push(Pos::new(0, 1));
        spaces.push(Pos::new(1, 0));
        spaces.push(Pos::new(5, 5));

        let pattern = MatchPattern::new(0, &spaces[..]);

        let mut board_pos = HashSet::new();
        board_pos.insert(Pos::new(5, 1));
        board_pos.insert(Pos::new(6, 0));
        board_pos.insert(Pos::new(10, 5));

        let match1 = Match::new(&pattern, Pos::new(6, 0), board_pos);
        assert_eq!(Pos::new(6, 0), match1.changed_pos());
    }

    #[test]
    fn new_match_created_with_board_pos_has_board_pos() {
        let mut spaces = Vec::new();
        spaces.push(Pos::new(0, 1));
        spaces.push(Pos::new(1, 0));
        spaces.push(Pos::new(5, 5));

        let pattern = MatchPattern::new(0, &spaces[..]);

        let mut board_pos = HashSet::new();
        board_pos.insert(Pos::new(5, 1));
        board_pos.insert(Pos::new(6, 0));
        board_pos.insert(Pos::new(10, 5));

        let mut expected_board_pos = HashSet::new();
        expected_board_pos.insert(Pos::new(5, 1));
        expected_board_pos.insert(Pos::new(6, 0));
        expected_board_pos.insert(Pos::new(10, 5));

        let match1 = Match::new(&pattern, Pos::new(6, 0), board_pos);
        assert_eq!(expected_board_pos, match1.iter().map(|&pos| pos).collect());
    }
}
