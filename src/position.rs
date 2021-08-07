use std::ops::{Add, Sub};
use std::fmt::{Display, Formatter};

/// A position that represents a location in a two-dimensional plane.
#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub struct Pos {

    /// The horizontal component of the position
    x: u32,

    /// The vertical component of the position
    y: u32

}

impl Pos {

    /// Creates a new position with a horizontal and vertical component.
    ///
    /// # Arguments
    ///
    /// * `x` - the horizontal component of the position
    /// * `y` - the vertical component of the position
    pub fn new(x: u32, y: u32) -> Pos {
        Pos { x, y }
    }

    /// Returns the horizontal component of the position.
    pub fn x(&self) -> u32 {
        self.x
    }

    /// Returns the vertical component of the position.
    pub fn y(&self) -> u32 {
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
    fn sub(self, rhs: Self) -> Self::Output {
        Pos {x: self.x - rhs.x, y: self.y - rhs.y}
    }

}

impl Display for Pos {

    /// Returns a formatted position as (horizontal, vertical).
    ///
    /// # Arguments
    ///
    /// * `formatter` - represents formatting options. Used internally by Rust.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "({}, {})", self.x, self.y)
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
    fn new_negative_x_component_allowed() {
        let pos = Pos::new(-1, 4);
        assert_eq!(-1, pos.x());
        assert_eq!(4, pos.y());
    }

    #[test]
    fn new_negative_y_component_allowed() {
        let pos = Pos::new(1, -4);
        assert_eq!(1, pos.x());
        assert_eq!(-4, pos.y());
    }

    #[test]
    fn new_negative_components_allowed() {
        let pos = Pos::new(-1, -4);
        assert_eq!(-1, pos.x());
        assert_eq!(-4, pos.y());
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
    fn add_negative_first_x_component_summed() {
        let pos1 = Pos::new(-1, 4);
        let pos2 = Pos::new(15, 5);
        let sum = pos1 + pos2;
        assert_eq!(14, sum.x());
        assert_eq!(9, sum.y());
    }

    #[test]
    fn add_negative_first_y_component_summed() {
        let pos1 = Pos::new(1, -4);
        let pos2 = Pos::new(15, 5);
        let sum = pos1 + pos2;
        assert_eq!(16, sum.x());
        assert_eq!(1, sum.y());
    }

    #[test]
    fn add_negative_second_x_component_summed() {
        let pos1 = Pos::new(1, 4);
        let pos2 = Pos::new(-15, 5);
        let sum = pos1 + pos2;
        assert_eq!(-14, sum.x());
        assert_eq!(9, sum.y());
    }

    #[test]
    fn add_negative_second_y_component_summed() {
        let pos1 = Pos::new(1, 4);
        let pos2 = Pos::new(15, -5);
        let sum = pos1 + pos2;
        assert_eq!(16, sum.x());
        assert_eq!(-1, sum.y());
    }

    #[test]
    fn add_negative_all_components_summed() {
        let pos1 = Pos::new(-1, -4);
        let pos2 = Pos::new(-15, -5);
        let sum = pos1 + pos2;
        assert_eq!(-16, sum.x());
        assert_eq!(-9, sum.y());
    }

    #[test]
    fn sub_positive_components_subtracted() {
        let pos1 = Pos::new(1, 4);
        let pos2 = Pos::new(15, 5);
        let diff = pos1 - pos2;
        assert_eq!(-14, diff.x());
        assert_eq!(-1, diff.y());
    }

    #[test]
    fn sub_negative_first_x_component_subtracted() {
        let pos1 = Pos::new(-1, 4);
        let pos2 = Pos::new(15, 5);
        let diff = pos1 - pos2;
        assert_eq!(-16, diff.x());
        assert_eq!(-1, diff.y());
    }

    #[test]
    fn sub_negative_first_y_component_subtracted() {
        let pos1 = Pos::new(1, -4);
        let pos2 = Pos::new(15, 5);
        let diff = pos1 - pos2;
        assert_eq!(-14, diff.x());
        assert_eq!(-9, diff.y());
    }

    #[test]
    fn sub_negative_second_x_component_subtracted() {
        let pos1 = Pos::new(1, 4);
        let pos2 = Pos::new(-15, 5);
        let diff = pos1 - pos2;
        assert_eq!(16, diff.x());
        assert_eq!(-1, diff.y());
    }

    #[test]
    fn sub_negative_second_y_component_subtracted() {
        let pos1 = Pos::new(1, 4);
        let pos2 = Pos::new(15, -5);
        let diff = pos1 - pos2;
        assert_eq!(-14, diff.x());
        assert_eq!(9, diff.y());
    }

    #[test]
    fn sub_negative_all_components_subtracted() {
        let pos1 = Pos::new(-1, -4);
        let pos2 = Pos::new(-15, -5);
        let diff = pos1 - pos2;
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
    fn equals_both_signs_diff_not_equal() {
        let pos1 = Pos::new(1, 4);
        let pos2 = Pos::new(-1, -4);
        assert_ne!(pos1, pos2);
    }

    #[test]
    fn equals_x_signs_diff_not_equal() {
        let pos1 = Pos::new(1, 4);
        let pos2 = Pos::new(-1, 4);
        assert_ne!(pos1, pos2);
    }

    #[test]
    fn equals_y_signs_diff_not_equal() {
        let pos1 = Pos::new(1, 4);
        let pos2 = Pos::new(1, -4);
        assert_ne!(pos1, pos2);
    }

    #[test]
    fn equals_components_diff_not_equal() {
        let pos1 = Pos::new(1, 4);
        let pos2 = Pos::new(0, 15);
        assert_ne!(pos1, pos2);
    }

    #[test]
    fn format_positive_components_no_signs() {
        let pos = Pos::new(1, 4);
        assert_eq!("(1, 4)", format!("{}", pos));
    }

    #[test]
    fn format_negative_x_component_correct_signs() {
        let pos = Pos::new(-1, 4);
        assert_eq!("(-1, 4)", format!("{}", pos));
    }

    #[test]
    fn format_negative_y_component_correct_signs() {
        let pos = Pos::new(1, -4);
        assert_eq!("(1, -4)", format!("{}", pos));
    }

    #[test]
    fn format_negative_components_signs() {
        let pos = Pos::new(-1, -4);
        assert_eq!("(-1, -4)", format!("{}", pos));
    }
}