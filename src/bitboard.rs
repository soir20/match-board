use crate::position::Pos;
use std::collections::{HashSet, VecDeque};
use crate::piece::{Direction, ALL_DIRECTIONS};
use enumset::EnumSet;

const BOARD_WIDTH: u8 = 32;
type Pattern = HashSet<Pos>;
type Grid = [u32; BOARD_WIDTH as usize];
type PieceTypeId = usize;

enum BitBoardPiece {
    Regular(PieceTypeId, EnumSet<Direction>),
    Empty,
    Wall
}

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
            piece_grids.push([0; BOARD_WIDTH as usize]);
        }

        BitBoard {
            pieces: piece_grids,
            empty_pieces: [0; BOARD_WIDTH as usize],
            movable_directions: [[0; BOARD_WIDTH as usize]; 4]
        }
    }

    pub fn new(pieces: Vec<Grid>, empty_pieces: Grid, movable_directions: [Grid; 4]) -> BitBoard {
        BitBoard { pieces, empty_pieces, movable_directions }
    }

    pub fn piece(&self, pos: Pos) -> BitBoardPiece {
        if !is_within_board(pos) {
            return BitBoardPiece::Wall;
        }

        let x_index = pos.x() as usize;
        if is_set_in_column(self.empty_pieces[x_index], pos.y()) {
            return BitBoardPiece::Empty;
        }

        match find_piece_type(&self.pieces, pos) {
            None => BitBoardPiece::Wall,
            Some(piece_type) => {
                let mut movable_directions = EnumSet::new();

                for direction in ALL_DIRECTIONS {
                    if is_set_in_column(self.movable_directions[direction.value()][x_index], pos.y()) {
                        movable_directions.insert(direction);
                    }
                }

                BitBoardPiece::Regular(piece_type, movable_directions)
            }
        }
    }

    pub fn find_match(&self, piece_type: PieceTypeId, patterns: Vec<Pattern>, pos: Pos) -> Option<Pattern> {
        let grid = self.pieces.get(piece_type).expect("Unknown piece type");
        patterns.into_iter().find_map(|pattern| BitBoard::check_pattern(grid, pattern, pos))
    }

    pub fn trickle(&self) -> BitBoard {
        let mut mutable_board = MutableBitBoard::from(self.clone());
        for x in 0..BOARD_WIDTH {
            mutable_board.trickle_column(x);
        }

        mutable_board.trickle_diagonally();
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

struct MutableBitBoard {
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
    pub fn trickle_column(&mut self, x: u8) {
        let x_index = x as usize;
        let empty_column = self.empty_pieces[x_index];
        let movable_south = self.movable_directions[Direction::South.value()][x_index];

        let mut empty_spaces = VecDeque::new();

        for y in 0..BOARD_WIDTH {
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

    pub fn trickle_diagonally(&mut self) {
        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_WIDTH {
                let piece_pos = Pos::new(x, y);

                let mut did_trickle = false;

                let mut previous_trickled_pos = piece_pos;
                let mut current_trickled_pos = self.trickle_piece(previous_trickled_pos);
                while previous_trickled_pos != current_trickled_pos {
                    previous_trickled_pos = current_trickled_pos;
                    current_trickled_pos = self.trickle_piece(previous_trickled_pos);
                    did_trickle = true;
                }

                if did_trickle {
                    self.trickle_column(x);
                }
            }
        }
    }

    fn swap_piece_and_empty_in_column(&mut self, x: u8, piece_y: u8, empty_y: u8) {
        let x_index = x as usize;
        let original_pos = Pos::new(x, piece_y);
        let piece_type = find_piece_type(&self.pieces, original_pos)
            .expect(&*format!("Piece does not exist at {}", original_pos));

        let type_grid = self.pieces.get_mut(piece_type).expect("Found type doesn't exist");
        type_grid[x_index] = swap_in_column(type_grid[x_index], piece_y, empty_y);

        self.empty_pieces[x_index] = swap_in_column(self.empty_pieces[x_index], piece_y, empty_y);

        self.movable_directions.iter_mut().for_each(|direction_grid| {
            direction_grid[x_index] = swap_in_column(direction_grid[x_index], piece_y, empty_y);
        });
    }

    fn trickle_piece(&mut self, piece_pos: Pos) -> Pos {
        let mut diagonally_trickled_pos = self.trickle_piece_diagonally(piece_pos, true);
        if diagonally_trickled_pos == piece_pos {
            diagonally_trickled_pos = self.trickle_piece_diagonally(piece_pos, false);
        }

        self.trickle_piece_down(diagonally_trickled_pos)
    }

    fn trickle_piece_diagonally(&mut self, current_pos: Pos, to_west: bool) -> Pos {
        let original_x = current_pos.x() as usize;

        let mut piece_pos = current_pos;
        let mut empty_pos = MutableBitBoard::move_pos_down_diagonally(piece_pos, to_west);

        let horizontal_dir_col = match to_west {
            true => self.movable_directions[Direction::West.value()][original_x],
            false => self.movable_directions[Direction::East.value()][original_x]
        };
        let vertical_dir_col = self.movable_directions[Direction::South.value()][original_x as usize];

        if !is_set_in_column(horizontal_dir_col, current_pos.y()) ||
            !is_set_in_column(vertical_dir_col, current_pos.y()) {
            return piece_pos;
        }

        while is_within_board(empty_pos)
            && is_set_in_column(self.empty_pieces[empty_pos.x() as usize], empty_pos.y()) {

            self.swap_piece_and_empty_across_columns(piece_pos, empty_pos);
            piece_pos = empty_pos;

            empty_pos = MutableBitBoard::move_pos_down_diagonally(piece_pos, to_west);
        }

        piece_pos
    }

    fn trickle_piece_down(&mut self, piece_pos: Pos) -> Pos {
        let x_index = piece_pos.x() as usize;

        let vertical_dir_col = self.movable_directions[Direction::South.value()][x_index as usize];
        if !is_set_in_column(vertical_dir_col, piece_pos.y()) {
            return piece_pos;
        }

        let mut next_y = piece_pos.y();
        while next_y > 0 && is_set_in_column(self.empty_pieces[x_index], next_y - 1) {
            next_y -= 1;
        }
        self.swap_piece_and_empty_in_column(piece_pos.x(), piece_pos.y(), next_y);

        Pos::new(piece_pos.x(), next_y)
    }

    fn swap_piece_and_empty_across_columns(&mut self, piece: Pos, empty: Pos) {
        let piece_x = piece.x() as usize;
        let empty_x = empty.x() as usize;

        let piece_type = find_piece_type(&self.pieces, piece)
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

    fn move_pos_down_diagonally(pos: Pos, to_west: bool) -> Pos {
        match to_west {
            true => Pos::new(pos.x() - 1, pos.y() - 1),
            false => Pos::new(pos.x() + 1, pos.y() - 1)
        }
    }
}

fn find_piece_type(pieces: &Vec<Grid>, pos: Pos) -> Option<PieceTypeId> {
    pieces.iter().enumerate().find_map(|(index, grid)|
        match is_set_in_grid(grid, pos) {
            true => Some(index),
            false => None
        }
    )
}

fn is_within_board(pos: Pos) -> bool {
    pos.x() >= 0 && pos.x() < BOARD_WIDTH && pos.y() >= 0 && pos.y() < BOARD_WIDTH
}

fn is_set_in_grid(grid: &Grid, pos: Pos) -> bool {
    is_set_in_column(grid[pos.x() as usize], pos.y())
}

fn is_set_in_column(column: u32, y: u8) -> bool {
    (column >> y) & 1 == 1
}

fn flip_in_column(column: u32, y: u8) -> u32 {
    column ^ (1 << y)
}

fn set_in_column(column: u32, y: u8) -> u32 {
    column | (1 << y)
}

fn unset_in_column(column: u32, y: u8) -> u32 {
    column & !(1 << y)
}

fn swap_in_column(column: u32, from_y: u8, to_y: u8) -> u32 {
    let from_bit = (column >> from_y) & 1;
    let to_bit = (column >> to_y) & 1;

    let mut swapped_column = unset_in_column(column, from_y);
    swapped_column = unset_in_column(swapped_column, to_y);

    swapped_column | (from_bit >> to_y) | (to_bit >> from_y)
}

fn swap_across_columns(from_column: u32, to_column: u32, from_y: u8, to_y: u8) -> (u32, u32) {
    let from_bit = (from_column >> from_y) & 1;
    let to_bit = (to_column >> to_y) & 1;

    let swapped_from_column = from_column ^ (to_bit >> from_y);
    let swapped_to_column = from_column ^ (from_bit >> to_y);

    (swapped_from_column, swapped_to_column)
}