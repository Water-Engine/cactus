use crate::engine::{
    eval::precomputed_evals, game::{board::Board, r#move::{self, Move}, piece}, generate::{bitboard, move_generator::MAX_MOVES}
};

pub const MAX_KILLER_MOVE_PLY: usize = 32;

const MILLION: i32 = 1_000_000;
const HASH_MOVE_SCORE: i32 = 100 * MILLION;
const WINNING_CAPTURE_BIAS: i32 = 8 * MILLION;
const PROMOTE_BIAS: i32 = 6 * MILLION;
const KILLER_BIAS: i32 = 4 * MILLION;
const LOSING_CAPTURE_BIAS: i32 = 2 * MILLION;
const REGULAR_BIAS: i32 = 0;

pub struct MoveOrdering {
    move_scores: [i32; MAX_MOVES],
    killer_moves: [Killers; MAX_KILLER_MOVE_PLY],
    history: [[[i32; 64]; 64]; 2],
}

impl MoveOrdering {
    pub fn new() -> Self {
        Self {
            move_scores: [i32::default(); MAX_MOVES],
            killer_moves: std::array::from_fn(|_| Killers::default()),
            history: [[[i32::default(); 64]; 64]; 2],
        }
    }

    pub fn clear_history(&mut self) {
        self.history = [[[i32::default(); 64]; 64]; 2];
    }

    pub fn clear_killers(&mut self) {
        self.killer_moves = std::array::from_fn(|_| Killers::default());
    }

    pub fn order_moves(
        &mut self,
        board: &Board,
        hash_move: Move,
        moves: &mut Vec<Move>,
        opponent_attacks: u64,
        opponent_pawn_attacks: u64,
        in_q_search: bool,
        ply: i32,
    ) {
        let _opponent_pieces = board.enemy_diag_slider_bb
            | board.enemy_ortho_slider_bb
            | board.piece_bbs[piece::Piece::from((
                piece::KNIGHT,
                board.opponent_color().to_piece_color(),
            ))
            .value as usize];
        let _pawn_attacks = board
            .white_to_move
            .then(|| bitboard::get_bb_utility().white_pawn_attacks)
            .unwrap_or(bitboard::get_bb_utility().black_pawn_attacks);

        for i in 0..moves.len() {
            let mv = moves[i];

            if mv == hash_move {
                self.move_scores[i] = HASH_MOVE_SCORE;
                continue;
            }

            let mut score = 0;
            let start_square = mv.start_square();
            let target_square = mv.target_square();

            let move_square = board.squares[start_square as usize];
            let move_piece = piece::Piece::from(move_square);
            let move_piece_type = move_piece.get_type();

            let capture_square = board.squares[target_square as usize];
            let capture_piece = piece::Piece::from(capture_square);
            let capture_piece_type = capture_piece.get_type();

            let is_capture = capture_piece_type != piece::NONE;
            let flag = mv.move_flag();
            let piece_value = piece::get_piece_value(move_piece_type);

            if is_capture {
                // Order moves to try capturing the most valuable opponent piece with least valuable of own pieces first
                let capture_material_delta = piece::get_piece_value(capture_piece_type) - piece_value;
                let opponent_can_recapture = bitboard::BitBoard::contains_square(opponent_pawn_attacks | opponent_attacks, target_square);

                if opponent_can_recapture {
                    score += (capture_material_delta >= 0).then(|| WINNING_CAPTURE_BIAS).unwrap_or(LOSING_CAPTURE_BIAS) + capture_material_delta;
                } else {
                    score += WINNING_CAPTURE_BIAS + capture_material_delta;
                }
            }

            if move_piece_type == piece::PAWN {
                if flag == r#move::PROMOTE_TO_QUEEN_FLAG && !is_capture {
                    score += PROMOTE_BIAS;
                }
            } else {
                let to_score = precomputed_evals::get_pst().read(move_piece.value, target_square);
                let from_score = precomputed_evals::get_pst().read(move_piece.value, start_square);
                score += to_score - from_score;

                if bitboard::BitBoard::contains_square(opponent_pawn_attacks, target_square) {
                    score -= 50;
                } else if bitboard::BitBoard::contains_square(opponent_attacks, target_square) {
                    score -= 25;
                }
            }

            if !is_capture {
                let is_killer = !in_q_search && ply < MAX_KILLER_MOVE_PLY as i32 && self.killer_moves[ply as usize].matches(mv);
                score += is_killer.then(|| KILLER_BIAS).unwrap_or(REGULAR_BIAS);
                score += self.history[board.move_color() as usize][start_square as usize][target_square as usize];
            }

            self.move_scores[i] = score;
        }

        quicksort(moves, &mut self.move_scores, 0, moves.len() - 1);
    }
}

fn quicksort(moves: &mut Vec<Move>, scores: &mut [i32; MAX_MOVES], low: usize, high: usize) {
    if low < high {
        let pivot_idx = partition(moves, scores, low, high);
        quicksort(moves, scores, low, pivot_idx - 1);
        quicksort(moves, scores, pivot_idx + 1, high);
    }
}

fn partition(moves: &mut Vec<Move>, scores: &mut [i32; MAX_MOVES], low: usize, high: usize) -> usize {
    let pivot_score = scores[high];
    let mut i = low - 1;

    for j in low..high {
        if scores[j] > pivot_score {
            i += 1;
            moves.swap(i, j);
            scores.swap(i, j as usize);
        }
    }

    moves.swap(i + 1, high);
    scores.swap(i + 1, high);

    return i + 1;
}

#[derive(Debug, Default)]
pub struct Killers {
    pub move_a: Move,
    pub move_b: Move,
}

impl Killers {
    pub fn new(move_a: Move, move_b: Move) -> Self {
        Self {
            move_a: move_a,
            move_b: move_b,
        }
    }

    pub fn add(&mut self, mv: Move) {
        if mv.value != self.move_a.value {
            self.move_b = self.move_a;
            self.move_a = mv;
        }
    }

    pub fn matches(&self, mv: Move) -> bool {
        mv.value == self.move_a.value || mv.value == self.move_b.value
    }
}
