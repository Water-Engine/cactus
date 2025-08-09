use crate::engine::game::{board::Board, coord::Coord, piece};

pub const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/**
Get the fen string of the current position
* Ref: https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation
*/
pub fn current_fen(board: &Board, always_show_ep: bool) -> String {
    let mut fen = String::new();
    for rank in 7..=0 {
        let mut num_empty_files = 0;
        for file in 0..8 {
            let i = Coord::from((file, rank)).index();
            let current_piece = board.squares[i as usize];
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

    fen.push_str(&format!(" {}", board.white_to_move));
    let white_kingside = (board.state.castling_rights & 1) == 1;
    let white_queenside = (board.state.castling_rights >> 1 & 1) == 1;
    let black_kingside = (board.state.castling_rights >> 2 & 1) == 1;
    let black_queenside = (board.state.castling_rights >> 3 & 1) == 1;

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
    if board.state.castling_rights == 0 {
        fen.push('-');
    }

    fen.push(' ');
    let ep_file_idx = board.state.en_passant_file - 1;
    let ep_rank_idx = if board.white_to_move { 5 } else { 2 };
    let ep_coord = Coord::from((ep_file_idx, ep_rank_idx));
    let ep_name = ep_coord.to_string();

    let is_ep = ep_file_idx != -1;
    let include_ep = always_show_ep || ep_possible(ep_file_idx, ep_rank_idx, board);
    fen.push_str(if is_ep && include_ep { &ep_name } else { "-" });

    fen.push_str(&format!(" {}", board.state.halfmove_clock));
    fen.push_str(&format!(" {}", (board.ply_count / 2) + 1));

    fen
}

fn ep_possible(ep_file_idx: i32, ep_rank_idx: i32, board: &Board) -> bool {
    todo!("Not implemented")
}
