use std::collections::{HashMap, VecDeque};
use crate::piece::{Piece, PieceType};
use std::ops::{Add, Sub};

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct Pos {
    pub x: i32,
    pub y: i32
}

impl Add for Pos {
    type Output = Pos;

    fn add(self, rhs: Self) -> Self::Output {
        Pos {x: self.x + rhs.x, y: self.y + rhs.y}
    }
}

impl Sub for Pos {
    type Output = Pos;

    fn sub(self, rhs: Self) -> Self::Output {
        Pos {x: self.x - rhs.x, y: self.y - rhs.y}
    }
}

pub struct Board {
    patterns: Vec<MatchPattern>,
    pieces: HashMap<Pos, Piece>,
    last_changed: VecDeque<Pos>
}

impl Board {
    pub fn get_piece(&self, pos: Pos) -> Option<&Piece> {
        self.pieces.get(&pos)
    }

    pub fn set_piece(&mut self, pos: Pos, piece: Piece) -> Option<Piece> {
        self.last_changed.push_back(pos);
        self.pieces.insert(pos, piece)
    }

    pub fn next_match(&mut self) -> Option<Match> {
        let mut next_pos;
        let mut next_match = None;

        while next_match.is_none() {
            next_pos = self.last_changed.pop_front()?;
            next_match = self.patterns.iter().find_map(|pattern| {
                let positions = pattern.find_match(self, next_pos)?;
                Some(Match { pattern, positions })
            });
        }

        next_match
    }
}

pub struct MatchPattern {
    spaces: HashMap<Pos, PieceType>,
    rank: u32
}

impl MatchPattern {
    fn find_match(&self, board: &Board, pos: Pos) -> Option<Vec<Pos>> {
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