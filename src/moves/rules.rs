use crate::core::{
    Color,
    board::{Board, State},
    piece::{PieceKind, PieceType},
};

impl Board {
    pub fn is_move_legal(
        &self,
        from: (usize, usize),
        to: (usize, usize),
    ) -> bool {
        let Some(piece) = self.piece_at(from) else {
            return false;
        };
        let color_to_move = match self.state {
            State::Playing { turn } => turn.color,
            _ => return false,
        };

        if piece.color() != color_to_move {
            return false;
        }

        is_valid_piece_move(piece, from, to)
    }
}

fn is_valid_piece_move(piece: PieceKind, from: (usize, usize), to: (usize, usize)) -> bool {
    match piece.to_type() {
        PieceType::Pawn => validate_pawn_move(piece.color(), from, to),
        PieceType::Knight => validate_knight_move(from, to),
        PieceType::Bishop => validate_bishop_move(from, to),
        PieceType::Rook => validate_rook_move(from, to),
        PieceType::Queen => validate_queen_move(from, to),
        PieceType::King => validate_king_move(from, to),
    }
}

fn validate_pawn_move(color: Color, from: (usize, usize), to: (usize, usize)) -> bool {
    let (fr, ff) = from;
    let (tr, tf) = to;

    let dir: isize = match color {
        Color::White => -1,
        Color::Black => 1,
    };

    let start_rank = match color {
        Color::White => 6,
        Color::Black => 1,
    };

    let dr = tr as isize - fr as isize;
    let df = tf as isize - ff as isize;

    match (dr, df) {
        (d, 0) if d == dir => true,
        (d, 0) if fr == start_rank && d == 2 * dir => true,
        (d, 1) | (d, -1) if d == dir => true,
        _ => false,
    }
}

fn validate_knight_move(from: (usize, usize), to: (usize, usize)) -> bool {
    let (fr, ff) = from;
    let (tr, tf) = to;
    let dr = (fr as isize - tr as isize).abs();
    let df = (ff as isize - tf as isize).abs();
    (dr == 2 && df == 1) || (dr == 1 && df == 2)
}

fn validate_bishop_move(from: (usize, usize), to: (usize, usize)) -> bool {
    let (fr, ff) = from;
    let (tr, tf) = to;
    (fr as isize - tr as isize).abs() == (ff as isize - tf as isize).abs()
}

fn validate_rook_move(from: (usize, usize), to: (usize, usize)) -> bool {
    let (fr, ff) = from;
    let (tr, tf) = to;
    fr == tr || ff == tf
}

fn validate_queen_move(from: (usize, usize), to: (usize, usize)) -> bool {
    validate_rook_move(from, to) || validate_bishop_move(from, to)
}

fn validate_king_move(from: (usize, usize), to: (usize, usize)) -> bool {
    let (fr, ff) = from;
    let (tr, tf) = to;
    let dr = (fr as isize - tr as isize).abs();
    let df = (ff as isize - tf as isize).abs();
    dr <= 1 && df <= 1
}
