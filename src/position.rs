use std::fmt::{Display, Formatter};
use std::ops::{Add, Sub};

use serde::{Serialize, Deserialize};

/// A position that represents a location in a two-dimensional plane.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Serialize, Deserialize)]
pub struct Pos {

    /// The horizontal component of the position
    x: u8,

    /// The vertical component of the position
    y: u8

}

impl Pos {

    /// Creates a new position with a horizontal and vertical component.
    ///
    /// # Arguments
    ///
    /// * `x` - the horizontal component of the position
    /// * `y` - the vertical component of the position
    pub fn new(x: u8, y: u8) -> Pos {
        Pos { x, y }
    }

    /// Returns the horizontal component of the position.
    pub fn x(&self) -> u8 {
        self.x
    }

    /// Returns the vertical component of the position.
    pub fn y(&self) -> u8 {
        self.y
    }

}

impl Add for Pos {
    type Output = Pos;

    /// Returns the component-wise sum of two positions.
    ///
    /// # Arguments
    ///
    /// * `rhs` - the "right-hand side" of the addition and the other point to sum
    ///
    /// # Panics
    ///
    /// Panics in debug move if the 8-bit integer limit is overflowed.
    fn add(self, rhs: Self) -> Self::Output {
        Pos {x: self.x + rhs.x, y: self.y + rhs.y}
    }

}

impl Sub for Pos {
    type Output = Pos;

    /// Returns the component-wise difference of two positions.
    ///
    /// # Arguments
    ///
    /// * `rhs` - the "right-hand side" of the subtraction and the point to subtract from this point
    ///
    /// # Panics
    ///
    /// Subtraction will normally panic in debug mode if there is integer overflow, so the
    /// components of self should be larger than those of the right hand side.
    fn sub(self, rhs: Self) -> Self::Output {
        Pos {x: self.x - rhs.x, y: self.y - rhs.y}
    }

}

impl Display for Pos {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use crate::position::Pos;

    #[test]
    fn new_positive_components_allowed() {
        let pos = Pos::new(1, 4);
        assert_eq!(1, pos.x());
        assert_eq!(4, pos.y());
    }

    #[test]
    fn add_positive_components_summed() {
        let pos1 = Pos::new(1, 4);
        let pos2 = Pos::new(15, 5);
        let sum = pos1 + pos2;
        assert_eq!(16, sum.x());
        assert_eq!(9, sum.y());
    }

    #[test]
    fn sub_positive_components_subtracted() {
        let pos1 = Pos::new(1, 4);
        let pos2 = Pos::new(15, 5);
        let diff = pos2 - pos1;
        assert_eq!(14, diff.x());
        assert_eq!(1, diff.y());
    }

    #[test]
    fn equals_same_pos_equal() {
        let pos1 = Pos::new(1, 4);
        let pos2 = Pos::new(1, 4);
        assert_eq!(pos1, pos2);
    }

    #[test]
    fn equals_components_diff_not_equal() {
        let pos1 = Pos::new(1, 4);
        let pos2 = Pos::new(0, 15);
        assert_ne!(pos1, pos2);
    }

    #[test]
    fn equals_components_reversed_not_equal() {
        let pos1 = Pos::new(1, 4);
        let pos2 = Pos::new(4, 1);
        assert_ne!(pos1, pos2);
    }

    #[test]
    fn format_positive_components_no_signs() {
        let pos = Pos::new(1, 4);
        assert_eq!("(1, 4)", format!("{}", pos));
    }
}