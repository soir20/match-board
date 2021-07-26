mod piece;

use std::collections::HashMap;
use enumset::EnumSet;
use crate::piece::{Piece, Pos};

struct Game {
    score: u64
}

struct Board {
    h_size: u32,
    v_size: u32,
    pieces: HashMap<Pos, Piece>
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
