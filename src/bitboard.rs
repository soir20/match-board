use crate::position::Pos;
use std::collections::{HashSet, VecDeque};

const BOARD_WIDTH: usize = 32;
type Pattern = HashSet<Pos>;
type Grid = [u32; BOARD_WIDTH];

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
            piece_grids.push([0; BOARD_WIDTH]);
        }

        BitBoard {
            pieces: piece_grids,
            empty_pieces: [0; BOARD_WIDTH],
            movable_directions: [[0; BOARD_WIDTH]; 4]
        }
    }

    pub fn new(pieces: Vec<Grid>, empty_pieces: Grid, movable_directions: [Grid; 4]) -> BitBoard {
        BitBoard { pieces, empty_pieces, movable_directions }
    }

    pub fn find_match(&self, piece_type: usize, patterns: Vec<Pattern>, pos: Pos) -> Option<Pattern> {
        let grid = self.pieces.get(piece_type).expect("Unknown piece type");
        patterns.into_iter().find_map(|pattern| BitBoard::check_pattern(grid, pattern, pos))
    }

    pub fn trickle(&self) -> BitBoard {
        let mut mutable_board = MutableBitBoard::from(self.clone());
        for x in 0..self.pieces.len() {
            mutable_board.trickle_column(x);
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
    pub fn flatten(&mut self) {
        for x in 0..BOARD_WIDTH as u32 {
            for y in 0..BOARD_WIDTH as u32 {
                self.flatten_to(Pos::new(x, y));
            }
        }
    }

    pub fn trickle_column(&mut self, x: usize) {
        let empty_column = self.empty_pieces[x];
        let movable_south = self.movable_directions[1][x];

        let mut empty_spaces = VecDeque::new();

        for y in 0..BOARD_WIDTH as u32 {
            if is_set_in_column(empty_column, y) {
                empty_spaces.push_back(y);
            } else if is_set_in_column(movable_south, y) {
                if let Some(space_to_fill) = empty_spaces.pop_front() {
                    self.swap_piece_and_empty_in_column(x, y, space_to_fill);
                }
            } else {
                empty_spaces.clear();
            }
        }
    }

    fn flatten_to(&mut self, empty_pos: Pos) {
        let closest_movable = self.find_closest_movable(empty_pos);

        if let Some(current_piece_pos) = closest_movable {
            self.swap_piece_and_empty_across_columns(current_piece_pos, empty_pos);
            self.trickle_column(current_piece_pos.x() as usize);
        }
    }

    fn find_closest_movable(&self, empty_pos: Pos) -> Option<Pos> {
        let x = empty_pos.x();
        let y = empty_pos.y();

        let mut to_east_y = u32::MAX;
        let mut to_west_y = u32::MAX;

        if let Some(east_y) = self.find_movable_in_column(x - 1, y + 1, true) {
            to_east_y = east_y
        }

        if let Some(west_y) = self.find_movable_in_column(x + 1, y + 1, false) {
            to_west_y = west_y;
        }

        if to_east_y == u32::MAX && to_west_y == u32::MAX {
            return None;
        }

        Some(match to_east_y >= to_west_y {
            true => Pos::new(x - 1, to_east_y),
            false => Pos::new(x + 1, to_west_y)
        })
    }

    fn find_movable_in_column(&self, x: u32, floor: u32, move_east: bool) -> Option<u32> {
        if (move_east && x == (BOARD_WIDTH - 1) as u32) || (!move_east && x == 0) {
            return None;
        }

        let x_index = (x - 1) as usize;
        let direction_index = match move_east {
            true => 2,
            false => 3
        };
        let movable_in_column = self.movable_directions[direction_index][x_index]
            & self.empty_pieces[x_index];

        for y in floor..BOARD_WIDTH as u32 {
            if is_set_in_column(movable_in_column, y) {
                return Some(y);
            }
        }

        None
    }

    fn swap_piece_and_empty_across_columns(&mut self, piece: Pos, empty: Pos) {
        let piece_x = piece.x() as usize;
        let empty_x = empty.x() as usize;

        let piece_type = self.find_piece_type(piece)
            .expect(&*format!("Piece does not exist at {}", piece));
        let type_grid = self.pieces.get_mut(piece_type).expect("Found type doesn't exist");

        let swapped_type_columns = swap_across_columns(
            type_grid[piece_x], type_grid[empty_x],
            piece.y(), empty.y()
        );
        type_grid[piece_x] = swapped_type_columns.0;
        type_grid[empty_x] = swapped_type_columns.1;

        let swapped_empty_columns = swap_across_columns(
            self.empty_pieces[piece_x], self.empty_pieces[empty_x],
            piece.y(), empty.y()
        );
        self.empty_pieces[piece_x] = swapped_empty_columns.0;
        self.empty_pieces[empty_x] = swapped_empty_columns.1;

        self.movable_directions.iter_mut().for_each(|direction_grid| {
            let swapped_direction_columns = swap_across_columns(
                direction_grid[piece_x], direction_grid[empty_x],
                piece.y(), empty.y()
            );
            direction_grid[piece_x] = swapped_direction_columns.0;
            direction_grid[empty_x] = swapped_direction_columns.1;
        });
    }

    fn swap_piece_and_empty_in_column(&mut self, x: usize, piece_y: u32, empty_y: u32) {
        let original_pos = Pos::new(x as u32, piece_y);
        let piece_type = self.find_piece_type(original_pos)
            .expect(&*format!("Piece does not exist at {}", original_pos));

        let type_grid = self.pieces.get_mut(piece_type).expect("Found type doesn't exist");
        type_grid[x] = swap_in_column(type_grid[x], piece_y, empty_y);

        self.empty_pieces[x] = swap_in_column(self.empty_pieces[x], piece_y, empty_y);

        self.movable_directions.iter_mut().for_each(|direction_grid| {
            direction_grid[x] = swap_in_column(direction_grid[x], piece_y, empty_y);
        });
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

fn set_in_column(column: u32, y: u32) -> u32 {
    column | (1 << y)
}

fn unset_in_column(column: u32, y: u32) -> u32 {
    column & !(1 << y)
}

fn swap_in_column(column: u32, from_y: u32, to_y: u32) -> u32 {
    let from_bit = (column >> from_y) & 1;
    let to_bit = (column >> to_y) & 1;

    let mut swapped_column = unset_in_column(column, from_y);
    swapped_column = unset_in_column(swapped_column, to_y);

    swapped_column | (from_bit >> to_y) | (to_bit >> from_y)
}

fn swap_across_columns(from_column: u32, to_column: u32, from_y: u32, to_y: u32) -> (u32, u32) {
    let from_bit = (from_column >> from_y) & 1;
    let to_bit = (to_column >> to_y) & 1;

    let swapped_from_column = from_column ^ (to_bit >> from_y);
    let swapped_to_column = from_column ^ (from_bit >> to_y);

    (swapped_from_column, swapped_to_column)
}