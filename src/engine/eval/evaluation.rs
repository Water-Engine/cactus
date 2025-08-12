use crate::engine::{
    eval::precomputed_evals::{self, PieceSquareTable},
    game::{
        board::{Board, Color},
        piece::{self, PieceList},
    },
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

pub struct Evaluation {
    board: Board,
    pub white_eval: EvaluationData,
    pub black_eval: EvaluationData,
}

impl Evaluation {
    pub fn evaluate(board: &Board) -> i32 {
        let mut eval = Self {
            board: board.clone(),
            white_eval: EvaluationData::default(),
            black_eval: EvaluationData::default(),
        };

        let white_material = eval.get_material_info(Color::White);
        let black_material = eval.get_material_info(Color::Black);

        eval.white_eval.material_score = white_material.material_score;
        eval.black_eval.material_score = black_material.material_score;

        eval.white_eval.piece_square_score =
            eval.evaluate_piece_square_tables(true, black_material.endgame_t);
        eval.black_eval.piece_square_score =
            eval.evaluate_piece_square_tables(false, white_material.endgame_t);

        // moving king to push enemy king at end of winning game is good
        eval.white_eval.mop_up_score = eval.mop_up_eval(true, &white_material, &black_material);
        eval.black_eval.mop_up_score = eval.mop_up_eval(false, &black_material, &white_material);

        eval.white_eval.pawn_score = eval.evaluate_pawns(Color::White);
        eval.black_eval.pawn_score = eval.evaluate_pawns(Color::Black);

        eval.white_eval.pawn_shield_score = eval.king_pawn_shield(
            Color::White,
            &black_material,
            eval.black_eval.piece_square_score as f32,
        );
        eval.black_eval.pawn_shield_score = eval.king_pawn_shield(
            Color::Black,
            &white_material,
            eval.white_eval.piece_square_score as f32,
        );

        let perspective = eval.board.white_to_move.then(|| 1).unwrap_or(-1);
        let eval = eval.white_eval.sum() - eval.black_eval.sum();
        eval * perspective
    }

    pub fn king_pawn_shield(
        &self,
        color: Color,
        enemy_material: &MaterialInfo,
        enemy_piece_square_score: f32,
    ) -> i32 {
        todo!("Not implemented")
    }

    pub fn evaluate_pawns(&self, color: Color) -> i32 {
        todo!("Not implemented")
    }

    fn endgame_phase_weight(material_count_without_pawns: i32) -> f32 {
        1.0 - 1.0_f32.min(material_count_without_pawns as f32 * ENDGAME_MULTIPLIER)
    }

    fn mop_up_eval(
        &self,
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

            let friendly_king_square = self.board.king_squares[friendly_idx];
            let opponent_king_square = self.board.king_squares[opponent_idx];

            todo!("Not implemented")
        } else {
            0
        }
    }

    fn count_material(&self, color: Color) -> i32 {
        let color_idx = color as usize;
        let mut material = 0;

        material += self.board.pawns[color_idx].count() as i32 * PAWN_VALUE;
        material += self.board.knights[color_idx].count() as i32 * KNIGHT_VALUE;
        material += self.board.bishops[color_idx].count() as i32 * BISHOP_VALUE;
        material += self.board.rooks[color_idx].count() as i32 * ROOK_VALUE;
        material += self.board.queens[color_idx].count() as i32 * QUEEN_VALUE;

        material
    }

    fn evaluate_piece_square_tables(&self, is_white: bool, endgame_t: f32) -> i32 {
        let mut value = 0;
        let color_idx = is_white.then(|| Color::White).unwrap_or(Color::Black) as usize;

        // Major piece states
        value += Self::evaluate_piece_square_table(
            precomputed_evals::ROOKS,
            self.board.rooks[color_idx],
            is_white,
        );
        value += Self::evaluate_piece_square_table(
            precomputed_evals::KNIGHTS,
            self.board.knights[color_idx],
            is_white,
        );
        value += Self::evaluate_piece_square_table(
            precomputed_evals::BISHOPS,
            self.board.bishops[color_idx],
            is_white,
        );
        value += Self::evaluate_piece_square_table(
            precomputed_evals::QUEENS,
            self.board.queens[color_idx],
            is_white,
        );

        // Pawn states
        let pawn_early_phase = Self::evaluate_piece_square_table(
            precomputed_evals::PAWNS,
            self.board.pawns[color_idx],
            is_white,
        );
        value += (pawn_early_phase as f32 * (1.0 - endgame_t)) as i32;

        let pawn_late_phase = Self::evaluate_piece_square_table(
            precomputed_evals::PAWNS_ENDGAME,
            self.board.pawns[color_idx],
            is_white,
        );
        value += (pawn_late_phase as f32 * endgame_t) as i32;

        // King states
        let king_early_phase = precomputed_evals::read(
            precomputed_evals::KING_START,
            self.board.king_squares[color_idx],
            is_white,
        );
        value += (king_early_phase as f32 * (1.0 - endgame_t)) as i32;

        let king_late_phase = precomputed_evals::read(
            precomputed_evals::KING_ENDGAME,
            self.board.king_squares[color_idx],
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

    fn get_material_info(&self, color: Color) -> MaterialInfo {
        let color_idx = color as usize;

        let num_pawns = self.board.pawns[color_idx].count() as i32;
        let num_knights = self.board.knights[color_idx].count() as i32;
        let num_bishops = self.board.bishops[color_idx].count() as i32;
        let num_rooks = self.board.rooks[color_idx].count() as i32;
        let num_queens = self.board.queens[color_idx].count() as i32;

        let my_color = color.to_piece_color();
        let enemy_color = (my_color == piece::WHITE)
            .then(|| piece::BLACK)
            .unwrap_or(piece::WHITE);

        let my_pawns =
            self.board.piece_bbs[piece::Piece::from((piece::PAWN, my_color)).value as usize];
        let enemy_pawns =
            self.board.piece_bbs[piece::Piece::from((piece::PAWN, enemy_color)).value as usize];

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
