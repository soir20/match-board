pub mod piece;
pub mod board;
pub mod position;
pub mod matching;
pub mod board_state;
mod bitboard;

struct Game {
    score: u64
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
