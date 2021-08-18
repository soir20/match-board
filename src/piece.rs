use enumset::enum_set;
use enumset::EnumSet;
use enumset::EnumSetType;

/// A unique category for board pieces.
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
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
    North = 0,
    South = 1,
    East = 2,
    West = 3
}

impl Direction {

    /// Converts the enum to a unique numerical value.
    pub fn value(&self) -> usize {
        match *self {
            Direction::North => 0,
            Direction::South => 1,
            Direction::East => 2,
            Direction::West => 3
        }
    }

}

pub const ALL_DIRECTIONS: EnumSet<Direction> = enum_set!(
    Direction::North | Direction::South | Direction::East | Direction::West
);

/// An individual, possibly-movable piece on a board that belongs to a category.
///
/// Empty pieces are always movable, while walls are never movable.
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub enum Piece {
    Regular(PieceType, EnumSet<Direction>),
    Empty,
    Wall
}

impl Piece {

    /// Checks if a piece is movable in a given direction.
    ///
    /// # Arguments
    ///
    /// * `direction` - the direction in which to test if the piece is movable
    pub fn is_movable(&self, direction: Direction) -> bool {
        match *self {
            Piece::Regular(_, ref directions) => directions.contains(direction),
            Piece::Empty => true,
            Piece::Wall => false
        }

    }

}

#[cfg(test)]
mod tests {
}
