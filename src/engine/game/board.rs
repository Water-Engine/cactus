use std::collections::VecDeque;

use crate::engine::game::{r#move::Move, piece::{self, PieceList}, state::State};

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

impl Board {

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