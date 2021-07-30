use std::ops::{Add, Sub};
use std::fmt::{Display, Formatter};

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub struct Pos {
    x: i32,
    y: i32
}

impl Pos {
    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }
}

impl Add for Pos {
    type Output = Pos;

    fn add(self, rhs: Self) -> Self::Output {
        Pos {x: self.x + rhs.x, y: self.y + rhs.y}
    }
}

impl Sub for Pos {
    type Output = Pos;

    fn sub(self, rhs: Self) -> Self::Output {
        Pos {x: self.x - rhs.x, y: self.y - rhs.y}
    }
}

impl Display for Pos {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "({}, {})", self.x, self.y)
    }
}