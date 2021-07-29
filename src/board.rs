use crate::matching::{MatchPattern, Match};
use std::collections::{HashMap, VecDeque};
use crate::piece::Piece;
use std::ops::{Add, Sub};

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