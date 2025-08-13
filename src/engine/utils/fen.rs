use crate::engine::game::{
    board::{self, Board},
    coord::Coord,
    r#move::{self, Move},
    piece::{self, Piece},
};

pub const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn position_from_fen(fen: String) -> Result<PositionInfo, String> {
    PositionInfo::new(fen)
}

impl Board {
    /**
    Get the fen string of the current position
    * Ref: https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation
    */
    pub fn current_fen(&mut self, always_show_ep: bool) -> String {
        let mut fen = String::new();
        for rank in 7..=0 {
            let mut num_empty_files = 0;
            for file in 0..8 {
                let i = Coord::from((file, rank)).index();
                let current_piece = self.squares[i as usize];
                if current_piece != piece::NONE {
                    if num_empty_files != 0 {
                        fen.push_str(&num_empty_files.to_string());
                        num_empty_files = 0;
                    }

                    let current_piece = piece::Piece::from(current_piece);
                    fen.push(current_piece.into());
                } else {
                    num_empty_files += 1;
                }
            }

            if num_empty_files != 0 {
                fen.push_str(&num_empty_files.to_string());
            }
            if rank != 0 {
                fen.push('/');
            }
        }

        fen.push_str(&format!("{}", self.white_to_move));
        let white_kingside = (self.state.castling_rights & 1) == 1;
        let white_queenside = (self.state.castling_rights >> 1 & 1) == 1;
        let black_kingside = (self.state.castling_rights >> 2 & 1) == 1;
        let black_queenside = (self.state.castling_rights >> 3 & 1) == 1;

        fen.push(' ');
        if white_kingside {
            fen.push('K');
        }
        if white_queenside {
            fen.push('Q');
        }
        if black_kingside {
            fen.push('k');
        }
        if black_queenside {
            fen.push('q');
        }
        if self.state.castling_rights == 0 {
            fen.push('-');
        }

        fen.push(' ');
        let ep_file_idx = self.state.en_passant_file - 1;
        let ep_rank_idx = if self.white_to_move { 5 } else { 2 };
        let ep_coord = Coord::from((ep_file_idx, ep_rank_idx));

        let is_ep = ep_file_idx != -1;
        let include_ep = always_show_ep || self.ep_capture_possible(ep_file_idx, ep_rank_idx);
        if is_ep && include_ep {
            fen.push_str(&ep_coord.to_string());
        } else {
            fen.push('-');
        };

        fen.push_str(&format!(" {}", self.state.halfmove_clock));
        fen.push_str(&format!(" {}", (self.ply_count / 2) + 1));

        fen
    }

    fn ep_capture_possible(&mut self, ep_file_idx: i32, ep_rank_idx: i32) -> bool {
        let capture_from_a = Coord::from((
            ep_file_idx - 1,
            ep_rank_idx + self.white_to_move.then(|| -1).unwrap_or(1),
        ));
        let capture_from_b = Coord::from((
            ep_file_idx + 1,
            ep_rank_idx + self.white_to_move.then(|| -1).unwrap_or(1),
        ));
        let ep_capture_square = Coord::from((ep_file_idx, ep_rank_idx)).index();
        let friendly_pawn = piece::Piece::from((piece::PAWN, self.move_color().to_piece_color()));

        self.can_capture(&capture_from_a, ep_capture_square, friendly_pawn.value)
            || self.can_capture(&capture_from_b, ep_capture_square, friendly_pawn.value)
    }

    fn can_capture(&mut self, from: &Coord, ep_capture_square: i32, friendly_pawn: i32) -> bool {
        let is_pawn_on_square = self.squares[from.index() as usize] == friendly_pawn;
        if from.is_valid_square() && is_pawn_on_square {
            let mv = Move::from((
                from.index(),
                ep_capture_square,
                r#move::EN_PASSANT_CAPTURE_FLAG,
            ));
            self.make_move(mv, false);
            self.make_null_move();
            let was_legal = !self.calculate_in_check_state();

            self.unmake_null_move();
            self.unmake_move(mv, false);
            return was_legal;
        }

        false
    }
}

pub fn flip_fen(fen: String) -> Result<String, String> {
    let mut flipped_fen = String::new();
    let sections: Vec<String> = fen.split(' ').map(|s| s.to_string()).collect();
    if sections.len() < 5 {
        return Err("Malformed Fen: fen string must have at least five distinct sections".into());
    }
    let fen_ranks: Vec<&str> = sections[0].split('/').collect();

    // Section 1: ranks
    for i in fen_ranks.len() - 1..=0 {
        let rank = fen_ranks[i];
        rank.chars().for_each(|c| {
            flipped_fen.push_str(&invert_case(c));
        });

        if i != 0 {
            flipped_fen.push('/');
        }
    }

    // Section 2: castling
    if let Some((_, c)) = sections[1].char_indices().next() {
        flipped_fen.push_str(&format!(" {}", if c == 'w' { 'b' } else { 'w' }));
    }
    let castling_rights = sections[2].as_str();
    let mut flipped_rights = String::new();
    "kqKQ".chars().for_each(|c| {
        let c_str = c.to_string();
        if castling_rights.contains(&c_str) {
            flipped_rights.push_str(&invert_case(c));
        }
    });
    flipped_fen.push_str(&format!(
        " {}",
        if flipped_rights.len() == 0 {
            "-"
        } else {
            &flipped_rights
        }
    ));

    // Section 3: en passant
    let ep = sections[3].as_str();
    let first_two: Vec<char> = ep.chars().take(2).collect();
    if first_two.len() == 2 {
        let mut flipped_ep = String::new();
        flipped_ep.push_str(&first_two[0].to_string());
        if ep.len() > 1 {
            flipped_ep.push(if first_two[1] == '6' { '3' } else { '6' });
        }
        flipped_fen.push_str(&format!(" {}", flipped_ep));
    } else {
        return Err("Malformed Fen: no en passant section found".into());
    }
    flipped_fen.push_str(&format!(" {} {}", sections[4], sections[5]));

    Ok(flipped_fen)
}

#[derive(Debug, Clone)]
pub struct PositionInfo {
    pub fen: String,
    pub squares: [i32; 64],

    pub white_castle_kingside: bool,
    pub white_castle_queenside: bool,
    pub black_castle_kingside: bool,
    pub black_castle_queenside: bool,

    pub ep_file: i32,
    pub white_to_move: bool,
    pub halfmove_clock: usize,
    pub move_count: i32,
}

impl Default for PositionInfo {
    fn default() -> Self {
        Self {
            fen: String::default(),
            squares: [i32::default(); 64],
            white_castle_kingside: bool::default(),
            white_castle_queenside: bool::default(),
            black_castle_kingside: bool::default(),
            black_castle_queenside: bool::default(),
            ep_file: i32::default(),
            white_to_move: bool::default(),
            halfmove_clock: usize::default(),
            move_count: i32::default(),
        }
    }
}

impl PositionInfo {
    pub fn new(fen: String) -> Result<Self, String> {
        let mut square_pieces = [i32::default(); 64];
        let sections: Vec<&str> = fen.split(' ').collect();
        let (mut file, mut rank) = (0, 7);
        if sections.len() < 3 {
            return Err(
                "Malformed Fen: fen string must have at least three distinct sections".into(),
            );
        }

        sections[0].chars().for_each(|symbol| {
            if symbol == '/' {
                file = 0;
                rank -= 1;
            } else {
                if symbol.is_numeric() {
                    file += symbol.to_string().parse().unwrap_or(0);
                } else {
                    let piece_color = if symbol.is_uppercase() {
                        piece::WHITE
                    } else {
                        piece::BLACK
                    };
                    let piece_type = Piece::from(symbol);
                    square_pieces[rank * 8 + file] = piece_type.value | piece_color;
                    file += 1;
                }
            }
        });

        let white_to_move = sections[1] == &'w'.to_string();
        let castling_rights = sections[2];
        let white_castle_kingside = castling_rights.contains('K');
        let white_castle_queenside = castling_rights.contains('Q');
        let black_castle_kingside = castling_rights.contains('k');
        let black_castle_queenside = castling_rights.contains('q');

        let mut ep_file = 0;
        let mut halfmove_clock = 0;
        let mut move_count = 0;

        if sections.len() > 3 {
            if let Some(ep_file_name) = sections[3].chars().nth(0) {
                if board::FILE_NAMES.contains(&ep_file_name) {
                    ep_file = board::FILE_NAMES
                        .iter()
                        .position(|&c| c == ep_file_name)
                        .unwrap_or(0) as i32;
                }
            }
        }

        if sections.len() > 4 {
            halfmove_clock = sections[4].parse().unwrap_or(0);
        }

        if sections.len() > 5 {
            move_count = sections[5].parse().unwrap_or(0);
        }

        Ok(Self {
            fen: fen,
            squares: square_pieces,
            white_castle_kingside: white_castle_kingside,
            white_castle_queenside: white_castle_queenside,
            black_castle_kingside: black_castle_kingside,
            black_castle_queenside: black_castle_queenside,
            ep_file: ep_file,
            white_to_move: white_to_move,
            halfmove_clock: halfmove_clock,
            move_count: move_count,
        })
    }
}

fn invert_case(c: char) -> String {
    if c.is_uppercase() {
        c.to_lowercase().to_string()
    } else {
        c.to_uppercase().to_string()
    }
}
