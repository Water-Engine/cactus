use std::collections::HashMap;

use rand::{SeedableRng, rngs::StdRng};

const BOOK: &str = include_str!("../../../assets/book.txt");

pub struct OpeningBook {
    moves_by_position: HashMap<String, Vec<BookMove>>,
    rng: StdRng,
}

impl Default for OpeningBook {
    fn default() -> Self {
        Self {
            moves_by_position: HashMap::new(),
            rng: StdRng::from_os_rng(),
        }
    }
}

impl OpeningBook {
    pub fn new(file: &str) -> Self {
        let mut book = OpeningBook::default();
        book
    }

    pub fn has_book_move(&self, position_fen: String) -> bool {
        self.moves_by_position.contains_key(&Self::remove_move_counters_from_fen(position_fen))
    }

    fn remove_move_counters_from_fen(fen: String) -> String {
        if let Some(last_space) = fen.rfind(' ') {
            let fen_a = &fen[..last_space];
            if let Some(second_last_space) = fen_a.rfind(' ') {
                return fen_a[..second_last_space].to_string();
            }
        }
        fen
    }
}

pub struct BookMove {
    pub move_string: String,
    pub num_times_played: i32,
}

impl BookMove {
    pub fn new(move_string: String, num_times_played: i32) -> Self {
        Self {
            move_string: move_string,
            num_times_played: num_times_played,
        }
    }
}
