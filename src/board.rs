use std::collections::{HashMap, VecDeque};
use crate::piece::{Piece, Direction};
use crate::position::Pos;
use crate::matching::{MatchPattern, Match};

pub struct Board {
    patterns: Vec<MatchPattern>,
    swap_rules: Vec<Box<dyn Fn(&Board, Pos, Pos) -> bool>>,
    pieces: HashMap<Pos, Piece>,
    last_changed: VecDeque<Pos>
}

impl Board {
    pub fn new(patterns: Vec<MatchPattern>,
               mut swap_rules: Vec<Box<dyn Fn(&Board, Pos, Pos) -> bool>>) -> Board {
        swap_rules.insert(0, Box::from(Board::are_pieces_movable));
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

        self.last_changed.push_back(first);
        self.last_changed.push_back(second);

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
                let positions = self.find_match(pattern, next_pos)?;
                Some(Match::new(pattern, next_pos, positions))
            });
        }

        next_match
    }

    fn find_match(&self, pattern: &MatchPattern, pos: Pos) -> Option<HashMap<Pos, Pos>> {
        pattern.get_spaces().keys().into_iter().find_map(|&original|
            self.check_variant_match(pattern, pos - original)
        )
    }

    fn check_variant_match(&self, pattern: &MatchPattern, new_origin: Pos) -> Option<HashMap<Pos, Pos>> {
        let original_to_board_pos = Board::change_origin(pattern, new_origin);
        let is_match = original_to_board_pos.iter().all(|(original_pos, board_pos)|
            match self.get_piece(*board_pos) {
                None => false,
                Some(piece) => piece.get_type() == pattern.get_spaces().get(original_pos)
                    .expect("Known piece wasn't found in pattern!")
            }
        );

        match is_match {
            true => Some(original_to_board_pos),
            false => None
        }
    }

    fn change_origin(pattern: &MatchPattern, origin: Pos) -> HashMap<Pos, Pos> {
        let mut original_positions: Vec<Pos> = pattern.get_spaces().keys().map(|&pos| pos).collect();
        let mut changed_positions: Vec<Pos> = original_positions.iter().map(
            |&original| original + origin
        ).collect();

        original_positions.drain(..).zip(changed_positions.drain(..)).collect()
    }

    fn are_pieces_movable(board: &Board, first: Pos, second: Pos) -> bool {
        let is_first_movable = match board.get_piece(first) {
            None => true,
            Some(piece) => Board::is_movable(first, second, piece)
        };

        let is_second_movable = match board.get_piece(second) {
            None => true,
            Some(piece) => Board::is_movable(second, first, piece)
        };

        is_first_movable && is_second_movable
    }

    fn is_movable(from: Pos, to: Pos, piece: &Piece) -> bool {
        Board::is_vertically_movable(from, to, piece) && Board::is_horizontally_movable(from, to, piece)
    }

    fn is_vertically_movable(from: Pos, to: Pos, piece: &Piece) -> bool {
        let vertical_change = to.get_y() - from.get_y();
        if vertical_change > 0 {
            return piece.is_movable(Direction::North);
        } else if vertical_change < 0 {
            return piece.is_movable(Direction::South);
        }

        true
    }

    fn is_horizontally_movable(from: Pos, to: Pos, piece: &Piece) -> bool {
        let horizontal_change = to.get_x() - from.get_x();
        if horizontal_change > 0 {
            return piece.is_movable(Direction::East);
        } else if horizontal_change < 0 {
            return piece.is_movable(Direction::West);
        }

        true
    }
}