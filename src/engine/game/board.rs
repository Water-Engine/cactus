use std::collections::VecDeque;

use crate::engine::{
    game::{
        board, coord::Coord, r#move::{self, Move}, piece::{self, PieceList}, state::{State, Zobrist}
    },
    generate::bitboard::BitBoard,
    utils::fen::{self, PositionInfo},
};

pub const ROOK_DIRECTIONS: [Coord; 4] = [
    Coord::from((-1, 0)),
    Coord::from((1, 0)),
    Coord::from((0, 1)),
    Coord::from((0, -1)),
];

pub const BISHOP_DIRECTIONS: [Coord; 4] = [
    Coord::from((-1, 1)),
    Coord::from((1, 1)),
    Coord::from((1, -1)),
    Coord::from((-1, -1)),
];

pub const FILE_NAMES: &[char] = &['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
pub const RANK_NAMES: &[char] = &['1', '2', '3', '4', '5', '6', '7', '8'];

pub const A1: usize = 0;
pub const B1: usize = 1;
pub const C1: usize = 2;
pub const D1: usize = 3;
pub const E1: usize = 4;
pub const F1: usize = 5;
pub const G1: usize = 6;
pub const H1: usize = 7;

pub const A8: usize = 56;
pub const B8: usize = 57;
pub const C8: usize = 58;
pub const D8: usize = 59;
pub const E8: usize = 60;
pub const F8: usize = 61;
pub const G8: usize = 62;
pub const H8: usize = 63;

#[derive(Clone, Copy)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    pub fn to_piece_color(&self) -> i32 {
        match self {
            Self::White => 0,
            Self::Black => 1,
        }
    }
}

#[derive(Debug)]
pub struct Board {
    pub squares: [i32; 64],
    pub king_squares: [i32; 2],

    pub piece_bbs: [u64; piece::MAX_PIECE_INDEX + 1],
    pub color_bbs: [u64; 2],
    pub all_piece_bb: u64,
    pub friendly_ortho_slider_bb: u64,
    pub friendly_diag_slider_bb: u64,
    pub enemy_ortho_slider_bb: u64,
    pub enemy_diag_slider_bb: u64,

    /// Count of all material excluding pawns and kings
    pub total_heavy_material: usize,

    pub rooks: [PieceList; 2],
    pub bishops: [PieceList; 2],
    pub queens: [PieceList; 2],
    pub knights: [PieceList; 2],
    pub pawns: [PieceList; 2],

    pub white_to_move: bool,
    pub repetition_history: VecDeque<u64>,

    pub ply_count: i32,
    pub state: State,
    pub all_moves: Vec<Move>,

    all_piece_lists: [PieceList; piece::MAX_PIECE_INDEX + 1],
    game_state_history: VecDeque<State>,
    start_pos_info: PositionInfo,
    cached_in_check_value: bool,
    has_cached_in_check_value: bool,
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    pub fn new() -> Self {
        let rooks = [PieceList::new(); 2];
        let bishops = [PieceList::new(); 2];
        let queens = [PieceList::new(); 2];
        let knights = [PieceList::new(); 2];
        let pawns = [PieceList::new(); 2];

        let mut all_pl = [PieceList::new(); piece::MAX_PIECE_INDEX + 1];
        all_pl[piece::WHITE_PAWN as usize] = pawns[Color::White as usize];
        all_pl[piece::WHITE_KNIGHT as usize] = knights[Color::White as usize];
        all_pl[piece::WHITE_BISHOP as usize] = bishops[Color::White as usize];
        all_pl[piece::WHITE_ROOK as usize] = rooks[Color::White as usize];
        all_pl[piece::WHITE_QUEEN as usize] = queens[Color::White as usize];
        all_pl[piece::WHITE_KING as usize] = PieceList::new();

        all_pl[piece::BLACK_PAWN as usize] = pawns[Color::Black as usize];
        all_pl[piece::BLACK_KNIGHT as usize] = knights[Color::Black as usize];
        all_pl[piece::BLACK_BISHOP as usize] = bishops[Color::Black as usize];
        all_pl[piece::BLACK_ROOK as usize] = rooks[Color::Black as usize];
        all_pl[piece::BLACK_QUEEN as usize] = queens[Color::Black as usize];
        all_pl[piece::BLACK_KING as usize] = PieceList::new();

        Self {
            squares: [i32::default(); 64],
            king_squares: [i32::default(); 2],

            piece_bbs: [u64::default(); piece::MAX_PIECE_INDEX + 1],
            color_bbs: [u64::default(); 2],
            all_piece_bb: 0,
            friendly_ortho_slider_bb: 0,
            friendly_diag_slider_bb: 0,
            enemy_ortho_slider_bb: 0,
            enemy_diag_slider_bb: 0,

            total_heavy_material: 0,

            rooks: rooks,
            bishops: bishops,
            queens: queens,
            knights: knights,
            pawns: pawns,

            white_to_move: true,
            repetition_history: VecDeque::new(),

            ply_count: 0,
            state: State::default(),
            all_moves: Vec::new(),

            all_piece_lists: all_pl,
            game_state_history: VecDeque::new(),
            start_pos_info: PositionInfo::default(),
            cached_in_check_value: false,
            has_cached_in_check_value: false,
        }
    }

    pub fn make_move(&mut self, mv: Move, in_search: bool) {
        let move_color = self.move_color();
        let opponent_color = self.opponent_color();

        let start_square = mv.start_square();
        let target_square = mv.target_square();
        let move_flag = mv.move_flag();
        let is_promotion = mv.is_promotion();
        let is_ep = move_flag == r#move::EN_PASSANT_CAPTURE_FLAG;

        let moved_piece = self.squares[start_square as usize];
        let moved_piece_type = piece::Piece::from(moved_piece).get_type();
        let captured_piece = is_ep
            .then(|| piece::Piece::from((piece::PAWN, (opponent_color.to_piece_color()))).value)
            .unwrap_or(self.squares[target_square as usize]);
        let captured_piece_type = piece::Piece::from(captured_piece).get_type();

        let prev_castle_state = self.state.castling_rights;
        let prev_en_passant_file = self.state.en_passant_file;
        let mut new_zobrist_key = self.state.zobrist.key;
        let mut new_castling_rights = self.state.castling_rights;
        let new_en_passant_file = 0;

        self.move_piece(moved_piece, start_square, target_square);

        // Captures
        if captured_piece_type != piece::NONE {
            let mut capture_square = target_square;

            if is_ep {
                capture_square = target_square + self.white_to_move.then(|| -8).unwrap_or(8);
                self.squares[capture_square as usize] = piece::NONE;
            }

            if captured_piece_type != piece::NONE {
                self.total_heavy_material -= 1;
            }

            self.all_piece_lists[captured_piece as usize].remove_piece(capture_square);
            BitBoard::toggle_square(&mut self.piece_bbs[captured_piece as usize], capture_square);
            BitBoard::toggle_square(&mut self.color_bbs[opponent_color as usize], capture_square);
            new_zobrist_key ^=
                self.state.zobrist.pieces_array[captured_piece as usize][capture_square as usize];
        }

        // King moves
        if moved_piece_type == piece::KING {
            self.king_squares[move_color as usize] = target_square;
            new_castling_rights &= self.white_to_move.then(|| 0b1100).unwrap_or(0b0011);

            if move_flag == r#move::CASTLE_FLAG {
                let rook_piece = piece::Piece::from((piece::ROOK, move_color.to_piece_color()));
                let king_side = target_square == board::G1 as i32 || target_square == board::G8 as i32;
                let castling_rook_from_idx = king_side.then(|| target_square + 1).unwrap_or(target_square - 2);
                let castling_rook_to_idx = king_side.then(|| target_square - 1).unwrap_or(target_square + 1);

                // STOPPED HERE 8/10 MIDNIGHT
            }
        }
    }

    /**
    A raw piece move. Updates piece lists and board info without respect to:
    * Removal of a captured piece
    * Movement of rook when castling
    * Removal of pawn from 1st/8th rank during pawn promotion
    * Addition of promoted piece during pawn promotion
    */
    fn move_piece(&mut self, piece: i32, start_square: i32, target_square: i32) {
        let move_color = self.move_color();
        BitBoard::toggle_squares(
            &mut self.piece_bbs[piece as usize],
            &[start_square, target_square],
        );
        BitBoard::toggle_squares(
            &mut self.color_bbs[move_color as usize],
            &[start_square, target_square],
        );

        self.all_piece_lists[piece as usize].move_piece(start_square, target_square);
        self.squares[start_square as usize] = piece::NONE;
        self.squares[target_square as usize] = piece;
    }

    fn update_slider_bbs(&mut self) {
        let move_color = self.move_color();
        let friendly_rook = piece::Piece::from((piece::ROOK, move_color.to_piece_color())).value;
        let friendly_queen = piece::Piece::from((piece::QUEEN, move_color.to_piece_color())).value;
        let friendly_bishop =
            piece::Piece::from((piece::BISHOP, move_color.to_piece_color())).value;
        self.friendly_ortho_slider_bb =
            self.piece_bbs[friendly_rook as usize] | self.piece_bbs[friendly_queen as usize];
        self.friendly_diag_slider_bb =
            self.piece_bbs[friendly_bishop as usize] | self.piece_bbs[friendly_queen as usize];

        let opponent_color = self.opponent_color();
        let enemy_rook = piece::Piece::from((piece::ROOK, opponent_color.to_piece_color())).value;
        let enemy_queen = piece::Piece::from((piece::QUEEN, opponent_color.to_piece_color())).value;
        let enemy_bishop =
            piece::Piece::from((piece::BISHOP, opponent_color.to_piece_color())).value;
        self.enemy_ortho_slider_bb =
            self.piece_bbs[enemy_rook as usize] | self.piece_bbs[enemy_queen as usize];
        self.enemy_diag_slider_bb =
            self.piece_bbs[enemy_bishop as usize] | self.piece_bbs[enemy_queen as usize];
    }

    pub fn load_start_pos(&mut self) -> Result<(), String> {
        self.load_from_fen(fen::STARTING_FEN)
    }

    pub fn load_from_fen(&mut self, fen: &str) -> Result<(), String> {
        let position = fen::position_from_fen(fen.to_string())?;
        self.load_from_position(position);
        Ok(())
    }

    pub fn load_from_position(&mut self, position: PositionInfo) {
        self.reset();

        for square_idx in 0..64 {
            let square = position.squares[square_idx];
            let piece = piece::Piece::from(square);
            let piece_type = piece.get_type();
            let color_idx = if piece.is_white() {
                Color::White
            } else {
                Color::Black
            };
            self.squares[square_idx] = square;

            if piece_type != piece::NONE {
                BitBoard::set_square(&mut self.piece_bbs[piece.value as usize], square_idx as i32);
                BitBoard::set_square(&mut self.color_bbs[color_idx as usize], square_idx as i32);

                if piece_type == piece::KING {
                    self.king_squares[color_idx as usize] = square_idx as i32;
                } else {
                    self.all_piece_lists[piece.value as usize].add_piece(square_idx as i32);
                }
                self.total_heavy_material += if [piece::PAWN, piece::KING].contains(&piece_type) {
                    0
                } else {
                    1
                };
            }
        }

        self.update_slider_bbs();
        let white_castle = position.white_castle_kingside.then(|| 1 << 0).unwrap_or(0)
            | position.white_castle_queenside.then(|| 1 << 1).unwrap_or(0);
        let black_castle = position.black_castle_kingside.then(|| 1 << 2).unwrap_or(0)
            | position.black_castle_queenside.then(|| 1 << 3).unwrap_or(0);
        let castling_rights = white_castle | black_castle;

        self.ply_count =
            (position.move_count - 1) * 2 + (self.white_to_move.then(|| 0).unwrap_or(1));

        self.state = State::new(
            piece::NONE,
            position.ep_file,
            castling_rights,
            position.halfmove_clock as i32,
            Zobrist::default(),
        );
        let mut zobrist = Zobrist::new();
        zobrist.key(self);
        self.state = State::new(
            piece::NONE,
            position.ep_file,
            castling_rights,
            position.halfmove_clock as i32,
            zobrist,
        );

        self.repetition_history.push_back(zobrist.key);
        self.game_state_history.push_back(self.state);

        self.all_piece_bb =
            self.color_bbs[Color::White as usize] | self.color_bbs[Color::Black as usize];
        self.white_to_move = position.white_to_move;
        self.start_pos_info = position;
    }
}

// Helper IMPL
impl Board {
    pub fn move_color(&self) -> Color {
        self.white_to_move
            .then(|| Color::White)
            .unwrap_or(Color::Black)
    }

    pub fn opponent_color(&self) -> Color {
        self.white_to_move
            .then(|| Color::Black)
            .unwrap_or(Color::White)
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn to_string(&self) -> String {
        self.diagram(self.white_to_move, true, true)
    }

    pub fn diagram(&self, black_at_top: bool, include_fen: bool, include_zobrist: bool) -> String {
        let mut diagram = String::new();
        let last_move_square = if self.all_moves.len() > 0 {
            self.all_moves[self.all_moves.len() - 1].target_square()
        } else {
            -1
        };

        for y in 0..8 {
            let rank_idx = if black_at_top { 7 - y } else { y };
            diagram.push_str("+---+---+---+---+---+---+---+---+\n");

            for x in 0..8 {
                let file_idx = if black_at_top { x } else { 7 - x };
                let square_idx = Coord::from((file_idx, rank_idx)).index();
                let highlight = square_idx == last_move_square;
                let piece = piece::Piece::from(self.squares[square_idx as usize]);

                if highlight {
                    diagram.push_str(&format!("|({})", piece.get_symbol()));
                } else {
                    diagram.push_str(&format!("| {} ", piece.get_symbol()));
                }

                if x == 7 {
                    diagram.push_str(&format!("| {}\n", rank_idx + 1));
                }
            }

            if y == 7 {
                diagram.push_str("+---+---+---+---+---+---+---+---+\n");
                let file_names = "  a   b   c   d   e   f   g   h  ";
                let file_names_rev = "  h   g   f   e   d   c   b   a  ";
                diagram.push_str(&format!(
                    "{}\n\n",
                    if black_at_top {
                        file_names
                    } else {
                        file_names_rev
                    }
                ));

                if include_fen {
                    diagram.push_str(&format!("Fen         : {}\n", fen::current_fen(self, true)));
                }

                if include_zobrist {
                    diagram.push_str(&format!("Zobrist Key : {}", self.state.zobrist.key));
                }
            }
        }
        diagram
    }
}
