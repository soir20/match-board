use enumset::enum_set;
use enumset::EnumSet;
use enumset::EnumSetType;

/// A unique category for board pieces.
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct PieceType {
    name: &'static str
}

impl PieceType {

    /// Creates a new piece type.
    /// # Arguments
    ///
    /// * `name` - the name of the type. Must be unique among all other piece types.
    pub fn new(name: &'static str) -> PieceType {
        PieceType { name }
    }

}

/// A direction that a piece could move.
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

/// An individual, possibly-movable piece on a board that belongs to a category.
pub struct Piece {
    piece_type: PieceType,
    movable_directions: EnumSet<Direction>
}

impl Piece {
    pub fn new(piece_type: PieceType) -> Piece {
        Piece { piece_type, movable_directions: ALL_DIRECTIONS }
    }

    /// Makes the piece movable in one direction, not affecting its movability in any
    /// other direction. Does not need to be called unless the piece has been made
    /// unmovable in the given direction.
    ///
    /// # Arguments
    ///
    /// * `direction` - the direction in which to make the piece movable
    pub fn make_movable(&mut self, direction: Direction) {
        self.movable_directions.insert(direction);
    }

    /// Convenience method to make the piece movable in all directions. Does not need
    /// to be called unless the piece has been made unmovable in a direction.
    pub fn make_movable_all(&mut self) {
        self.movable_directions = ALL_DIRECTIONS;
    }

    /// Makes a piece unmovable in a direction. An unmovable piece cannot be swapped,
    /// though it can be replaced on the board.
    ///
    /// # Arguments
    ///
    /// * `direction` - the direction in which to make the piece unmovable
    pub fn make_unmovable(&mut self, direction: Direction) {
        self.movable_directions.remove(direction);
    }

    /// Makes a piece unmovable in all directions. An unmovable piece cannot be swapped,
    /// though it can be replaced on the board.
    pub fn make_unmovable_all(&mut self) {
        self.movable_directions.clear();
    }

    /// Checks if a piece is movable in a given direction.
    ///
    /// # Arguments
    ///
    /// * `direction` - the direction in which to test if the piece is movable
    pub fn is_movable(&self, direction: Direction) -> bool {
        self.movable_directions.contains(direction)
    }

    /// Gets the type that this piece belongs to.
    pub fn get_type(&self) -> &PieceType {
        &self.piece_type
    }

}