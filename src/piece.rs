use enumset::enum_set;
use enumset::EnumSet;
use enumset::EnumSetType;
use std::fmt::{self, Debug, Display, Formatter};
use serde::{Serialize, Deserialize};

/// A unique category for board pieces.
pub type PieceType = char;

/// A direction that a piece could move.
#[derive(EnumSetType, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub enum Direction {
    North = 0,
    South = 1,
    East = 2,
    West = 3
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

pub const ALL_DIRECTIONS: EnumSet<Direction> = enum_set!(
    Direction::North | Direction::South | Direction::East | Direction::West
);

/// An individual, possibly-movable piece on a board that belongs to a category.
///
/// Empty pieces are always movable, while walls are never movable.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
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

impl Display for Piece {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match *self {
            Piece::Regular(piece_type, _) => piece_type,
            Piece::Empty => ' ',
            Piece::Wall => '#'
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::piece::Direction;

    #[test]
    fn display_direction_north_is_direction_name() {
        assert_eq!("North", format!("{}", Direction::North));
    }

    #[test]
    fn display_direction_south_is_direction_name() {
        assert_eq!("South", format!("{}", Direction::South));
    }

    #[test]
    fn display_direction_east_is_direction_name() {
        assert_eq!("East", format!("{}", Direction::East));
    }

    #[test]
    fn display_direction_west_is_direction_name() {
        assert_eq!("West", format!("{}", Direction::West));
    }
}
