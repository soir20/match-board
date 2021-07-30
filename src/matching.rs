use std::collections::HashMap;
use crate::position::Pos;
use crate::piece::PieceType;

pub struct MatchPattern {
    spaces: HashMap<Pos, PieceType>,
    rank: u32
}

impl MatchPattern {

    pub fn get_spaces(&self) -> &HashMap<Pos, PieceType> {
        &self.spaces
    }

    pub fn get_rank(&self) -> u32 {
        self.rank
    }

}

pub struct Match<'a> {
    pattern: &'a MatchPattern,
    changed_pos: Pos,
    pattern_to_board_pos: HashMap<Pos, Pos>
}

impl Match<'_> {
    pub(crate) fn new(pattern: &MatchPattern, changed_pos: Pos,
                      pattern_to_board_pos: HashMap<Pos, Pos>) -> Match {
        Match { pattern, changed_pos, pattern_to_board_pos }
    }

    pub fn get_pattern(&self) -> &MatchPattern {
        self.pattern
    }

    pub fn get_changed_pos(&self) -> Pos {
        self.changed_pos
    }

    pub fn convert_to_board_pos(&self, pattern_pos: Pos) -> Pos {
        *self.pattern_to_board_pos.get(&pattern_pos).expect(
            &*format!("The position {} is not in the pattern", pattern_pos)
        )
    }
}