use std::{collections::HashMap};

use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::engine::game::board::Board;

pub const BOOK: &str = include_str!("../../../assets/book.txt");

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
        let entries: Vec<&str> = file_contents
            .trim_matches(&[',', '\n'])
            .split("pos")
            .collect();
        book.moves_by_position.reserve(entries.len());

        for entry in entries {
            let entry_data: Vec<&str> = entry.trim_matches('\n').split('\n').collect();
            let position_fen = entry_data[0].trim();
            let all_move_data = entry_data[1..].to_vec();

            let mut book_moves = Vec::with_capacity(all_move_data.len());

            for move_idx in 0..all_move_data.len() {
                let move_data: Vec<&str> = all_move_data[move_idx].split_whitespace().collect();
                book_moves.push(BookMove::new(move_data[0].to_string(), move_data[1].parse().unwrap_or(0)));
            }

            book.moves_by_position
                .insert(position_fen.to_string(), book_moves);
        }
        book
    }

    pub fn has_book_move(&self, position_fen: String) -> bool {
        self.moves_by_position
            .contains_key(&Self::remove_move_counters_from_fen(position_fen))
    }

    /**
    Try to get a book move from the current position
    * Weight is clamped between 0 and 1 inclusive
    * 0 means all moves are picked with equal probability, 1 means moves are weighted by num times played.
    */
    pub fn try_get_book_move(&mut self, board: &mut Board, weight_pow: f32) -> Option<String> {
        let position_fen = board.current_fen(false);
        let weight_pow = weight_pow.clamp(0.0, 1.0);

        let weighted_play_count = |play_count: i32| (play_count as f32).powf(weight_pow) as i32;

        if let Some(moves) = self.moves_by_position.get(&position_fen) {
            let mut total_play_count = 0;
            moves
                .iter()
                .for_each(|mv| total_play_count += weighted_play_count(mv.num_times_played));

            let mut weights = Vec::with_capacity(moves.len());
            let mut weight_sum = 0.0;
            for i in 0..moves.len() {
                let weight =
                    weighted_play_count(moves[i].num_times_played) as f32 / total_play_count as f32;
                weight_sum += weight;
                weights[i] = weight;
            }

            let mut prob_cum = Vec::with_capacity(moves.len());
            for i in 0..weights.len() {
                let prob = weights[i] / weight_sum;
                prob_cum[i] = prob_cum[0.max(i - 1)] + prob;
            }

            let random = self.rng.random::<f32>();
            for i in 0..moves.len() {
                if random <= prob_cum[i] {
                    return Some(moves[i].move_string.clone());
                }
            }

            None
        } else {
            None
        }
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
