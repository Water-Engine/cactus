use crate::engine::{
    eval::precomputed_evals::{self, PieceSquareTable},
    game::{
        board::{Board, Color},
        coord::Coord,
        piece::{self, PieceList},
    },
    generate::{bitboard, move_generator},
};

pub const PAWN_VALUE: i32 = 100;
pub const KNIGHT_VALUE: i32 = 300;
pub const BISHOP_VALUE: i32 = 320;
pub const ROOK_VALUE: i32 = 500;
pub const QUEEN_VALUE: i32 = 900;

const PASSED_PAWN_BONUSES: [i32; 7] = [0, 120, 80, 50, 30, 15, 15];
const ISOLATED_PAWN_PENALTY_BY_COUNT: [i32; 9] = [0, -10, -25, -50, -75, -75, -75, -75, -75];
const KING_PAWN_SHIELD_SCORES: [i32; 6] = [4, 7, 4, 3, 6, 3];

const QUEEN_ENDGAME_WEIGHT: i32 = 45;
const ROOK_ENDGAME_WEIGHT: i32 = 20;
const BISHOP_ENDGAME_WEIGHT: i32 = 10;
const KNIGHT_ENDGAME_WEIGHT: i32 = 10;

const ENDGAME_START_WEIGHT: i32 = 2 * ROOK_ENDGAME_WEIGHT
    + 2 * BISHOP_ENDGAME_WEIGHT
    + 2 * KNIGHT_ENDGAME_WEIGHT
    + QUEEN_ENDGAME_WEIGHT;
const ENDGAME_MATERIAL_START: f32 = (ROOK_VALUE * 2 + BISHOP_VALUE + KNIGHT_VALUE) as f32;
const ENDGAME_MULTIPLIER: f32 = 1.0 / ENDGAME_MATERIAL_START;

struct Evaluation {
    pub white_eval: EvaluationData,
    pub black_eval: EvaluationData,
}

impl Board {
    pub fn evaluate(&self) -> i32 {
        let mut eval = Evaluation {
            white_eval: EvaluationData::default(),
            black_eval: EvaluationData::default(),
        };

        let white_material = eval.get_material_info(&self, Color::White);
        let black_material = eval.get_material_info(&self, Color::Black);

        eval.white_eval.material_score = white_material.material_score;
        eval.black_eval.material_score = black_material.material_score;

        eval.white_eval.piece_square_score =
            eval.evaluate_piece_square_tables(&self, true, black_material.endgame_t);
        eval.black_eval.piece_square_score =
            eval.evaluate_piece_square_tables(&self, false, white_material.endgame_t);

        // moving king to push enemy king at end of winning game is good
        eval.white_eval.mop_up_score =
            eval.mop_up_eval(&self, true, &white_material, &black_material);
        eval.black_eval.mop_up_score =
            eval.mop_up_eval(&self, false, &black_material, &white_material);

        eval.white_eval.pawn_score = eval.evaluate_pawns(&self, Color::White);
        eval.black_eval.pawn_score = eval.evaluate_pawns(&self, Color::Black);

        eval.white_eval.pawn_shield_score = eval.king_pawn_shield(
            &self,
            Color::White,
            &black_material,
            eval.black_eval.piece_square_score as f32,
        );
        eval.black_eval.pawn_shield_score = eval.king_pawn_shield(
            &self,
            Color::Black,
            &white_material,
            eval.white_eval.piece_square_score as f32,
        );

        let perspective = self.white_to_move.then(|| 1).unwrap_or(-1);
        let eval = eval.white_eval.sum() - eval.black_eval.sum();
        eval * perspective
    }
}

impl Evaluation {
    pub fn king_pawn_shield(
        &self,
        board: &Board,
        color: Color,
        enemy_material: &MaterialInfo,
        enemy_piece_square_score: f32,
    ) -> i32 {
        if enemy_material.endgame_t >= 1.0 {
            return 0;
        }

        let mut penalty = 0;

        let is_white = color == Color::White;
        let friendly_pawn = piece::Piece::from((piece::PAWN, is_white));
        let king_square = board.king_squares[color as usize];
        let king_file = Coord::file_of_square(king_square);

        let mut uncastled_king_penalty = 0;

        if king_file <= 2 || king_file >= 5 {
            let white_pawn_shields = &precomputed_evals::get_ped().pawn_shield_squares
                [Color::White as usize][king_square as usize];
            let black_pawn_shields = &precomputed_evals::get_ped().pawn_shield_squares
                [Color::Black as usize][king_square as usize];
            let squares = is_white
                .then(|| white_pawn_shields)
                .unwrap_or(black_pawn_shields);

            for i in 0..(squares.len() / 2) {
                let shield_square_idx = squares[i];
                if board.squares[shield_square_idx as usize] != friendly_pawn.value {
                    if squares.len() > 3
                        && board.squares[squares[i + 3] as usize] == friendly_pawn.value
                    {
                        penalty += KING_PAWN_SHIELD_SCORES[i + 3];
                    } else {
                        penalty += KING_PAWN_SHIELD_SCORES[i];
                    }
                }
            }
            penalty *= penalty;
        } else {
            let enemy_development_score =
                ((enemy_piece_square_score + 10.0) / 130.0).clamp(0.0, 1.0);
            uncastled_king_penalty = 50 * enemy_development_score as i32;
        }

        let mut open_file_against_king_penalty = 0;

        if enemy_material.num_rooks > 1
            || (enemy_material.num_rooks > 0 && enemy_material.num_queens > 0)
        {
            let clamped_king_file = king_file.clamp(1, 6);
            let my_pawns = enemy_material.enemy_pawns;

            for attack_file in clamped_king_file..=(clamped_king_file + 1) {
                let file_mask = bitboard::get_bitmasks().file[attack_file as usize];
                let is_king_file = attack_file == king_file;

                if (enemy_material.pawns & file_mask) == 0 {
                    open_file_against_king_penalty += is_king_file.then(|| 25).unwrap_or(15);
                    if (my_pawns & file_mask) == 0 {
                        open_file_against_king_penalty += is_king_file.then(|| 15).unwrap_or(10);
                    }
                }
            }
        }

        let mut pawn_shield_weight = 1.0 - enemy_material.endgame_t;
        if board.queens[1 - color as usize].count() == 0 {
            pawn_shield_weight *= 0.6;
        }

        (-penalty - uncastled_king_penalty - open_file_against_king_penalty)
            * pawn_shield_weight as i32
    }

    pub fn evaluate_pawns(&self, board: &Board, color: Color) -> i32 {
        let pawns = board.pawns[color as usize];
        let is_white = color == Color::White;
        let opponent_pawns = board.piece_bbs[piece::Piece::from((
            piece::PAWN,
            is_white.then(|| piece::BLACK).unwrap_or(piece::WHITE),
        ))
        .value as usize];
        let friendly_pawns = board.piece_bbs[piece::Piece::from((
            piece::PAWN,
            is_white.then(|| piece::WHITE).unwrap_or(piece::BLACK),
        ))
        .value as usize];
        let masks = is_white
            .then(|| bitboard::get_bitmasks().white_passed_pawn)
            .unwrap_or(bitboard::get_bitmasks().black_passed_pawn);

        let mut bonus = 0;
        let mut num_isolated_pawns = 0;

        for i in 0..pawns.count() {
            let square = pawns[i];
            let passed_mask = masks[square as usize];

            if (opponent_pawns & passed_mask) == 0 {
                let rank = Coord::rank_of_square(square);
                let num_squares_from_promotion = is_white.then(|| 7 - rank).unwrap_or(rank);
                bonus += PASSED_PAWN_BONUSES[num_squares_from_promotion as usize];
            }

            if (friendly_pawns
                & bitboard::get_bitmasks().adj_file[Coord::file_of_square(square) as usize])
                == 0
            {
                num_isolated_pawns += 1;
            }
        }

        bonus + ISOLATED_PAWN_PENALTY_BY_COUNT[num_isolated_pawns as usize]
    }

    fn endgame_phase_weight(material_count_without_pawns: i32) -> f32 {
        1.0 - 1.0_f32.min(material_count_without_pawns as f32 * ENDGAME_MULTIPLIER)
    }

    fn mop_up_eval(
        &self,
        board: &Board,
        is_white: bool,
        my_material: &MaterialInfo,
        enemy_material: &MaterialInfo,
    ) -> i32 {
        if my_material.material_score > enemy_material.material_score + PAWN_VALUE * 2
            && enemy_material.endgame_t > 0.0
        {
            let mut mop_up_score = 0;
            let friendly_idx = is_white.then(|| Color::White).unwrap_or(Color::Black) as usize;
            let opponent_idx = is_white.then(|| Color::Black).unwrap_or(Color::White) as usize;

            let friendly_king_square = board.king_squares[friendly_idx];
            let opponent_king_square = board.king_squares[opponent_idx];

            // Encourage moving king closer to opponent king
            mop_up_score += (14
                - move_generator::get_pmd().orthogonal_distance[friendly_king_square as usize]
                    [opponent_king_square as usize])
                * 4;

            // Encourage pushing opponent king to edge of board
            mop_up_score += move_generator::get_pmd().center_manhattan_distance
                [opponent_king_square as usize]
                * 10;
            (mop_up_score as f32 * enemy_material.endgame_t) as i32
        } else {
            0
        }
    }

    fn count_material(&self, board: &Board, color: Color) -> i32 {
        let color_idx = color as usize;
        let mut material = 0;

        material += board.pawns[color_idx].count() as i32 * PAWN_VALUE;
        material += board.knights[color_idx].count() as i32 * KNIGHT_VALUE;
        material += board.bishops[color_idx].count() as i32 * BISHOP_VALUE;
        material += board.rooks[color_idx].count() as i32 * ROOK_VALUE;
        material += board.queens[color_idx].count() as i32 * QUEEN_VALUE;

        material
    }

    fn evaluate_piece_square_tables(&self, board: &Board, is_white: bool, endgame_t: f32) -> i32 {
        let mut value = 0;
        let color_idx = is_white.then(|| Color::White).unwrap_or(Color::Black) as usize;

        // Major piece states
        value += Self::evaluate_piece_square_table(
            precomputed_evals::ROOKS,
            board.rooks[color_idx],
            is_white,
        );
        value += Self::evaluate_piece_square_table(
            precomputed_evals::KNIGHTS,
            board.knights[color_idx],
            is_white,
        );
        value += Self::evaluate_piece_square_table(
            precomputed_evals::BISHOPS,
            board.bishops[color_idx],
            is_white,
        );
        value += Self::evaluate_piece_square_table(
            precomputed_evals::QUEENS,
            board.queens[color_idx],
            is_white,
        );

        // Pawn states
        let pawn_early_phase = Self::evaluate_piece_square_table(
            precomputed_evals::PAWNS,
            board.pawns[color_idx],
            is_white,
        );
        value += (pawn_early_phase as f32 * (1.0 - endgame_t)) as i32;

        let pawn_late_phase = Self::evaluate_piece_square_table(
            precomputed_evals::PAWNS_ENDGAME,
            board.pawns[color_idx],
            is_white,
        );
        value += (pawn_late_phase as f32 * endgame_t) as i32;

        // King states
        let king_early_phase = precomputed_evals::read(
            precomputed_evals::KING_START,
            board.king_squares[color_idx],
            is_white,
        );
        value += (king_early_phase as f32 * (1.0 - endgame_t)) as i32;

        let king_late_phase = precomputed_evals::read(
            precomputed_evals::KING_ENDGAME,
            board.king_squares[color_idx],
            is_white,
        );
        value += (king_late_phase as f32 * endgame_t) as i32;

        value
    }

    fn evaluate_piece_square_table(table: [i32; 64], piece_list: PieceList, is_white: bool) -> i32 {
        let mut value = 0;
        for i in 0..piece_list.count() {
            value += precomputed_evals::read(table, piece_list[i], is_white);
        }
        value
    }

    fn get_material_info(&self, board: &Board, color: Color) -> MaterialInfo {
        let color_idx = color as usize;

        let num_pawns = board.pawns[color_idx].count() as i32;
        let num_knights = board.knights[color_idx].count() as i32;
        let num_bishops = board.bishops[color_idx].count() as i32;
        let num_rooks = board.rooks[color_idx].count() as i32;
        let num_queens = board.queens[color_idx].count() as i32;

        let my_color = color.to_piece_color();
        let enemy_color = (my_color == piece::WHITE)
            .then(|| piece::BLACK)
            .unwrap_or(piece::WHITE);

        let my_pawns = board.piece_bbs[piece::Piece::from((piece::PAWN, my_color)).value as usize];
        let enemy_pawns =
            board.piece_bbs[piece::Piece::from((piece::PAWN, enemy_color)).value as usize];

        MaterialInfo::new(
            num_pawns,
            num_knights,
            num_bishops,
            num_queens,
            num_rooks,
            my_pawns,
            enemy_pawns,
        )
    }
}

#[derive(Debug, Default)]
pub struct EvaluationData {
    pub material_score: i32,
    pub mop_up_score: i32,
    pub piece_square_score: i32,
    pub pawn_score: i32,
    pub pawn_shield_score: i32,
}

impl EvaluationData {
    pub fn sum(&self) -> i32 {
        self.material_score
            + self.mop_up_score
            + self.piece_square_score
            + self.pawn_score
            + self.pawn_shield_score
    }
}

#[derive(Debug, Default)]
pub struct MaterialInfo {
    pub material_score: i32,

    pub num_pawns: i32,
    pub num_majors: i32,
    pub num_minors: i32,
    pub num_bishops: i32,
    pub num_queens: i32,
    pub num_rooks: i32,

    pub pawns: u64,
    pub enemy_pawns: u64,

    pub endgame_t: f32,
}

impl MaterialInfo {
    pub fn new(
        num_pawns: i32,
        num_knights: i32,
        num_bishops: i32,
        num_queens: i32,
        num_rooks: i32,
        my_pawns: u64,
        enemy_pawns: u64,
    ) -> Self {
        let mut material_score = 0;
        material_score += num_pawns * PAWN_VALUE;
        material_score += num_knights * KNIGHT_VALUE;
        material_score += num_bishops * BISHOP_VALUE;
        material_score += num_rooks * ROOK_VALUE;
        material_score += num_queens * QUEEN_VALUE;

        let endgame_weight_sum = num_queens * QUEEN_ENDGAME_WEIGHT
            + num_rooks * ROOK_ENDGAME_WEIGHT
            + num_bishops * BISHOP_ENDGAME_WEIGHT
            + num_knights * KNIGHT_ENDGAME_WEIGHT;
        let endgame_t = 1.0 - 1.0_f32.min(endgame_weight_sum as f32 / ENDGAME_START_WEIGHT as f32);

        Self {
            material_score: material_score,
            num_pawns: num_pawns,
            num_majors: num_rooks + num_queens,
            num_minors: num_bishops + num_knights,
            num_bishops: num_bishops,
            num_queens: num_queens,
            num_rooks: num_rooks,
            pawns: my_pawns,
            enemy_pawns: enemy_pawns,
            endgame_t: endgame_t,
        }
    }
}
