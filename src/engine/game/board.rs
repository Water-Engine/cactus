use std::collections::VecDeque;

use crate::engine::{game::{
    coord::Coord,
    r#move::Move,
    piece::{self, PieceList},
    state::State,
}, utils::fen};

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

pub enum Color {
    White = 0,
    Black = 1,
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
    cached_in_check_value: bool,
    has_cached_in_check_value: bool,
}

impl Default for Board {
    fn default() -> Self {
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
            cached_in_check_value: false,
            has_cached_in_check_value: false,
        }
    }
}

// Helper IMPL
impl Board {
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
                    diagram.push_str(&format!("Fen         : {}", fen::current_fen(self, true)));
                }
                
                if include_zobrist {
                    diagram.push_str(&format!("Zobrist Key : {}", self.state.zobrist_key));
                }
            }
        }
        diagram
    }
}