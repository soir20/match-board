use std::collections::HashMap;
use crate::board::{Pos, Board};
use crate::piece::PieceType;

pub struct MatchPattern {
    spaces: HashMap<Pos, PieceType>,
    rank: u32
}

impl MatchPattern {
    pub fn find_match(&self, board: &Board, pos: Pos) -> Option<Vec<Pos>> {
        self.spaces.keys().into_iter().find_map(|&original|
            self.check_variant_match(board, pos - original)
        )
    }

    fn check_variant_match(&self, board: &Board, new_origin: Pos) -> Option<Vec<Pos>> {
        let mut original_to_board_pos = self.change_origin(new_origin);
        let is_match = original_to_board_pos.iter().all(|(original_pos, board_pos)|
            match board.get_piece(*board_pos) {
                None => false,
                Some(piece) => piece.get_type() == self.spaces.get(original_pos)
                    .expect("Known piece wasn't found in pattern!")
            }
        );

        match is_match {
            true => Some(original_to_board_pos.drain(..).map(|(_, board)| board).collect()),
            false => None
        }
    }

    fn change_origin(&self, origin: Pos) -> Vec<(Pos, Pos)> {
        let mut original_positions: Vec<Pos> = self.spaces.keys().map(|&pos| pos).collect();
        let mut changed_positions: Vec<Pos> = original_positions.iter().map(
            |&original| original + origin
        ).collect();

        original_positions.drain(..).zip(changed_positions.drain(..)).collect()
    }
}

pub struct Match<'a> {
    pub pattern: &'a MatchPattern,
    pub positions: Vec<Pos>
}