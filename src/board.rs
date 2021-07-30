use std::collections::{HashMap, VecDeque};
use crate::piece::{Piece, PieceType, Direction};
use std::ops::{Add, Sub};
use std::fmt::{Display, Formatter};

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub struct Pos {
    x: i32,
    y: i32
}

impl Pos {
    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }
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

impl Display for Pos {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "({}, {})", self.x, self.y)
    }
}

pub struct Board {
    patterns: Vec<MatchPattern>,
    swap_rules: Vec<Box<dyn Fn(&Board, Pos, Pos) -> bool>>,
    pieces: HashMap<Pos, Piece>,
    last_changed: VecDeque<Pos>
}

impl Board {
    pub fn new(patterns: Vec<MatchPattern>,
               mut swap_rules: Vec<Box<dyn Fn(&Board, Pos, Pos) -> bool>>) -> Board {
        swap_rules.insert(0, Box::from(are_pieces_movable));
        Board { patterns, swap_rules, pieces: HashMap::new(), last_changed: VecDeque::new() }
    }

    pub fn get_piece(&self, pos: Pos) -> Option<&Piece> {
        self.pieces.get(&pos)
    }

    #[must_use]
    pub fn swap_pieces(&mut self, first: Pos, second: Pos) -> bool {
        if !self.swap_rules.iter().all(|rule| rule(self, first, second)) {
            return false;
        }

        let original_first_piece = self.pieces.remove(&first);
        let original_second_piece = self.pieces.remove(&second);

        if let Some(piece) = original_first_piece {
            self.pieces.insert(second, piece);
        }

        if let Some(piece) = original_second_piece {
            self.pieces.insert(first, piece);
        }

        true
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
                Some(Match { pattern, changed_pos: next_pos, pattern_to_board_pos: positions })
            });
        }

        next_match
    }
}

fn are_pieces_movable(board: &Board, first: Pos, second: Pos) -> bool {
    let is_first_movable = match board.get_piece(first) {
        None => true,
        Some(piece) => is_movable(first, second, piece)
    };

    let is_second_movable = match board.get_piece(second) {
        None => true,
        Some(piece) => is_movable(second, first, piece)
    };

    is_first_movable && is_second_movable
}

fn is_movable(from: Pos, to: Pos, piece: &Piece) -> bool {
    is_vertically_movable(from, to, piece) && is_horizontally_movable(from, to, piece)
}

fn is_vertically_movable(from: Pos, to: Pos, piece: &Piece) -> bool {
    let vertical_change = to.y - from.y;
    if vertical_change > 0 {
        return piece.is_movable(Direction::North);
    } else if vertical_change < 0 {
        return piece.is_movable(Direction::South);
    }

    true
}

fn is_horizontally_movable(from: Pos, to: Pos, piece: &Piece) -> bool {
    let horizontal_change = to.x - from.x;
    if horizontal_change > 0 {
        return piece.is_movable(Direction::East);
    } else if horizontal_change < 0 {
        return piece.is_movable(Direction::West);
    }

    true
}

pub struct MatchPattern {
    spaces: HashMap<Pos, PieceType>,
    rank: u32
}

impl MatchPattern {
    fn find_match(&self, board: &Board, pos: Pos) -> Option<HashMap<Pos, Pos>> {
        self.spaces.keys().into_iter().find_map(|&original|
            self.check_variant_match(board, pos - original)
        )
    }

    fn check_variant_match(&self, board: &Board, new_origin: Pos) -> Option<HashMap<Pos, Pos>> {
        let original_to_board_pos = self.change_origin(new_origin);
        let is_match = original_to_board_pos.iter().all(|(original_pos, board_pos)|
            match board.get_piece(*board_pos) {
                None => false,
                Some(piece) => piece.get_type() == self.spaces.get(original_pos)
                    .expect("Known piece wasn't found in pattern!")
            }
        );

        match is_match {
            true => Some(original_to_board_pos),
            false => None
        }
    }

    fn change_origin(&self, origin: Pos) -> HashMap<Pos, Pos> {
        let mut original_positions: Vec<Pos> = self.spaces.keys().map(|&pos| pos).collect();
        let mut changed_positions: Vec<Pos> = original_positions.iter().map(
            |&original| original + origin
        ).collect();

        original_positions.drain(..).zip(changed_positions.drain(..)).collect()
    }
}

pub struct Match<'a> {
    pattern: &'a MatchPattern,
    changed_pos: Pos,
    pattern_to_board_pos: HashMap<Pos, Pos>
}

impl Match<'_> {
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