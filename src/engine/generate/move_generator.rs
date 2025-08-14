use std::sync::OnceLock;

use crate::engine::{
    game::{
        board::{self, Board, Color},
        coord::Coord,
        r#move::{self, Move},
        piece,
    },
    generate::{
        bitboard::{self, BitBoard},
        magic::{self, Magic},
    },
};

pub const MAX_MOVES: usize = 218;

#[derive(Debug)]
pub enum PromotionMode {
    All,
    Queen,
    QueenAndKnight,
}

impl Default for PromotionMode {
    fn default() -> Self {
        PromotionMode::All
    }
}

#[derive(Debug, Default)]
pub struct MoveGenerator {
    pub promotions_to_generate: PromotionMode,

    white_to_move: bool,
    friendly_color: Color,
    opponent_color: Color,
    friendly_king_square: i32,
    friendly_index: i32,
    enemy_index: i32,

    pub in_check: bool,
    in_double_check: bool,

    /// Contains squares in line from checking piece up to king, or all 1s if not checked
    check_ray_bitboard: u64,

    pin_rays: u64,
    not_pin_rays: u64,
    opponent_attack_map_no_pawns: u64,
    pub opponent_attack_map: u64,
    pub opponent_pawn_attack_map: u64,
    opponent_sliding_attack_map: u64,

    generate_quiet_moves: bool,
    current_move_index: usize,

    enemy_pieces: u64,
    friendly_pieces: u64,
    all_pieces: u64,
    empty_squares: u64,
    empty_or_enemy_squares: u64,

    /// If only captures should be generated, this will have 1s only in positions of enemy pieces
    move_type_mask: u64,
}

impl Board {
    pub fn generate_moves(&mut self, captures_only: bool) -> (Vec<Move>, MoveGenerator) {
        let mut moves = Vec::with_capacity(MAX_MOVES);
        let mg = self.append_moves(&mut moves, captures_only);
        (moves.to_vec(), mg)
    }

    pub fn append_moves(&mut self, moves: &mut Vec<Move>, captures_only: bool) -> MoveGenerator {
        MoveGenerator::generate(self, moves, captures_only)
    }
}

impl MoveGenerator {
    fn generate(board: &mut Board, moves: &mut Vec<Move>, captures_only: bool) -> Self {
        let mut move_gen = Self::new(board, captures_only);

        move_gen.generate_king_moves(board, moves);

        if !move_gen.in_double_check {
            move_gen.generate_sliding_moves(board, moves);
            move_gen.generate_knight_moves(board, moves);
            move_gen.generate_pawn_moves(board, moves);
        }

        moves.truncate(move_gen.current_move_index);
        move_gen
    }

    fn new(board: &mut Board, captures_only: bool) -> Self {
        let mut mg = Self::default();
        mg.generate_quiet_moves = !captures_only;

        // Store some info for convenience
        mg.white_to_move = board.move_color() == Color::White;
        mg.friendly_color = board.move_color();
        mg.opponent_color = board.opponent_color();
        mg.friendly_king_square = board.king_squares[board.move_color().to_piece_color() as usize];
        mg.friendly_index = board.move_color().to_piece_color();
        mg.enemy_index = 1 - mg.friendly_index;

        // Store some bitboards for convenience
        mg.enemy_pieces = board.color_bbs[mg.enemy_index as usize];
        mg.friendly_pieces = board.color_bbs[mg.friendly_index as usize];
        mg.all_pieces = board.all_piece_bb;
        mg.empty_squares = !mg.all_pieces;
        mg.empty_or_enemy_squares = mg.empty_squares | mg.enemy_pieces;
        mg.move_type_mask = mg
            .generate_quiet_moves
            .then(|| u64::MAX)
            .unwrap_or(mg.enemy_pieces);

        mg.calculate_attack_map(board);
        mg
    }

    fn generate_king_moves(&mut self, board: &Board, moves: &mut Vec<Move>) {
        let legal_mask = !(self.opponent_attack_map | self.friendly_pieces);
        let mut king_moves = bitboard::get_bb_utility().king_moves
            [self.friendly_king_square as usize]
            & legal_mask
            & self.move_type_mask;
        while king_moves != 0 {
            let target_square = BitBoard::pop_lsb(&mut king_moves);
            if self.current_move_index > MAX_MOVES {
                return;
            }

            if self.current_move_index > MAX_MOVES {
                return;
            }
            moves.push(Move::from((self.friendly_king_square, target_square)));

            self.current_move_index += 1;
        }

        if !self.in_check && self.generate_quiet_moves {
            let castle_blockers = self.opponent_attack_map | board.all_piece_bb;
            if board.state.can_castle_kingside(board.white_to_move) {
                let castle_mask = board
                    .white_to_move
                    .then(|| bitboard::WHITE_KINGSIDE_MASK)
                    .unwrap_or(bitboard::BLACK_KINGSIDE_MASK);

                if (castle_mask & castle_blockers) == 0 {
                    let target_square = board.white_to_move.then(|| board::G1).unwrap_or(board::G8);
                    if self.current_move_index > MAX_MOVES {
                        return;
                    }

                    moves.push(Move::from((
                        self.friendly_king_square,
                        target_square,
                        r#move::CASTLE_FLAG,
                    )));
                    self.current_move_index += 1;
                }
            }

            if board.state.can_castle_queenside(board.white_to_move) {
                let castle_mask = board
                    .white_to_move
                    .then(|| bitboard::WHITE_QUEENSIDE_MASK2)
                    .unwrap_or(bitboard::BLACK_QUEENSIDE_MASK2);
                let castle_block_mask = board
                    .white_to_move
                    .then(|| bitboard::WHITE_QUEENSIDE_MASK)
                    .unwrap_or(bitboard::BLACK_QUEENSIDE_MASK);

                if (castle_mask & castle_blockers) == 0
                    && (castle_block_mask & board.all_piece_bb) == 0
                {
                    let target_square = board.white_to_move.then(|| board::C1).unwrap_or(board::C8);
                    if self.current_move_index > MAX_MOVES {
                        return;
                    }

                    moves.push(Move::from(
                        ((
                            self.friendly_king_square,
                            target_square,
                            r#move::CASTLE_FLAG,
                        )),
                    ));
                    self.current_move_index += 1;
                }
            }
        }
    }

    fn generate_sliding_moves(&mut self, board: &Board, moves: &mut Vec<Move>) {
        let move_mask = self.empty_or_enemy_squares & self.check_ray_bitboard & self.move_type_mask;

        let mut orthogonal_sliders = board.friendly_ortho_slider_bb;
        let mut diagonal_sliders = board.friendly_diag_slider_bb;

        if self.in_check {
            orthogonal_sliders &= !self.pin_rays;
            diagonal_sliders &= !self.pin_rays;
        }

        while orthogonal_sliders != 0 {
            let start_square = BitBoard::pop_lsb(&mut orthogonal_sliders);
            let mut move_squares =
                magic::get_magic().get_rook_attacks(start_square, self.all_pieces) & move_mask;

            if self.is_pinned(start_square) {
                move_squares &=
                    get_pmd().align_mask[start_square as usize][self.friendly_king_square as usize];
            }

            while move_squares != 0 {
                let target_square = BitBoard::pop_lsb(&mut move_squares);
                if self.current_move_index > MAX_MOVES {
                    return;
                }

                moves.push(Move::from((start_square, target_square)));
                self.current_move_index += 1;
            }
        }

        while diagonal_sliders != 0 {
            let start_square = BitBoard::pop_lsb(&mut diagonal_sliders);
            let mut move_squares =
                magic::get_magic().get_bishop_attacks(start_square, self.all_pieces) & move_mask;

            if self.is_pinned(start_square) {
                move_squares &=
                    get_pmd().align_mask[start_square as usize][self.friendly_king_square as usize];
            }

            while move_squares != 0 {
                let target_square = BitBoard::pop_lsb(&mut move_squares);
                if self.current_move_index > MAX_MOVES {
                    return;
                }

                moves.push(Move::from((start_square, target_square)));
                self.current_move_index += 1;
            }
        }
    }

    fn generate_knight_moves(&mut self, board: &Board, moves: &mut Vec<Move>) {
        let friendly_knight_piece =
            piece::Piece::from((piece::KNIGHT, board.move_color().to_piece_color()));
        let mut knights = board.piece_bbs[friendly_knight_piece.value as usize] & self.not_pin_rays;
        let move_mask = self.empty_or_enemy_squares & self.check_ray_bitboard & self.move_type_mask;

        while knights != 0 {
            let knight_square = BitBoard::pop_lsb(&mut knights);
            let mut move_squares =
                bitboard::get_bb_utility().knight_attacks[knight_square as usize] & move_mask;

            while move_squares != 0 {
                let target_square = BitBoard::pop_lsb(&mut move_squares);
                if self.current_move_index > MAX_MOVES {
                    return;
                }

                moves.push(Move::from((knight_square, target_square)));
                self.current_move_index += 1;
            }
        }
    }

    fn generate_pawn_moves(&mut self, board: &Board, moves: &mut Vec<Move>) {
        let push_dir = board.white_to_move.then(|| 1).unwrap_or(-1);
        let push_offset = push_dir * 8;

        let friendly_pawn_piece =
            piece::Piece::from((piece::PAWN, board.move_color().to_piece_color()));
        let pawns = board.piece_bbs[friendly_pawn_piece.value as usize];

        let promotion_rank_mask = board
            .white_to_move
            .then(|| bitboard::RANK_8)
            .unwrap_or(bitboard::RANK_1);
        let single_push = BitBoard::shift(pawns, push_offset) & self.empty_squares;
        let mut push_promotions = single_push & promotion_rank_mask & self.check_ray_bitboard;

        let capture_edge_file_mask = board
            .white_to_move
            .then(|| bitboard::NOT_A_FILE)
            .unwrap_or(bitboard::NOT_H_FILE);
        let capture_edge_file_mask2 = board
            .white_to_move
            .then(|| bitboard::NOT_H_FILE)
            .unwrap_or(bitboard::NOT_A_FILE);
        let mut capture_a =
            BitBoard::shift(pawns & capture_edge_file_mask, push_dir * 7) & self.enemy_pieces;
        let mut capture_b =
            BitBoard::shift(pawns & capture_edge_file_mask2, push_dir * 9) & self.enemy_pieces;

        let mut single_push_no_promotion =
            single_push & !promotion_rank_mask & self.check_ray_bitboard;

        let mut capture_promotions_a = capture_a & promotion_rank_mask & self.check_ray_bitboard;
        let mut capture_promotions_b = capture_b & promotion_rank_mask & self.check_ray_bitboard;

        capture_a &= self.check_ray_bitboard & !promotion_rank_mask;
        capture_b &= self.check_ray_bitboard & !promotion_rank_mask;

        // Single / double push
        if self.generate_quiet_moves {
            while single_push_no_promotion != 0 {
                let target_square = BitBoard::pop_lsb(&mut single_push_no_promotion);
                let start_square = target_square - push_offset;

                if !self.is_pinned(start_square)
                    || get_pmd().align_mask[start_square as usize]
                        [self.friendly_king_square as usize]
                        == get_pmd().align_mask[target_square as usize]
                            [self.friendly_king_square as usize]
                {
                    if self.current_move_index > MAX_MOVES {
                        return;
                    }

                    moves.push(Move::from((start_square, target_square)));
                    self.current_move_index += 1;
                }
            }

            let double_push_target_rank_mask = board
                .white_to_move
                .then(|| bitboard::RANK_4)
                .unwrap_or(bitboard::RANK_5);
            let mut double_push = BitBoard::shift(single_push, push_offset)
                & self.empty_squares
                & double_push_target_rank_mask
                & self.check_ray_bitboard;

            while double_push != 0 {
                let target_square = BitBoard::pop_lsb(&mut double_push);
                let start_square = target_square - push_offset * 2;

                if !self.is_pinned(start_square)
                    || get_pmd().align_mask[start_square as usize]
                        [self.friendly_king_square as usize]
                        == get_pmd().align_mask[target_square as usize]
                            [self.friendly_king_square as usize]
                {
                    if self.current_move_index > MAX_MOVES {
                        return;
                    }

                    moves.push(Move::from((
                        start_square,
                        target_square,
                        r#move::PAWN_TWO_UP_FLAG,
                    )));
                    self.current_move_index += 1;
                }
            }
        }

        // Captures
        while capture_a != 0 {
            let target_square = BitBoard::pop_lsb(&mut capture_a);
            let start_square = target_square - push_dir * 7;

            if !self.is_pinned(start_square)
                || get_pmd().align_mask[start_square as usize][self.friendly_king_square as usize]
                    == get_pmd().align_mask[target_square as usize]
                        [self.friendly_king_square as usize]
            {
                if self.current_move_index > MAX_MOVES {
                    return;
                }

                moves.push(Move::from((start_square, target_square)));
                self.current_move_index += 1;
            }
        }

        while capture_b != 0 {
            let target_square = BitBoard::pop_lsb(&mut capture_b);
            let start_square = target_square - push_dir * 9;

            if !self.is_pinned(start_square)
                || get_pmd().align_mask[start_square as usize][self.friendly_king_square as usize]
                    == get_pmd().align_mask[target_square as usize]
                        [self.friendly_king_square as usize]
            {
                if self.current_move_index > MAX_MOVES {
                    return;
                }

                moves.push(Move::from((start_square, target_square)));
                self.current_move_index += 1;
            }
        }

        // Promotions
        while push_promotions != 0 {
            let target_square = BitBoard::pop_lsb(&mut push_promotions);
            let start_square = target_square - push_offset;

            if !self.is_pinned(start_square) {
                self.generate_promotions(start_square, target_square, moves);
            }
        }

        while capture_promotions_a != 0 {
            let target_square = BitBoard::pop_lsb(&mut capture_promotions_a);
            let start_square = target_square - push_dir * 7;

            if !self.is_pinned(start_square)
                || get_pmd().align_mask[start_square as usize][self.friendly_king_square as usize]
                    == get_pmd().align_mask[target_square as usize]
                        [self.friendly_king_square as usize]
            {
                self.generate_promotions(start_square, target_square, moves);
            }
        }

        while capture_promotions_b != 0 {
            let target_square = BitBoard::pop_lsb(&mut capture_promotions_b);
            let start_square = target_square - push_dir * 9;

            if !self.is_pinned(start_square)
                || get_pmd().align_mask[start_square as usize][self.friendly_king_square as usize]
                    == get_pmd().align_mask[target_square as usize]
                        [self.friendly_king_square as usize]
            {
                self.generate_promotions(start_square, target_square, moves);
            }
        }

        if board.state.en_passant_file > 0 {
            let ep_file_idx = board.state.en_passant_file - 1;
            let ep_rank_idx = board.white_to_move.then(|| 5).unwrap_or(2);
            let target_square = ep_rank_idx * 8 + ep_file_idx;
            let captured_pawn_square = target_square - push_offset;

            if BitBoard::contains_square(self.check_ray_bitboard, captured_pawn_square) {
                let mut pawns_that_can_capture_ep =
                    pawns & BitBoard::pawn_attacks(1 << target_square, !board.white_to_move);

                while pawns_that_can_capture_ep != 0 {
                    let start_square = BitBoard::pop_lsb(&mut pawns_that_can_capture_ep);
                    if !self.is_pinned(start_square)
                        || get_pmd().align_mask[start_square as usize]
                            [self.friendly_king_square as usize]
                            == get_pmd().align_mask[target_square as usize]
                                [self.friendly_king_square as usize]
                    {
                        if !self.in_check_after_ep(
                            board,
                            start_square,
                            target_square,
                            captured_pawn_square,
                        ) {
                            if self.current_move_index > MAX_MOVES {
                                return;
                            }

                            moves.push(Move::from(
                                ((start_square, target_square, r#move::EN_PASSANT_CAPTURE_FLAG)),
                            ));
                            self.current_move_index += 1;
                        }
                    }
                }
            }
        }
    }

    fn generate_promotions(
        &mut self,
        start_square: i32,
        target_square: i32,
        moves: &mut Vec<Move>,
    ) {
        if self.current_move_index > MAX_MOVES {
            return;
        }

        moves[self.current_move_index] =
            Move::from((start_square, target_square, r#move::PROMOTE_TO_QUEEN_FLAG));
        self.current_move_index += 1;

        if self.generate_quiet_moves {
            match self.promotions_to_generate {
                PromotionMode::All => {
                    if self.current_move_index > MAX_MOVES {
                        return;
                    }

                    moves[self.current_move_index] =
                        Move::from((start_square, target_square, r#move::PROMOTE_TO_KNIGHT_FLAG));
                    self.current_move_index += 1;
                    if self.current_move_index > MAX_MOVES {
                        return;
                    }

                    moves[self.current_move_index] =
                        Move::from((start_square, target_square, r#move::PROMOTE_TO_ROOK_FLAG));
                    self.current_move_index += 1;
                    if self.current_move_index > MAX_MOVES {
                        return;
                    }

                    moves[self.current_move_index] =
                        Move::from((start_square, target_square, r#move::PROMOTE_TO_BISHOP_FLAG));
                    self.current_move_index += 1;
                }
                PromotionMode::QueenAndKnight => {
                    if self.current_move_index > MAX_MOVES {
                        return;
                    }

                    moves[self.current_move_index] =
                        Move::from((start_square, target_square, r#move::PROMOTE_TO_KNIGHT_FLAG));
                    self.current_move_index += 1;
                }
                _ => {}
            }
        }
    }

    fn generate_sliding_attack_map(&mut self, board: &mut Board) {
        self.opponent_sliding_attack_map = 0;

        self.update_slide_attack(board.all_piece_bb, &mut board.enemy_ortho_slider_bb, true);
        self.update_slide_attack(board.all_piece_bb, &mut board.enemy_diag_slider_bb, false);
    }

    fn calculate_attack_map(&mut self, board: &mut Board) {
        self.generate_sliding_attack_map(board);
        let mut start_dir_idx = 0;
        let mut end_dir_idx = 8;

        if board.queens[self.enemy_index as usize].count() == 0 {
            start_dir_idx = (board.rooks[self.enemy_index as usize].count() > 0)
                .then(|| 0)
                .unwrap_or(4);
            end_dir_idx = (board.bishops[self.enemy_index as usize].count() > 0)
                .then(|| 8)
                .unwrap_or(4);
        }

        for dir in start_dir_idx..end_dir_idx {
            let is_diagonal = dir > 3;
            let slider = is_diagonal
                .then(|| board.enemy_diag_slider_bb)
                .unwrap_or(board.enemy_ortho_slider_bb);
            if (get_pmd().dir_ray_mask[dir][self.friendly_king_square as usize] & slider) == 0 {
                continue;
            }

            let n = get_pmd().num_squares_to_edge[self.friendly_king_square as usize][dir];
            let direction_offset = DIRECTION_OFFSETS[dir];
            let mut is_friendly_piece_along_ray = false;
            let mut ray_mask = 0;

            for i in 0..n {
                let square_idx = self.friendly_king_square + direction_offset * (i + 1);
                ray_mask |= 1 << square_idx;
                let piece = board.squares[square_idx as usize];
                let piece = piece::Piece::from(piece);

                if piece.value != piece::NONE {
                    if piece.is_color(self.friendly_color.to_piece_color()) {
                        if !is_friendly_piece_along_ray {
                            // First friendly piece we have come across in this direction, so it might be pinned
                            is_friendly_piece_along_ray = true;
                        } else {
                            // This is the second friendly piece we've found in this direction, therefore pin is not possible
                            break;
                        }
                    } else {
                        // This square contains an enemy piece
                        let piece_type = piece.get_type();

                        // Check if piece is in bitboard of pieces able to move in current direction
                        if (is_diagonal && piece.can_diag_slide())
                            || (!is_diagonal && piece.can_ortho_slide())
                        {
                            if is_friendly_piece_along_ray {
                                // Friendly piece blocks the check, so this is a pin
                                self.pin_rays |= ray_mask;
                            } else {
                                // No friendly piece blocking the attack, so this is a check
                                self.check_ray_bitboard |= ray_mask;
                                self.in_double_check = self.in_check;
                                self.in_check = true;
                            }
                            break;
                        } else {
                            // This enemy piece is not able to move in the current direction, and so is blocking any checks/pins
                            break;
                        }
                    }
                }
            }

            // Stop searching for pins if in double check, as the king is the only piece able to move in that case anyway
            if self.in_double_check {
                break;
            }
        }

        self.not_pin_rays = !self.pin_rays;

        // Knight attacks
        let mut opponent_knight_attacks = 0;
        let mut knights_bb = board.piece_bbs[piece::Piece::from((
            piece::KNIGHT,
            board.opponent_color().to_piece_color(),
        ))
        .value as usize];
        let friendly_king_bb = board.piece_bbs
            [piece::Piece::from((piece::KING, board.move_color().to_piece_color())).value as usize];

        while knights_bb != 0 {
            let knight_square = BitBoard::pop_lsb(&mut knights_bb);
            let knight_attacks = bitboard::get_bb_utility().knight_attacks[knight_square as usize];
            opponent_knight_attacks |= knight_attacks;

            if (knight_attacks & friendly_king_bb) != 0 {
                self.in_double_check = self.in_check;
                self.in_check = true;
                self.check_ray_bitboard |= 1 << knight_square;
            }
        }

        // Pawn attacks
        let mut opponent_pawn_attack_map = 0;

        let opponent_pawns_bb = board.piece_bbs[piece::Piece::from((
            piece::PAWN,
            board.opponent_color().to_piece_color(),
        ))
        .value as usize];
        opponent_pawn_attack_map = BitBoard::pawn_attacks(opponent_pawns_bb, !self.white_to_move);
        if BitBoard::contains_square(opponent_pawn_attack_map, self.friendly_king_square) {
            self.in_double_check = self.in_check;
            self.in_check = true;
            let possible_pawn_attack_origins = board
                .white_to_move
                .then(|| {
                    bitboard::get_bb_utility().white_pawn_attacks
                        [self.friendly_king_square as usize]
                })
                .unwrap_or(
                    bitboard::get_bb_utility().black_pawn_attacks
                        [self.friendly_king_square as usize],
                );
            let pawn_check_map = opponent_pawns_bb & possible_pawn_attack_origins;
            self.check_ray_bitboard |= pawn_check_map;
        }

        let enemy_king_square = board.king_squares[self.enemy_index as usize];

        self.opponent_attack_map_no_pawns = self.opponent_sliding_attack_map
            | opponent_knight_attacks
            | bitboard::get_bb_utility().king_moves[enemy_king_square as usize];
        self.opponent_attack_map = self.opponent_attack_map_no_pawns | opponent_pawn_attack_map;

        if !self.in_check {
            self.check_ray_bitboard = u64::MAX;
        }
    }
}

// Helper IMPL
impl MoveGenerator {
    fn is_pinned(&self, square: i32) -> bool {
        ((self.pin_rays >> square) & 1) != 0
    }

    fn update_slide_attack(&mut self, all_piece_bb: u64, piece_bb: &mut u64, ortho: bool) {
        let blockers = all_piece_bb & !(1 << self.friendly_king_square);

        while *piece_bb != 0 {
            let start_square = BitBoard::pop_lsb(piece_bb);
            let move_board = magic::get_magic().get_slider_attacks(start_square, blockers, ortho);

            self.opponent_sliding_attack_map |= move_board;
        }
    }

    fn in_check_after_ep(
        &self,
        board: &Board,
        start_square: i32,
        target_square: i32,
        ep_capture_square: i32,
    ) -> bool {
        let enemy_ortho = board.enemy_ortho_slider_bb;

        if enemy_ortho != 0 {
            let masked_blockers =
                self.all_pieces ^ (1 << ep_capture_square | 1 << start_square | 1 << target_square);
            let rook_attacks =
                magic::get_magic().get_rook_attacks(self.friendly_king_square, masked_blockers);
            (rook_attacks & enemy_ortho) != 0
        } else {
            false
        }
    }
}

// Orthogonal and diagonal: N, S, W, E, NW, SE, NE, SW
pub const DIRECTION_OFFSETS: [i32; 8] = [8, -8, -1, 1, 7, -7, 9, -9];

const DIR_OFFSETS_2D: [Coord; 8] = [
    Coord::from((0, 1)),
    Coord::from((0, -1)),
    Coord::from((-1, 0)),
    Coord::from((1, 0)),
    Coord::from((-1, 1)),
    Coord::from((1, -1)),
    Coord::from((1, 1)),
    Coord::from((-1, -1)),
];

pub const PAWN_ATTACK_DIRECTIONS: [[u8; 2]; 2] = [[4, 6], [7, 5]];

const ALL_KNIGHT_JUMPS: [i32; 8] = [15, 17, -17, -15, 10, -6, 6, -10];

#[derive(Debug)]
pub struct PrecomputedMoveData {
    pub align_mask: [[u64; 64]; 64],
    pub dir_ray_mask: [[u64; 64]; 8],

    pub num_squares_to_edge: [[i32; 8]; 64],

    pub knight_moves: [Vec<u8>; 64],
    pub king_moves: [Vec<u8>; 64],

    pub pawn_attacks_white: [Vec<i32>; 64],
    pub pawn_attacks_black: [Vec<i32>; 64],
    pub direction_lookup: [i32; 127],

    pub king_attack_bitboards: [u64; 64],
    pub knight_attack_bitboards: [u64; 64],
    pub pawn_attack_bitboards: [[u64; 64]; 2],

    pub rook_moves: [u64; 64],
    pub bishop_moves: [u64; 64],
    pub queen_moves: [u64; 64],

    pub orthogonal_distance: [[i32; 64]; 64],
    pub king_distance: [[i32; 64]; 64],
    pub center_manhattan_distance: [i32; 64],
}

static PRECOMPUTED_MOVE_DATA: OnceLock<PrecomputedMoveData> = OnceLock::new();

pub fn get_pmd() -> &'static PrecomputedMoveData {
    PRECOMPUTED_MOVE_DATA.get_or_init(PrecomputedMoveData::new)
}

impl PrecomputedMoveData {
    fn new() -> Self {
        let mut data = Self::default();

        for square_idx in 0..64 {
            let y = square_idx / 8;
            let x = square_idx - y + 8;

            let north = 7 - y;
            let south = y;
            let west = x;
            let east = 7 - x;

            data.num_squares_to_edge[square_idx as usize][0] = north;
            data.num_squares_to_edge[square_idx as usize][1] = south;
            data.num_squares_to_edge[square_idx as usize][2] = west;
            data.num_squares_to_edge[square_idx as usize][3] = east;
            data.num_squares_to_edge[square_idx as usize][4] = north.min(west);
            data.num_squares_to_edge[square_idx as usize][5] = south.min(east);
            data.num_squares_to_edge[square_idx as usize][6] = north.min(east);
            data.num_squares_to_edge[square_idx as usize][7] = south.min(west);

            let mut legal_knight_jumps = Vec::new();
            let mut knight_bb = 0;

            // Calculate all squares knight can jump to from current square
            ALL_KNIGHT_JUMPS.iter().for_each(|knight_jump_delta| {
                let knight_jump_square = square_idx + knight_jump_delta;
                if knight_jump_square >= 0 && knight_jump_square < 64 {
                    let knight_square_y = knight_jump_square / 8;
                    let knight_square_x = knight_jump_square - knight_square_y * 8;

                    let max_coord_move_dst =
                        (x - knight_square_x).abs().max((y - knight_square_y).abs());
                    if max_coord_move_dst == 2 {
                        legal_knight_jumps.push(knight_jump_square as u8);
                        knight_bb |= 1 << knight_jump_square;
                    }
                }
            });
            data.knight_moves[square_idx as usize] = legal_knight_jumps;
            data.knight_attack_bitboards[square_idx as usize] = knight_bb;

            // Calculate all squares king can move to from current square (not including castling)
            let mut legal_king_moves = Vec::new();
            DIRECTION_OFFSETS.iter().for_each(|king_move_delta| {
                let king_move_square = square_idx + king_move_delta;
                if king_move_square >= 0 && king_move_square < 64 {
                    let king_square_y = king_move_square / 8;
                    let king_square_x = king_move_square - king_square_y * 8;

                    let max_coord_move_dst =
                        (x - king_square_x).abs().max((y - king_square_y).abs());
                    if max_coord_move_dst == 1 {
                        legal_king_moves.push(king_move_square as u8);
                        data.king_attack_bitboards[square_idx as usize] |= 1 << king_move_square;
                    }
                }
            });
            data.king_moves[square_idx as usize] = legal_king_moves;

            // Calculate legal pawn captures for white and black
            let mut pawn_captures_white = Vec::new();
            let mut pawn_captures_black = Vec::new();

            if x > 0 {
                if y < 7 {
                    pawn_captures_white.push(square_idx + 7);
                    data.pawn_attack_bitboards[square_idx as usize][Color::White as usize] |=
                        1 << (square_idx + 7);
                }
                if y > 0 {
                    pawn_captures_black.push(square_idx - 9);
                    data.pawn_attack_bitboards[square_idx as usize][Color::Black as usize] |=
                        1 << (square_idx - 9);
                }
            }

            if x < 7 {
                if y < 7 {
                    pawn_captures_white.push(square_idx + 9);
                    data.pawn_attack_bitboards[square_idx as usize][Color::White as usize] |=
                        1 << (square_idx + 9);
                }
                if y > 0 {
                    pawn_captures_black.push(square_idx - 7);
                    data.pawn_attack_bitboards[square_idx as usize][Color::White as usize] |=
                        1 << (square_idx - 7);
                }
            }

            data.pawn_attacks_white[square_idx as usize] = pawn_captures_white;
            data.pawn_attacks_black[square_idx as usize] = pawn_captures_black;

            // Rook & Bishop Moves
            for direction_idx in 0..8 {
                let current_dir_offset = DIRECTION_OFFSETS[direction_idx];
                for n in 0..data.num_squares_to_edge[square_idx as usize][direction_idx] {
                    let target_square = square_idx + current_dir_offset * (n + 1);
                    if direction_idx < 4 {
                        data.rook_moves[square_idx as usize] |= 1 << target_square;
                    } else {
                        data.bishop_moves[square_idx as usize] |= 1 << target_square;
                    }
                }
            }

            // Queen Moves
            data.queen_moves[square_idx as usize] =
                data.rook_moves[square_idx as usize] | data.bishop_moves[square_idx as usize];
        }

        // Direction lu
        for i in 0..127 {
            let offset: i32 = i - 63;
            let abs_offset = offset.abs();
            let mut abs_dir = 1;

            for &d in &[9, 8, 7] {
                if abs_offset % d == 0 {
                    abs_dir = d;
                    break;
                }
            }

            data.direction_lookup[i as usize] = abs_dir * offset.signum();
        }

        // Distance lu - POTENTIAL OPTIMIZATION
        for square_a in 0..64 {
            let coord_a = Coord::new(square_a);
            let file_dst_from_center = (3 - coord_a.file_idx).max(coord_a.file_idx - 4);
            let rank_dst_from_center = (3 - coord_a.rank_idx).max(coord_a.rank_idx - 4);
            data.center_manhattan_distance[square_a as usize] =
                file_dst_from_center + rank_dst_from_center;

            for square_b in 0..64 {
                let coord_b = Coord::new(square_b);
                let file_distance = (coord_a.file_idx - coord_b.file_idx).abs();
                let rank_distance = (coord_a.rank_idx - coord_b.rank_idx).abs();
                data.orthogonal_distance[square_a as usize][square_b as usize] =
                    file_distance + rank_distance;
                data.king_distance[square_a as usize][square_b as usize] =
                    file_distance.max(rank_distance);
            }
        }

        for square_a in 0..64 {
            for square_b in 0..64 {
                let coord_a = Coord::new(square_a);
                let coord_b = Coord::new(square_b);
                let delta = coord_a - coord_b;
                let dir = Coord::from((delta.file_idx.signum(), delta.rank_idx.signum()));

                for i in -8..8 {
                    let coord = Coord::new(square_a) + dir * i;
                    if coord.is_valid_square() {
                        data.align_mask[square_a as usize][square_b as usize] |= 1 << coord.index();
                    }
                }
            }
        }

        DIR_OFFSETS_2D
            .iter()
            .enumerate()
            .for_each(|(dir_idx, dir_offset)| {
                for square_idx in 0..64 {
                    let square = Coord::new(square_idx);

                    for i in 0..8 {
                        let coord = square + dir_offset * i;
                        if coord.is_valid_square() {
                            data.dir_ray_mask[dir_idx][square_idx as usize] |= 1 << coord.index();
                        } else {
                            break;
                        }
                    }
                }
            });

        data
    }

    fn default() -> Self {
        Self {
            align_mask: [[u64::default(); 64]; 64],
            dir_ray_mask: [[u64::default(); 64]; 8],
            num_squares_to_edge: [[i32::default(); 8]; 64],
            knight_moves: std::array::from_fn(|_| Vec::new()),
            king_moves: std::array::from_fn(|_| Vec::new()),
            pawn_attacks_white: std::array::from_fn(|_| Vec::new()),
            pawn_attacks_black: std::array::from_fn(|_| Vec::new()),
            direction_lookup: [i32::default(); 127],
            king_attack_bitboards: [u64::default(); 64],
            knight_attack_bitboards: [u64::default(); 64],
            pawn_attack_bitboards: [[u64::default(); 64]; 2],
            rook_moves: [u64::default(); 64],
            bishop_moves: [u64::default(); 64],
            queen_moves: [u64::default(); 64],
            orthogonal_distance: [[i32::default(); 64]; 64],
            king_distance: [[i32::default(); 64]; 64],
            center_manhattan_distance: [i32::default(); 64],
        }
    }
}

// Helper IMPL
impl PrecomputedMoveData {
    pub fn num_rook_moves_to_reach_square(&self, start_square: i32, target_square: i32) -> i32 {
        self.orthogonal_distance[start_square as usize][target_square as usize]
    }

    pub fn num_king_moves_to_reach_square(&self, start_square: i32, target_square: i32) -> i32 {
        self.king_distance[start_square as usize][target_square as usize]
    }
}
