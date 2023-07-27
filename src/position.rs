use std::fmt::{Display, Formatter};
use std::ops::{Add, Sub};

#[derive(Debug, PartialEq, Eq)]
pub enum ColError {
    OutOfBounds(usize)
}

/// Represents a column in a two-dimensional plane.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Col<const BOARD_WIDTH: usize> {

    /// The index of the column
    pub x: usize

}

impl<const W: usize> Col<W> {
    /// Creates a new position with a horizontal and vertical component.
    ///
    /// # Arguments
    ///
    /// * `x` - the horizontal component of the position
    ///
    /// # Panics
    ///
    /// Panics if the given coordinates are outside the board.
    pub fn new(x: usize) -> Col<W> {
        if x >= W {
            panic!("Tried to create column outside board: ({x})");
        }

        Col { x }
    }

    /// Attempts to create a new column, returning an error if the column is
    /// outside the board's bounds.
    ///
    /// # Arguments
    ///
    /// * `x` - the index of the column
    pub fn try_new(x: usize) -> Result<Col<W>, ColError> {
        if x >= W {
            return Err(ColError::OutOfBounds(x));
        }

        Ok(Col { x })
    }

}

#[derive(Debug, PartialEq, Eq)]
pub enum PosError {
    OutOfBounds(usize, usize),
    Overflow
}

/// A position that represents a location in a two-dimensional plane.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pos<const BOARD_WIDTH: usize, const BOARD_HEIGHT: usize> {

    /// The horizontal component of the position
    x: usize,

    /// The vertical component of the position
    y: usize

}

impl<const W: usize, const H: usize> Pos<W, H> {

    /// Creates a new position with a horizontal and vertical component.
    ///
    /// # Arguments
    ///
    /// * `x` - the horizontal component of the position
    /// * `y` - the vertical component of the position
    ///
    /// # Panics
    ///
    /// Panics if the given coordinates are outside the board.
    pub fn new(x: usize, y: usize) -> Pos<W, H> {
        if x >= W || y >= H {
            panic!("Tried to create position outside board: ({x}, {y})");
        }

        Pos { x, y }
    }

    /// Attempts to create a new position with a horizontal and vertical component, returning 
    /// an error if the position is outside the board's bounds.
    ///
    /// # Arguments
    ///
    /// * `x` - the horizontal component of the position
    /// * `y` - the vertical component of the position
    pub fn try_new(x: usize, y: usize) -> Result<Pos<W, H>, PosError> {
        if x >= W {
            return Err(PosError::OutOfBounds(x, y));
        }

        if y >= H {
            return Err(PosError::OutOfBounds(x, y));
        }

        Ok(Pos { x, y })
    }

    /// Returns the horizontal component of the position.
    pub fn x(&self) -> usize {
        self.x
    }

    /// Returns the vertical component of the position.
    pub fn y(&self) -> usize {
        self.y
    }

}

impl<const MX: usize, const MY: usize> Add for Pos<MX, MY> {
    type Output = Result<Pos<MX, MY>, PosError>;

    fn add(self, rhs: Self) -> Self::Output {
        let new_x = self.x.checked_add(rhs.x);
        if new_x.is_none() {
            return Err(PosError::Overflow);
        }

        let new_y = self.y.checked_add(rhs.y);
        if new_y.is_none() {
            return Err(PosError::Overflow);
        }

        Pos::try_new(new_x.unwrap(), new_y.unwrap())
    }
}

impl<const MX: usize, const MY: usize> Sub for Pos<MX, MY> {
    type Output = Result<Pos<MX, MY>, PosError>;

    fn sub(self, rhs: Self) -> Self::Output {
        let new_x = self.x.checked_sub(rhs.x);
        if new_x.is_none() {
            return Err(PosError::Overflow);
        }

        let new_y = self.y.checked_sub(rhs.y);
        if new_y.is_none() {
            return Err(PosError::Overflow);
        }

        Pos::try_new(new_x.unwrap(), new_y.unwrap())
    }
}

impl<const MX: usize, const MY: usize> Display for Pos<MX, MY> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Col, ColError, PosError};
    use crate::position::Pos;

    #[test]
    #[should_panic]
    fn col_new_x_out_of_bounds() {
        Col::<15>::new(15);
    }

    #[test]
    #[should_panic]
    fn col_new_large_x_out_of_bounds() {
        Col::<15>::new(usize::MAX);
    }

    #[test]
    fn col_new_positive_component_allowed() {
        let col: Col<15> = Col::new(1);
        assert_eq!(1, col.x);
    }

    #[test]
    fn col_try_new_x_out_of_bounds() {
        assert_eq!(Col::<15>::try_new(15), Err(ColError::OutOfBounds(15)));
    }

    #[test]
    fn col_try_new_large_x_out_of_bounds() {
        assert_eq!(Col::<15>::try_new(usize::MAX), Err(ColError::OutOfBounds(usize::MAX)));
    }

    #[test]
    fn col_try_new_positive_components_allowed() {
        let col: Col<15> = Col::try_new(1).unwrap();
        assert_eq!(1, col.x);
    }

    #[test]
    #[should_panic]
    fn pos_new_x_out_of_bounds() {
        Pos::<15, 16>::new(15, 4);
    }

    #[test]
    #[should_panic]
    fn pos_new_y_out_of_bounds() {
        Pos::<15, 16>::new(1, 16);
    }

    #[test]
    #[should_panic]
    fn pos_new_large_x_out_of_bounds() {
        Pos::<15, 16>::new(usize::MAX, 4);
    }

    #[test]
    #[should_panic]
    fn pos_new_large_y_out_of_bounds() {
        Pos::<15, 16>::new(1, usize::MAX);
    }

    #[test]
    fn pos_new_positive_components_allowed() {
        let pos: Pos<15, 16> = Pos::new(1, 4);
        assert_eq!(1, pos.x());
        assert_eq!(4, pos.y());
    }

    #[test]
    fn pos_try_new_x_out_of_bounds() {
        assert_eq!(Pos::<15, 16>::try_new(15, 4), Err(PosError::OutOfBounds(15, 4)));
    }

    #[test]
    fn pos_try_new_y_out_of_bounds() {
        assert_eq!(Pos::<15, 16>::try_new(1, 16), Err(PosError::OutOfBounds(1, 16)));
    }

    #[test]
    fn pos_try_new_large_x_out_of_bounds() {
        assert_eq!(Pos::<15, 16>::try_new(usize::MAX, 4), Err(PosError::OutOfBounds(usize::MAX, 4)));
    }

    #[test]
    fn pos_try_new_large_y_out_of_bounds() {
        assert_eq!(Pos::<15, 16>::try_new(1, usize::MAX), Err(PosError::OutOfBounds(1, usize::MAX)));
    }

    #[test]
    fn pos_try_new_positive_components_allowed() {
        let pos: Pos<15, 16> = Pos::try_new(1, 4).unwrap();
        assert_eq!(1, pos.x());
        assert_eq!(4, pos.y());
    }

    #[test]
    fn add_positive_components_summed() {
        let pos1: Pos<15, 16> = Pos::new(1, 4);
        let pos2: Pos<15, 16> = Pos::new(13, 5);
        let sum = (pos1 + pos2).unwrap();
        assert_eq!(14, sum.x());
        assert_eq!(9, sum.y());
    }

    #[test]
    fn add_positive_components_out_of_bounds() {
        let pos1: Pos<15, 16> = Pos::new(1, 4);
        let pos2: Pos<15, 16> = Pos::new(14, 5);
        assert_eq!((pos1 + pos2), Err(PosError::OutOfBounds(15, 9)));
    }

    #[test]
    fn add_positive_components_overflow() {
        let pos1: Pos<{ usize::MAX }, 16> = Pos::new(usize::MAX - 5, 4);
        let pos2: Pos<{ usize::MAX }, 16> = Pos::new(6, 5);
        assert_eq!((pos1 + pos2), Err(PosError::Overflow));
    }

    #[test]
    fn sub_positive_components_subtracted() {
        let pos1: Pos<15, 16> = Pos::new(1, 4);
        let pos2: Pos<15, 16> = Pos::new(14, 5);
        let diff = (pos2 - pos1).unwrap();
        assert_eq!(13, diff.x());
        assert_eq!(1, diff.y());
    }

    #[test]
    fn sub_positive_components_overflow() {
        let pos1: Pos<{ usize::MAX }, 16> = Pos::new(5, 4);
        let pos2: Pos<{ usize::MAX }, 16> = Pos::new(6, 5);
        assert_eq!((pos1 - pos2), Err(PosError::Overflow));
    }

    #[test]
    fn equals_same_pos_equal() {
        let pos1: Pos<15, 16> = Pos::new(1, 4);
        let pos2: Pos<15, 16> = Pos::new(1, 4);
        assert_eq!(pos1, pos2);
    }

    #[test]
    fn equals_components_diff_not_equal() {
        let pos1: Pos<15, 16> = Pos::new(1, 4);
        let pos2: Pos<15, 16> = Pos::new(0, 15);
        assert_ne!(pos1, pos2);
    }

    #[test]
    fn equals_components_reversed_not_equal() {
        let pos1: Pos<15, 16> = Pos::new(1, 4);
        let pos2: Pos<15, 16> = Pos::new(4, 1);
        assert_ne!(pos1, pos2);
    }

    #[test]
    fn format_positive_components_no_signs() {
        let pos: Pos<15, 16> = Pos::new(1, 4);
        assert_eq!("(1, 4)", format!("{}", pos));
    }
}
