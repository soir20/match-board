use std::collections::{HashMap, VecDeque};
use crate::piece::{Piece, Direction, PieceType};
use crate::position::Pos;
use crate::matching::{MatchPattern, Match};
use crate::bitboard::{BitBoard, BitBoardPiece, PieceTypeId};
use enumset::EnumSet;

pub struct Board {
    piece_types: Vec<PieceType>,
    patterns: Vec<MatchPattern>,
    swap_rules: Vec<Box<dyn Fn(&Board, Pos, Pos) -> bool>>,
    bitboard: BitBoard,
    last_changed: VecDeque<Pos>
}

impl Board {

    /// Creates a new board.
    ///
    /// # Arguments
    ///
    /// * `patterns` - the match patterns the board should use to detect matches. If
    ///                two patterns have the same rank, no order is guaranteed.
    /// * `swap_rules` - the swap rules that define whether two pieces can be swapped.
    ///                  If any rule returns false for two positions, the pieces are
    ///                  not swapped, and the swap method returns false. These rules
    ///                  are executed in the order provided after the default rule,
    ///                  so less expensive calculations should be done in earlier rules.
    pub fn new(piece_types: Vec<PieceType>, mut patterns: Vec<MatchPattern>,
               mut swap_rules: Vec<Box<dyn Fn(&Board, Pos, Pos) -> bool>>) -> Board {
        patterns.sort_by(|a, b| b.rank().cmp(&a.rank()));
        swap_rules.insert(0, Box::from(Board::are_pieces_movable));

        let num_types = piece_types.len();

        Board {
            piece_types,
            patterns,
            swap_rules,
            bitboard: BitBoard::default(num_types),
            last_changed: VecDeque::new()
        }
    }

    /// Gets a piece at the given position on the board. If the position is
    /// outside the board, a wall is returned. By default, all pieces on the
    /// board are walls.
    ///
    /// # Arguments
    ///
    /// * `pos` - position of the piece to get
    pub fn piece(&self, pos: Pos) -> Piece {
        match self.bitboard.piece(pos) {
            BitBoardPiece::Regular(piece_type, directions) => {
                Piece::Regular(
                    *self.piece_types.get(piece_type).expect("Unknown piece type"),
                    directions
                )
            }
            BitBoardPiece::Empty => Piece::Empty,
            BitBoardPiece::Wall => Piece::Wall
        }
    }

    /// Attempts to swap two pieces on the board. If any swap rule is broken (i.e. it
    /// results false), then the pieces will not be swapped, and this method will
    /// return false.
    ///
    /// If the swap is successful, both swapped positions will be marked for a match check.
    ///
    /// Swapping a piece in a direction in which it is marked unmovable is automatically
    /// a violation of the swap rules.
    ///
    /// Swapping with a piece that is empty is considered valid. The existing piece
    /// moves into the empty space while the other space is cleared. It is also valid to
    /// swap a piece with itself, though this has no effect on the board besides marking
    /// the piece for a match check.
    ///
    /// The order of two positions provided does not matter.
    ///
    /// # Arguments
    ///
    /// * `first` - the first position of a piece to swap
    /// * `second` - the second position of a piece to swap
    #[must_use]
    pub fn swap_pieces(&mut self, first: Pos, second: Pos) -> bool {
        if !self.swap_rules.iter().all(|rule| rule(self, first, second)) {
            return false;
        }

        self.last_changed.push_back(first);
        self.last_changed.push_back(second);

        let original_second_piece = self.piece(second);
        let original_first_piece = self.set_piece(first, original_second_piece);
        self.set_piece(second, original_first_piece);

        true
    }

    /// Replaces a piece at the given position and returns the previous piece.
    /// The space is marked as needing a match check. Swap rules do not apply
    /// and the replacement is always successful.
    ///
    /// # Arguments
    ///
    /// * `pos` - the position of the piece to replace
    /// * `piece` - the piece to put at the given position
    ///
    /// # Panics
    ///
    /// Panics if the type of the piece is not registered with tbe board.
    pub fn set_piece(&mut self, pos: Pos, piece: Piece) -> Piece {
        self.last_changed.push_back(pos);
        let old_piece = self.piece(pos);

        let new_bitboard_piece = match piece {
            Piece::Regular(piece_type, directions) => BitBoardPiece::Regular(
                self.piece_type_id(piece_type),
                directions
            ),
            Piece::Empty => BitBoardPiece::Empty,
            Piece::Wall => BitBoardPiece::Wall
        };

        self.bitboard = self.bitboard.replace_piece(pos, new_bitboard_piece);

        old_piece
    }

    /// Gets the next match on the board. Matches from pieces that were changed
    /// earlier are returned first. Matches are always based on the current board
    /// state, not the board state when the match occurred.
    ///
    /// Pieces that were changed but did not create a match are skipped.
    ///
    /// Regardless of whether a match is found, each piece is unmarked for a
    /// match check, unless it has been marked multiple times.
    pub fn next_match(&mut self) -> Option<Match> {
        let mut next_pos;
        let mut next_match = None;

        while next_match.is_none() {
            next_pos = self.last_changed.pop_front()?;
            next_match = self.patterns.iter().find_map(|pattern| {
                let positions = self.bitboard.check_match(
                    self.piece_type_id(pattern.piece_type()),
                    pattern.spaces(),
                    next_pos
                )?;
                Some(Match::new(pattern, next_pos, positions))
            });
        }

        next_match
    }

    /// Gets the ID of a piece type that is used by the bitboard.
    ///
    /// # Arguments
    ///
    /// * `piece_type` - the piece type to get the bitboard ID of
    ///
    /// # Panics
    ///
    /// Panics if the piece type is not known to this board.
    fn piece_type_id(&self, piece_type: PieceType) -> PieceTypeId {
        self.piece_types.iter().position(|&next_type| piece_type == next_type)
            .expect(&format!("The piece type \"{:?}\" is not registered with the game", piece_type))
    }

    /// Checks if the pieces at two positions on the board are both movable in the
    /// direction in which they would be swapped.
    ///
    /// # Arguments
    ///
    /// * `first` - the position of the first piece to check
    /// * `second` - the position of the second piece to check
    fn are_pieces_movable(&self, first: Pos, second: Pos) -> bool {
        let is_first_movable = Board::is_movable(first, second, self.piece(first));
        let is_second_movable = Board::is_movable(second, first, self.piece(second));

        is_first_movable && is_second_movable
    }

    /// Checks if a piece is movable vertically and horizontally.
    ///
    /// # Arguments
    ///
    /// * `from` - the current position of the piece
    /// * `to` - the position where the piece will be moved
    /// * `piece` - the at the "from" position
    fn is_movable(from: Pos, to: Pos, piece: Piece) -> bool {
        Board::is_vertically_movable(from, to, piece)
            && Board::is_horizontally_movable(from, to, piece)
    }

    /// Checks if a piece is vertically movable from one position to another.
    /// If there is no vertical change between the two positions, the piece
    /// is considered movable.
    ///
    /// # Arguments
    ///
    /// * `from` - the current position of the piece
    /// * `to` - the position where the piece will be moved
    /// * `piece` - the at the "from" position
    fn is_vertically_movable(from: Pos, to: Pos, piece: Piece) -> bool {
        let vertical_change = to.y() - from.y();
        if vertical_change > 0 {
            return piece.is_movable(Direction::North);
        } else if vertical_change < 0 {
            return piece.is_movable(Direction::South);
        }

        true
    }

    /// Checks if a piece is horizontally movable from one position to another.
    /// If there is no horizontal change between the two positions, the piece
    /// is considered movable.
    ///
    /// # Arguments
    ///
    /// * `from` - the current position of the piece
    /// * `to` - the position where the piece will be moved
    /// * `piece` - the at the "from" position
    fn is_horizontally_movable(from: Pos, to: Pos, piece: Piece) -> bool {
        let horizontal_change = to.x() - from.x();
        if horizontal_change > 0 {
            return piece.is_movable(Direction::East);
        } else if horizontal_change < 0 {
            return piece.is_movable(Direction::West);
        }

        true
    }

}

