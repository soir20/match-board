use std::collections::HashMap;
use crate::position::Pos;
use crate::piece::PieceType;

/// A pattern of piece types that represents a valid match on a board.
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
    pub fn get_spaces(&self) -> &HashMap<Pos, PieceType> {
        &self.spaces
    }

    // Gets the rank of this pattern.
    pub fn get_rank(&self) -> u32 {
        self.rank
    }

}

// A match found in a board.
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
    pub fn get_pattern(&self) -> &MatchPattern {
        self.pattern
    }

    /// Gets the changed position that triggered this match.
    pub fn get_changed_pos(&self) -> Pos {
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