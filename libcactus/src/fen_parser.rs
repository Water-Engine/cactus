use crate::bitboard::ChessPieces;

pub fn fen_to_board(fen: &str) -> Result<ChessPieces, String> {
    let mut board = ChessPieces::empty();

    // Get only the piece placement part (before any spaces)
    let piece_placement = fen.split_whitespace().next().unwrap_or(fen);

    let ranks: Vec<&str> = piece_placement.split('/').collect();

    if ranks.len() != 8 {
        return Err(format!(
            "Invalid FEN: expected 8 ranks, got {}",
            ranks.len()
        ));
    }

    // Process ranks from 8 to 1 (FEN starts from rank 8)
    for (rank_idx, rank_str) in ranks.iter().enumerate() {
        let rank = 7 - rank_idx; // Convert to 0-based from bottom
        let mut file = 0;

        for ch in rank_str.chars() {
            if file >= 8 {
                return Err(format!("Invalid FEN: too many files in rank {}", rank + 1));
            }

            if ch.is_ascii_digit() {
                // Empty squares
                let empty_count = ch.to_digit(10).unwrap() as usize;
                file += empty_count;
            } else {
                // Piece character
                let square = (rank * 8 + file) as u8;
                let bit = 1u64 << square;

                match ch {
                    'P' => board.white_pawns |= bit,
                    'N' => board.white_knights |= bit,
                    'B' => board.white_bishops |= bit,
                    'R' => board.white_rooks |= bit,
                    'Q' => board.white_queens |= bit,
                    'K' => board.white_king |= bit,
                    'p' => board.black_pawns |= bit,
                    'n' => board.black_knights |= bit,
                    'b' => board.black_bishops |= bit,
                    'r' => board.black_rooks |= bit,
                    'q' => board.black_queens |= bit,
                    'k' => board.black_king |= bit,
                    _ => return Err(format!("Invalid piece character: {}", ch)),
                }

                file += 1;
            }
        }

        if file != 8 {
            return Err(format!(
                "Invalid FEN: rank {} has {} files, expected 8",
                rank + 1,
                file
            ));
        }
    }

    Ok(board)
}

pub fn board_to_fen(board: &ChessPieces) -> String {
    let mut fen = String::new();

    for rank in (0..8).rev() {
        let mut empty_count = 0;

        for file in 0..8 {
            let square = (rank * 8 + file) as u8;
            let bit = 1u64 << square;

            let piece = match () {
                _ if board.white_pawns & bit != 0 => Some('P'),
                _ if board.white_knights & bit != 0 => Some('N'),
                _ if board.white_bishops & bit != 0 => Some('B'),
                _ if board.white_rooks & bit != 0 => Some('R'),
                _ if board.white_queens & bit != 0 => Some('Q'),
                _ if board.white_king & bit != 0 => Some('K'),
                _ if board.black_pawns & bit != 0 => Some('p'),
                _ if board.black_knights & bit != 0 => Some('n'),
                _ if board.black_bishops & bit != 0 => Some('b'),
                _ if board.black_rooks & bit != 0 => Some('r'),
                _ if board.black_queens & bit != 0 => Some('q'),
                _ if board.black_king & bit != 0 => Some('k'),
                _ => None,
            };

            if let Some(p) = piece {
                if empty_count > 0 {
                    fen.push_str(&empty_count.to_string());
                    empty_count = 0;
                }
                fen.push(p);
            } else {
                empty_count += 1;
            }
        }

        if empty_count > 0 {
            fen.push_str(&empty_count.to_string());
        }

        if rank > 0 {
            fen.push('/');
        }
    }

    fen
}
