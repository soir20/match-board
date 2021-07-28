use enumset::enum_set;
use enumset::EnumSet;
use enumset::EnumSetType;

#[derive(Eq, PartialEq)]
pub struct PieceType {
    name: String
}

#[derive(EnumSetType)]
pub enum Direction {
    North,
    South,
    East,
    West
}
const ALL_DIRECTIONS: EnumSet<Direction> = enum_set!(
    Direction::North | Direction::South | Direction::East | Direction::West
);

pub struct Piece {
    piece_type: PieceType,
    movable_directions: EnumSet<Direction>
}

impl Piece {
    pub fn new(piece_type: PieceType) -> Piece {
        Piece { piece_type, movable_directions: ALL_DIRECTIONS }
    }

    pub fn make_movable(&mut self, direction: Direction) {
        self.movable_directions.insert(direction);
    }

    pub fn make_movable_all(&mut self) {
        self.movable_directions = ALL_DIRECTIONS;
    }

    pub fn make_unmovable(&mut self, direction: Direction) {
        self.movable_directions.remove(direction);
    }

    pub fn make_unmovable_all(&mut self) {
        self.movable_directions.clear();
    }

    pub fn is_movable(&self, direction: Direction) -> bool {
        self.movable_directions.contains(direction)
    }

    pub fn get_type(&self) -> &PieceType {
        &self.piece_type
    }
}