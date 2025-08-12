use crate::engine::game::{
    board::{Board, Color},
    coord::Coord,
};

#[derive(Debug, Clone, Copy)]
pub enum Result {
    NotStarted,
    InProgress,
    WhiteIsMated,
    BlackIsMated,
    Stalemate,
    Repetition,
    FiftyMoveRule,
    InsufficientMaterial,
    DrawByArbiter,
    WhiteTimeout,
    BlackTimeout,
    WhiteIllegalMove,
    BlackIllegalMove,
}

pub fn is_draw(result: Result) -> bool {
    match result {
        Result::DrawByArbiter
        | Result::FiftyMoveRule
        | Result::Repetition
        | Result::Stalemate
        | Result::InsufficientMaterial => true,
        _ => false,
    }
}

pub fn is_win(result: Result) -> bool {
    white_winner(result) || black_winner(result)
}

pub fn white_winner(result: Result) -> bool {
    match result {
        Result::BlackIsMated | Result::BlackTimeout | Result::BlackIllegalMove => true,
        _ => false,
    }
}

pub fn black_winner(result: Result) -> bool {
    match result {
        Result::WhiteIsMated | Result::WhiteTimeout | Result::WhiteIllegalMove => true,
        _ => false,
    }
}

pub fn get_game_state(board: &Board) -> Result {
    todo!("Not implemented")
}

pub fn insufficient_material(board: &Board) -> bool {
    if board.pawns[Color::White as usize].count() > 0
        || board.pawns[Color::Black as usize].count() > 0
    {
        return false;
    }

    if board.friendly_ortho_slider_bb != 0 || board.enemy_ortho_slider_bb != 0 {
        return false;
    }

    let num_white_bishops = board.bishops[Color::White as usize].count();
    let num_black_bishops = board.bishops[Color::Black as usize].count();
    let num_white_knights = board.knights[Color::White as usize].count();
    let num_black_knights = board.knights[Color::Black as usize].count();
    let num_white_minors = num_white_bishops + num_white_knights;
    let num_black_minors = num_black_bishops + num_black_knights;
    let num_minors = num_white_minors + num_black_minors;

    // King vs King + single minor:
    if num_minors <= 1 {
        return true;
    }

    // Bishop vs bishop: is insufficient when bishops are same color
    if num_minors == 2 && num_white_bishops == 1 && num_black_bishops == 1 {
        Coord::new(board.bishops[Color::White as usize][0]).is_light_square()
            == Coord::new(board.bishops[Color::Black as usize][0]).is_light_square()
    } else {
        false
    }
}
