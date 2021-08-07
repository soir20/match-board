use crate::position::Pos;
use std::collections::{HashSet, VecDeque};

type Pattern = HashSet<Pos>;
type Grid = [u32; 32];

#[derive(Clone)]
pub(crate) struct BitBoard {
    pieces: Vec<Grid>,
    empty_pieces: Grid,
    movable_directions: [Grid; 4]
}

impl BitBoard {

    pub fn default(num_piece_types: usize) -> BitBoard {
        let mut piece_grids = Vec::new();

        for _ in 0..num_piece_types {
            piece_grids.push([0; 32]);
        }

        BitBoard { pieces: piece_grids, empty_pieces: [0; 32], movable_directions: [[0; 32]; 4] }
    }

    pub fn new(pieces: Vec<Grid>, empty_pieces: Grid, movable_directions: [Grid; 4]) -> BitBoard {
        BitBoard { pieces, empty_pieces, movable_directions }
    }

    pub fn find_match(&self, piece_type: usize, patterns: Vec<Pattern>, pos: Pos) -> Option<Pattern> {
        let grid = self.pieces.get(piece_type).expect("Unknown piece type");
        patterns.into_iter().find_map(|pattern| BitBoard::check_pattern(grid, pattern, pos))
    }

    pub fn trickle(&self, floor: u32) -> BitBoard {
        let mut mutable_board = MutableBitBoard::from(self.clone());
        for x in 0..self.pieces.len() {
            mutable_board.trickle_column(floor, x);
        }
        mutable_board.into()
    }

    fn check_pattern(grid: &Grid, pattern: Pattern, pos: Pos) -> Option<Pattern> {
        pattern.iter().find_map(|&original| BitBoard::check_variant(grid, pattern, pos - original))
    }

    fn check_variant(grid: &Grid, pattern: Pattern, new_origin: Pos) -> Option<Pattern> {
        let grid_pos = BitBoard::change_origin(pattern, new_origin);
        match grid_pos.iter().all(|&pos| is_set_in_grid(grid, pos)) {
            true => Some(grid_pos),
            false => None
        }
    }

    fn change_origin(pattern: Pattern, origin: Pos) -> Pattern {
        pattern.iter().map(|&original| original + origin).collect()
    }
}

pub(crate) struct MutableBitBoard {
    pieces: Vec<Grid>,
    empty_pieces: Grid,
    movable_directions: [Grid; 4]
}

impl From<BitBoard> for MutableBitBoard {
    fn from(bit_board: BitBoard) -> Self {
        MutableBitBoard {
            pieces: bit_board.pieces,
            empty_pieces: bit_board.empty_pieces,
            movable_directions: bit_board.movable_directions
        }
    }
}

impl Into<BitBoard> for MutableBitBoard {
    fn into(self) -> BitBoard {
        BitBoard::new(self.pieces, self.empty_pieces, self.movable_directions)
    }
}

impl MutableBitBoard {
    pub fn trickle_column(&mut self, floor: u32, x: usize) {
        let empty_column = self.empty_pieces[x];
        let movable_south = self.movable_directions[3][x];

        let mut empty_spaces = VecDeque::new();

        for y in floor..32 {
            if is_set_in_column(empty_column, y) {
                empty_spaces.push_back(y);
            } else if is_set_in_column(movable_south, y) {
                if let Some(space_to_fill) = empty_spaces.pop_front() {
                    self.swap_in_column(x, y, space_to_fill);
                }
            } else {
                empty_spaces.clear();
            }
        }
    }

    fn swap_in_column(&mut self, x: usize, from_y: u32, to_y: u32) {
        let original_pos = Pos::new(x as u32, from_y);
        let piece_type = self.find_piece_type(original_pos)
            .expect("Missing piece marked as movable");

        let type_grid = self.pieces.get_mut(piece_type).expect("Found type doesn't exist");
        type_grid[x] = MutableBitBoard::swap_single_column(type_grid[x], from_y, to_y);

        self.empty_pieces[x] = MutableBitBoard::swap_single_column(self.empty_pieces[x], from_y, to_y);

        self.movable_directions.iter_mut().for_each(|direction_grid| {
            direction_grid[x] = MutableBitBoard::swap_single_column(direction_grid[x], from_y, to_y);
        });
    }

    fn swap_single_column(column: u32, from_y: u32, to_y: u32) -> u32 {
        let mut swapped_column = flip_in_column(column, from_y);
        swapped_column = flip_in_column(column, to_y);
        swapped_column
    }

    fn find_piece_type(&self, pos: Pos) -> Option<usize> {
        self.pieces.iter().enumerate().find_map(|(index, grid)|
            match is_set_in_grid(grid, pos) {
                true => Some(index),
                false => None
            }
        )
    }
}

fn is_set_in_grid(grid: &Grid, pos: Pos) -> bool {
    is_set_in_column(grid[pos.x() as usize], pos.y())
}

fn is_set_in_column(column: u32, y: u32) -> bool {
    (column >> y) & 1 == 1
}

fn flip_in_column(column: u32, y: u32) -> u32 {
    column ^ (1 << y)
}