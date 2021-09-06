use crate::board::PosSet;
use crate::piece::PieceType;
use crate::position::Pos;

use std::fmt::{Display, Formatter};

/// A pattern of piece positions that represents a valid match on a board.
#[derive(Clone, Eq, PartialEq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
        let min_x = spaces.iter().map(|space | space.x()).min().unwrap_or(0);
        let min_y = spaces.iter().map(|space | space.y()).min().unwrap_or(0);

        let spaces_around_origin = spaces.iter().map(
            |space| Pos::new(space.x() - min_x, space.y() - min_y)
        ).collect();

        MatchPattern { piece_type, spaces: spaces_around_origin, rank }
    }

    /// Gets the type of pieces in this pattern.
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

impl Display for MatchPattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut str = String::new();

        let type_abbreviation = self.piece_type;

        let max_x = self.spaces.iter().map(|pos| pos.x()).max().unwrap_or(0);
        let max_y = self.spaces.iter().map(|pos| pos.y()).max().unwrap_or(0);

        for y in (0..=max_y).rev() {
            for x in 0..=max_x {
                match self.spaces.contains(&Pos::new(x, y)) {
                    true => str.push(type_abbreviation),
                    false => str.push('.')
                };
            }

            str.push('\n');
        }

        str.push_str("r = ");
        str.push_str(&self.rank().to_string());

        write!(f, "{}", str)
    }
}

// A match found in a board.
#[derive(Clone, Eq, PartialEq, Debug)]
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

impl Display for Match<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut str = String::new();

        let type_abbreviation = self.pattern().piece_type();

        let max_x = self.board_pos.iter().map(|pos| pos.x()).max().unwrap_or(0);
        let max_y = self.board_pos.iter().map(|pos| pos.y()).max().unwrap_or(0);

        for y in (0..=max_y).rev() {
            for x in 0..=max_x {
                let pos = Pos::new(x, y);
                str.push(if pos == self.changed_pos() {
                    'X'
                } else if self.board_pos.contains(&pos) {
                    type_abbreviation
                } else {
                    '.'
                });
            }

            str.push('\n');
        }

        str.push_str("r = ");
        str.push_str(&self.pattern().rank().to_string());

        write!(f, "{}", str)
    }
}

#[cfg(test)]
mod tests {
    use crate::matching::{MatchPattern, Match};
    use std::collections::{HashSet};
    use crate::position::Pos;

    #[test]
    fn new_pattern_empty_set_works() {
        let spaces = HashSet::new();
        let pattern = MatchPattern::new('t', spaces, 10);
        assert!(pattern.spaces().is_empty());
    }

    #[test]
    fn new_pattern_filled_set_works() {
        let mut spaces = HashSet::new();
        spaces.insert(Pos::new(0, 1));
        spaces.insert(Pos::new(1, 0));
        spaces.insert(Pos::new(5, 5));

        let pattern = MatchPattern::new('t', spaces, 10);

        let mut expected_spaces = HashSet::new();
        expected_spaces.insert(Pos::new(0, 1));
        expected_spaces.insert(Pos::new(1, 0));
        expected_spaces.insert(Pos::new(5, 5));

        assert_eq!(expected_spaces, *pattern.spaces());
    }

    #[test]
    fn new_pattern_not_at_origin_set_moved_horizontally() {
        let mut spaces = HashSet::new();
        spaces.insert(Pos::new(4, 1));
        spaces.insert(Pos::new(5, 0));
        spaces.insert(Pos::new(9, 5));

        let pattern = MatchPattern::new('t', spaces, 10);

        let mut expected_spaces = HashSet::new();
        expected_spaces.insert(Pos::new(0, 1));
        expected_spaces.insert(Pos::new(1, 0));
        expected_spaces.insert(Pos::new(5, 5));

        assert_eq!(expected_spaces, *pattern.spaces());
    }

    #[test]
    fn new_pattern_not_at_origin_set_moved_vertically() {
        let mut spaces = HashSet::new();
        spaces.insert(Pos::new(0, 6));
        spaces.insert(Pos::new(1, 5));
        spaces.insert(Pos::new(5, 10));

        let pattern = MatchPattern::new('t', spaces, 10);

        let mut expected_spaces = HashSet::new();
        expected_spaces.insert(Pos::new(0, 1));
        expected_spaces.insert(Pos::new(1, 0));
        expected_spaces.insert(Pos::new(5, 5));

        assert_eq!(expected_spaces, *pattern.spaces());
    }

    #[test]
    fn new_pattern_not_at_origin_set_moved_horizontally_vertically() {
        let mut spaces = HashSet::new();
        spaces.insert(Pos::new(4, 6));
        spaces.insert(Pos::new(5, 5));
        spaces.insert(Pos::new(9, 10));

        let pattern = MatchPattern::new('t', spaces, 10);

        let mut expected_spaces = HashSet::new();
        expected_spaces.insert(Pos::new(0, 1));
        expected_spaces.insert(Pos::new(1, 0));
        expected_spaces.insert(Pos::new(5, 5));

        assert_eq!(expected_spaces, *pattern.spaces());
    }

    #[test]
    fn new_pattern_at_max_set_moved_horizontally_vertically() {
        let mut spaces = HashSet::new();
        spaces.insert(Pos::new(u8::MAX, u8::MAX));
        spaces.insert(Pos::new(u8::MAX, u8::MAX - 1));
        spaces.insert(Pos::new(u8::MAX - 1, u8::MAX));

        let pattern = MatchPattern::new('t', spaces, 10);

        let mut expected_spaces = HashSet::new();
        expected_spaces.insert(Pos::new(0, 1));
        expected_spaces.insert(Pos::new(1, 0));
        expected_spaces.insert(Pos::new(1, 1));

        assert_eq!(expected_spaces, *pattern.spaces());
    }

    #[test]
    fn new_pattern_created_with_rank_has_rank() {
        let mut spaces = HashSet::new();
        spaces.insert(Pos::new(0, 1));
        spaces.insert(Pos::new(1, 0));
        spaces.insert(Pos::new(5, 5));

        let pattern = MatchPattern::new('t', spaces, 10);
        assert_eq!(10, pattern.rank());
    }

    #[test]
    fn new_pattern_created_with_type_has_type() {
        let mut spaces = HashSet::new();
        spaces.insert(Pos::new(0, 1));
        spaces.insert(Pos::new(1, 0));
        spaces.insert(Pos::new(5, 5));

        let pattern = MatchPattern::new('t', spaces, 10);
        assert_eq!('t', pattern.piece_type());
    }

    #[test]
    fn display_pattern_shows_points_at_origin() {
        let mut spaces = HashSet::new();
        spaces.insert(Pos::new(3, 4));
        spaces.insert(Pos::new(4, 2));
        spaces.insert(Pos::new(5, 5));

        let pattern = MatchPattern::new('t', spaces, 10);

        let expected = "\
        ..t\
        \nt..\
        \n...\
        \n.t.\
        \nr = 10\
        ";

        assert_eq!(expected, format!("{}", pattern));
    }

    #[test]
    fn new_match_created_with_pattern_has_pattern() {
        let mut spaces = HashSet::new();
        spaces.insert(Pos::new(0, 1));
        spaces.insert(Pos::new(1, 0));
        spaces.insert(Pos::new(5, 5));

        let pattern = MatchPattern::new('t', spaces, 10);

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

        let pattern = MatchPattern::new('t', spaces, 10);

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

        let pattern = MatchPattern::new('t', spaces, 10);

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

    #[test]
    fn display_match_shows_points_on_board() {
        let mut spaces = HashSet::new();
        spaces.insert(Pos::new(0, 1));
        spaces.insert(Pos::new(1, 0));
        spaces.insert(Pos::new(5, 5));

        let pattern = MatchPattern::new('t', spaces, 10);

        let mut board_pos = HashSet::new();
        board_pos.insert(Pos::new(2, 1));
        board_pos.insert(Pos::new(3, 0));
        board_pos.insert(Pos::new(7, 5));

        let match1 = Match::new(&pattern, Pos::new(3, 0), board_pos);

        let expected = "\
        .......t\
        \n........\
        \n........\
        \n........\
        \n..t.....\
        \n...X....\
        \nr = 10\
        ";

        assert_eq!(expected, format!("{}", match1));
    }
}