mod piece;

use enumset::EnumSet;

struct Game {
    score: u64
}

struct Board {
    h_size: u32,
    v_size: u32,
    pieces: vec![0; length]
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
