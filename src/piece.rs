use std::fmt::{Debug, Display, Formatter};

use enumset::enum_set;
use enumset::EnumSet;
use enumset::EnumSetType;
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
    use crate::piece::{Direction, Piece, ALL_DIRECTIONS};
    use enumset::enum_set;

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

    #[test]
    fn is_movable_piece_regular_north_true() {
        assert!(Piece::Regular('t', enum_set!(Direction::North)).is_movable(Direction::North));
    }

    #[test]
    fn is_movable_piece_regular_north_false() {
        assert!(!Piece::Regular('t', enum_set!()).is_movable(Direction::North));
    }

    #[test]
    fn is_movable_piece_regular_south_true() {
        assert!(Piece::Regular('t', enum_set!(Direction::South)).is_movable(Direction::South));
    }

    #[test]
    fn is_movable_piece_regular_south_false() {
        assert!(!Piece::Regular('t', enum_set!()).is_movable(Direction::South));
    }

    #[test]
    fn is_movable_piece_regular_east_true() {
        assert!(Piece::Regular('t', enum_set!(Direction::East)).is_movable(Direction::East));
    }

    #[test]
    fn is_movable_piece_regular_east_false() {
        assert!(!Piece::Regular('t', enum_set!()).is_movable(Direction::East));
    }

    #[test]
    fn is_movable_piece_regular_west_true() {
        assert!(Piece::Regular('t', enum_set!(Direction::West)).is_movable(Direction::West));
    }

    #[test]
    fn is_movable_piece_regular_west_false() {
        assert!(!Piece::Regular('t', enum_set!()).is_movable(Direction::West));
    }

    #[test]
    fn is_movable_piece_empty_north_true() {
        assert!(Piece::Empty.is_movable(Direction::North));
    }

    #[test]
    fn is_movable_piece_empty_south_true() {
        assert!(Piece::Empty.is_movable(Direction::South));
    }

    #[test]
    fn is_movable_piece_empty_east_true() {
        assert!(Piece::Empty.is_movable(Direction::East));
    }

    #[test]
    fn is_movable_piece_east_west_true() {
        assert!(Piece::Empty.is_movable(Direction::West));
    }

    #[test]
    fn is_movable_piece_wall_north_false() {
        assert!(!Piece::Wall.is_movable(Direction::North));
    }

    #[test]
    fn is_movable_piece_wall_south_false() {
        assert!(!Piece::Wall.is_movable(Direction::South));
    }

    #[test]
    fn is_movable_piece_wall_east_false() {
        assert!(!Piece::Wall.is_movable(Direction::East));
    }

    #[test]
    fn is_movable_piece_wall_west_false() {
        assert!(!Piece::Wall.is_movable(Direction::West));
    }

    #[test]
    fn display_piece_regular_type() {
        assert_eq!("t", format!("{}", Piece::Regular('t', ALL_DIRECTIONS)));
    }

    #[test]
    fn display_piece_empty_space() {
        assert_eq!(" ", format!("{}", Piece::Empty));
    }

    #[test]
    fn display_piece_wall_pound() {
        assert_eq!("#", format!("{}", Piece::Wall));
    }
}
