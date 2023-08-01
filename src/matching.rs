use std::collections::HashSet;
use crate::position::Pos;

/// A pattern of piece positions that represents a valid match on a board.
#[derive(Clone, Eq, PartialEq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchPattern<M, const BOARD_WIDTH: usize, const BOARD_HEIGHT: usize> {
    match_type: M,
    spaces: HashSet<Pos<BOARD_WIDTH, BOARD_HEIGHT>>
}

impl<M: Copy, const W: usize, const H: usize> MatchPattern<M, W, H> {

    /// Creates a new pattern.
    ///
    /// # Arguments
    ///
    /// * `match_type` - match type all pieces must have for this pattern to apply
    /// * `spaces` - unique positions that represents a pattern. The values of the
    ///              positions do not matter: only their relative positions matter.
    pub fn new(match_type: M, spaces: &[Pos<W, H>]) -> MatchPattern<M, W, H> {
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
    pub fn iter(&self) -> impl Iterator<Item=&Pos<W, H>> {
        self.spaces.iter()
    }

}

// A match found in a board.
#[derive(Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Match<'a, M, const BOARD_WIDTH: usize, const BOARD_HEIGHT: usize> {
    pattern: &'a MatchPattern<M, BOARD_WIDTH, BOARD_HEIGHT>,
    changed_pos: Pos<BOARD_WIDTH, BOARD_HEIGHT>,
    board_pos: HashSet<Pos<BOARD_WIDTH, BOARD_HEIGHT>>
}

impl<M, const W: usize, const H: usize> Match<'_, M, W, H> {

    /// Creates a new match.
    ///
    /// # Arguments
    ///
    /// * `pattern` - the pattern of the found match
    /// * `changed_pos` - the position that was changed and triggered the match
    /// * `board_pos` - actual positions on the board
    pub(crate) fn new(pattern: &MatchPattern<M, W, H>, changed_pos: Pos<W, H>, board_pos: HashSet<Pos<W, H>>) -> Match<M, W, H> {
        Match { pattern, changed_pos, board_pos }
    }

    /// Gets the pattern associated with this match.
    pub fn pattern(&self) -> &MatchPattern<M, W, H> {
        self.pattern
    }

    /// Gets the changed position that triggered this match.
    pub fn changed_pos(&self) -> Pos<W, H> {
        self.changed_pos
    }

    /// Checks if the given position on the board is part of the match.
    ///
    /// # Arguments
    ///
    /// * `pos` - position to check for in this match
    pub fn contains(&self, pos: Pos<W, H>) -> bool {
        self.board_pos.contains(&pos)
    }

    /// Returns an iterator of all of the board positions where this pattern is located.
    pub fn iter(&self) -> impl Iterator<Item=&Pos<W, H>> {
        self.board_pos.iter()
    }

}

// A group of pieces where one needs to change to make a match.
#[derive(Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchMove<'a, M, const BOARD_WIDTH: usize, const BOARD_HEIGHT: usize> {
    pattern: &'a MatchPattern<M, BOARD_WIDTH, BOARD_HEIGHT>,
    missing_pos: Pos<BOARD_WIDTH, BOARD_HEIGHT>,
    completing_pos: Pos<BOARD_WIDTH, BOARD_HEIGHT>,
    board_pos: HashSet<Pos<BOARD_WIDTH, BOARD_HEIGHT>>
}

impl<M, const W: usize, const H: usize> MatchMove<'_, M, W, H> {

    /// Creates a new match move, pieces that can be swapped to produce a match.
    ///
    /// # Arguments
    ///
    /// * `pattern` - the pattern of the found match
    /// * `missing_pos` - the position that needs to be changed to make a match
    /// * `completing_pos` - the position of a piece that could be swapped with the `missing_pos`
    ///                      to create a match
    /// * `board_pos` - actual positions on the board
    pub(crate) fn new(pattern: &MatchPattern<M, W, H>, missing_pos: Pos<W, H>, completing_pos: Pos<W, H>,
                      board_pos: HashSet<Pos<W, H>>) -> MatchMove<M, W, H> {
        MatchMove { pattern, missing_pos, completing_pos, board_pos }
    }

    /// Gets the pattern associated with this move.
    pub fn pattern(&self) -> &MatchPattern<M, W, H> {
        self.pattern
    }

    /// Gets the position that needs to be changed to make a match.
    pub fn missing_pos(&self) -> Pos<W, H> {
        self.missing_pos
    }

    /// Gets a piece directly adjacent to the missing positions that can be moved to create a
    /// match. Returns the position of that piece if one is found. If multiple pieces could create a
    /// match, the position of one of them is returned, but there is no guarantee as to which piece
    /// will be selected.
    pub fn completing_pos(&self) -> Pos<W, H> {
        self.completing_pos
    }

    /// Checks if the given position on the board already contains a piece that would be part of the
    /// match.
    ///
    /// # Arguments
    ///
    /// * `pos` - position to check for in this close match
    pub fn contains(&self, pos: Pos<W, H>) -> bool {
        self.board_pos.contains(&pos)
    }

    /// Returns an iterator of all of the board positions where this pattern is located.
    /// Does not include the position that is missing.
    pub fn iter(&self) -> impl Iterator<Item=&Pos<W, H>> {
        self.board_pos.iter()
    }

}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use crate::MatchMove;
    use crate::matching::{MatchPattern, Match};
    use crate::position::Pos;

    #[test]
    fn new_pattern_empty_set_works() {
        let spaces = Vec::new();
        let pattern: MatchPattern<i32, 15, 16> = MatchPattern::new(0, &spaces[..]);
        assert!(pattern.iter().next().is_none());
    }

    #[test]
    fn new_pattern_filled_set_works() {
        let mut spaces: Vec<Pos<15, 16>> = Vec::new();
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
        let mut spaces: Vec<Pos<15, 16>> = Vec::new();
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
        let mut spaces: Vec<Pos<15, 16>> = Vec::new();
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
        let mut spaces: Vec<Pos<15, 16>> = Vec::new();
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
        let mut spaces: Vec<Pos<{ usize::MAX }, { usize::MAX }>> = Vec::new();
        spaces.push(Pos::new(usize::MAX - 1, usize::MAX - 1));
        spaces.push(Pos::new(usize::MAX - 1, usize::MAX - 2));
        spaces.push(Pos::new(usize::MAX - 2, usize::MAX - 1));

        let pattern = MatchPattern::new(0, &spaces[..]);

        let mut expected_spaces = HashSet::new();
        expected_spaces.insert(Pos::new(1, 1));
        expected_spaces.insert(Pos::new(1, 0));
        expected_spaces.insert(Pos::new(0, 1));

        assert_eq!(expected_spaces, pattern.iter().map(|&pos| pos).collect());
    }

    #[test]
    fn new_pattern_created_with_type_has_type() {
        let mut spaces: Vec<Pos<15, 16>> = Vec::new();
        spaces.push(Pos::new(0, 1));
        spaces.push(Pos::new(1, 0));
        spaces.push(Pos::new(5, 5));

        let pattern = MatchPattern::new(0, &spaces[..]);
        assert_eq!(0, pattern.match_type());
    }

    #[test]
    fn new_match_created_with_pattern_has_pattern() {
        let mut spaces: Vec<Pos<15, 16>> = Vec::new();
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
        let mut spaces: Vec<Pos<15, 16>> = Vec::new();
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
        let mut spaces: Vec<Pos<15, 16>> = Vec::new();
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

    #[test]
    fn new_close_match_created_with_pattern_has_pattern() {
        let mut spaces: Vec<Pos<15, 16>> = Vec::new();
        spaces.push(Pos::new(0, 1));
        spaces.push(Pos::new(1, 0));
        spaces.push(Pos::new(5, 5));

        let pattern = MatchPattern::new(0, &spaces[..]);

        let mut board_pos = HashSet::new();
        board_pos.insert(Pos::new(5, 1));
        board_pos.insert(Pos::new(6, 0));
        board_pos.insert(Pos::new(10, 5));

        let match1 = MatchMove::new(&pattern, Pos::new(6, 0), Pos::new(2, 3), board_pos);
        assert_eq!(pattern, *match1.pattern());
    }

    #[test]
    fn new_close_match_created_with_missing_pos_has_missing_pos() {
        let mut spaces: Vec<Pos<15, 16>> = Vec::new();
        spaces.push(Pos::new(0, 1));
        spaces.push(Pos::new(1, 0));
        spaces.push(Pos::new(5, 5));

        let pattern = MatchPattern::new(0, &spaces[..]);

        let mut board_pos = HashSet::new();
        board_pos.insert(Pos::new(5, 1));
        board_pos.insert(Pos::new(6, 0));
        board_pos.insert(Pos::new(10, 5));

        let match1 = MatchMove::new(&pattern, Pos::new(6, 0), Pos::new(2, 3), board_pos);
        assert_eq!(Pos::new(6, 0), match1.missing_pos());
    }

    #[test]
    fn new_close_match_created_with_board_pos_has_board_pos() {
        let mut spaces: Vec<Pos<15, 16>> = Vec::new();
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

        let match1 = MatchMove::new(&pattern, Pos::new(6, 0), Pos::new(2, 3), board_pos);
        assert_eq!(expected_board_pos, match1.iter().map(|&pos| pos).collect());
    }

    #[test]
    fn new_close_match_created_with_completing_pos_has_completing_pos() {
        let mut spaces: Vec<Pos<15, 16>> = Vec::new();
        spaces.push(Pos::new(0, 1));
        spaces.push(Pos::new(1, 0));
        spaces.push(Pos::new(5, 5));

        let pattern = MatchPattern::new(0, &spaces[..]);

        let mut board_pos = HashSet::new();
        board_pos.insert(Pos::new(5, 1));
        board_pos.insert(Pos::new(6, 0));
        board_pos.insert(Pos::new(10, 5));

        let match1 = MatchMove::new(&pattern, Pos::new(6, 0), Pos::new(2, 3), board_pos);
        assert_eq!(Pos::new(2, 3), match1.completing_pos());
    }
}
