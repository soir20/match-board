use crate::position::Pos;
use std::ops::{BitAnd, BitOr, BitXor, Not};

/// The size of a board as
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum BoardSize {
    FourByThirtyTwo,
    FiveByTwentyFive,
    SixByTwentyOne,
    SevenByEighteen,
    EightBySixteen,
    NineByFourteen,
    TenByTwelve,
    ElevenByEleven
}

impl BoardSize {
    pub fn width(&self) -> u8 {
        match *self {
            BoardSize::FourByThirtyTwo => 4,
            BoardSize::FiveByTwentyFive => 5,
            BoardSize::SixByTwentyOne => 6,
            BoardSize::SevenByEighteen => 7,
            BoardSize::EightBySixteen => 8,
            BoardSize::NineByFourteen => 9,
            BoardSize::TenByTwelve => 10,
            BoardSize::ElevenByEleven => 11
        }
    }

    pub fn height(&self) -> u8 {
        match *self {
            BoardSize::FourByThirtyTwo => 32,
            BoardSize::FiveByTwentyFive => 25,
            BoardSize::SixByTwentyOne => 21,
            BoardSize::SevenByEighteen => 18,
            BoardSize::EightBySixteen => 16,
            BoardSize::NineByFourteen => 14,
            BoardSize::TenByTwelve => 12,
            BoardSize::ElevenByEleven => 11
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub(crate) struct BitBoard {
    board: u128,
    width: u8,
    height: u8
}

impl BitBoard {

    pub fn new(size: BoardSize) -> BitBoard {
        BitBoard {
            board: 0,
            width: size.width(),
            height: size.height()
        }
    }

    pub fn is_set(&self, pos: Pos) -> bool {
        (self.board >> self.bit_pos(pos)) & 1 == 1
    }

    pub fn set(&self, pos: Pos) -> BitBoard {
        self.change_board(self.board | (1 << self.bit_pos(pos)))
    }

    pub fn unset(&self, pos: Pos) -> BitBoard {
        self.change_board(self.board & !(1 << self.bit_pos(pos)))
    }

    pub fn swap(&self, first: Pos, second: Pos) -> BitBoard {
        let bit1: u128 = self.is_set(first).into();
        let bit2: u128 = self.is_set(second).into();

        let xor_single = bit1 ^ bit2;
        let xor_in_pos = (xor_single << self.bit_pos(first)) | (xor_single << self.bit_pos(second));

        self.change_board(self.board ^ xor_in_pos)
    }

    fn change_board(&self, board: u128) -> BitBoard {
        BitBoard {
            board,
            width: self.width,
            height: self.height
        }
    }

    fn bit_pos(&self, pos: Pos) -> u8 {
        pos.x() * self.width + pos.y()
    }
}

impl BitAnd for BitBoard {
    type Output = BitBoard;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.change_board(self.board & rhs.board)
    }
}

impl BitOr for BitBoard {
    type Output = BitBoard;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.change_board(self.board | rhs.board)
    }
}

impl BitXor for BitBoard {
    type Output = BitBoard;

    fn bitxor(self, rhs: Self) -> Self::Output {
        self.change_board(self.board ^ rhs.board)
    }
}

impl Not for BitBoard {
    type Output = BitBoard;

    fn not(self) -> Self::Output {
        self.change_board(!self.board)
    }
}