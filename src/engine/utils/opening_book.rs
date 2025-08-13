use std::{collections::HashMap, sync::OnceLock};

use rand::{SeedableRng, rngs::StdRng};

use crate::engine::game::board::Board;

static OPENING_BOOK: OnceLock<OpeningBook> = OnceLock::new();

fn init_book() -> OpeningBook {
    OpeningBook::new(BOOK)
}

/// Retrieve the book instance containing "assets/book.txt"
pub fn get_book() -> &'static OpeningBook {
    OPENING_BOOK.get_or_init(init_book)
}

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
    pub fn new(file_contents: &str) -> Self {
        let mut book = OpeningBook::default();
        let entries: Vec<&str> = file_contents.trim_matches(&[',', '\n']).split("pos").collect();
        book.moves_by_position.reserve(entries.len());
        
        for entry in entries {
            let entry_data: Vec<&str> = entry.trim_matches('\n').split('\n').collect();
            let position_fen = entry_data[0].trim();
            let all_move_data = entry_data[1..].to_vec();

            let mut book_moves = Vec::with_capacity(all_move_data.len());

            for move_idx in 0..all_move_data.len() {
                let move_data: Vec<&str> = all_move_data[move_idx].split_whitespace().collect();
                book_moves[move_idx] = BookMove::new(move_data[0].to_string(), move_data[1].parse().unwrap_or(0));
            }

            book.moves_by_position.insert(position_fen.to_string(), book_moves);
        }
        book
    }

    pub fn has_book_move(&self, position_fen: String) -> bool {
        self.moves_by_position.contains_key(&Self::remove_move_counters_from_fen(position_fen))
    }

    pub fn try_get_book_move(board: &Board, weight_pow: f32) -> Option<String> {
        todo!()
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
