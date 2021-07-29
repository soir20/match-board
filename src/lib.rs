pub mod piece;
pub mod board;
pub mod matching;

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
