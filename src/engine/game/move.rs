use crate::engine::{
    game::{
        board::{self, Board},
        coord::Coord,
        r#move, piece,
    },
    generate::move_generator::MoveGenerator,
    utils::fen,
};

use super::piece::{BISHOP, KNIGHT, NONE, Piece, QUEEN, ROOK};

pub const NO_FLAG: i32 = 0b0000;
pub const EN_PASSANT_CAPTURE_FLAG: i32 = 0b0001;
pub const CASTLE_FLAG: i32 = 0b0010;
pub const PAWN_TWO_UP_FLAG: i32 = 0b0011;

pub const PROMOTE_TO_QUEEN_FLAG: i32 = 0b0100;
pub const PROMOTE_TO_KNIGHT_FLAG: i32 = 0b0101;
pub const PROMOTE_TO_ROOK_FLAG: i32 = 0b0110;
pub const PROMOTE_TO_BISHOP_FLAG: i32 = 0b0111;

const START_SQUARE_MASK: u16 = 0b0000000000111111;
const TARGET_SQUARE_MASK: u16 = 0b0000111111000000;

/**
Compact Move Representation (ffffttttttssssss)
* Bits 0-5: start square index
* Bits 6-11: target square index
* Bits 12-15: flag (promotion type, etc)
*/
#[derive(Debug, Default, Eq, PartialEq, Clone, Copy)]
pub struct Move {
    pub value: u16,
}

impl From<u16> for Move {
    fn from(value: u16) -> Self {
        Self { value: value }
    }
}

impl From<(i32, i32)> for Move {
    fn from((start_square, target_square): (i32, i32)) -> Self {
        Self {
            value: (start_square | target_square << 6) as u16,
        }
    }
}

impl From<(i32, i32, i32)> for Move {
    fn from((start_square, target_square, flag): (i32, i32, i32)) -> Self {
        Self {
            value: (start_square | target_square << 6 | flag << 12) as u16,
        }
    }
}

impl Move {
    pub fn null() -> Self {
        Self { value: 0 }
    }

    pub fn is_null(&self) -> bool {
        self.value == 0
    }

    pub fn start_square(&self) -> i32 {
        (self.value & START_SQUARE_MASK) as i32
    }

    pub fn target_square(&self) -> i32 {
        ((self.value & TARGET_SQUARE_MASK) >> 6) as i32
    }

    pub fn move_flag(&self) -> i32 {
        (self.value >> 12) as i32
    }

    pub fn is_promotion(&self) -> bool {
        self.move_flag() >= PROMOTE_TO_QUEEN_FLAG
    }

    pub fn promotion_type(&self) -> Piece {
        Piece::from(match self.move_flag() {
            PROMOTE_TO_ROOK_FLAG => ROOK,
            PROMOTE_TO_KNIGHT_FLAG => KNIGHT,
            PROMOTE_TO_BISHOP_FLAG => BISHOP,
            PROMOTE_TO_QUEEN_FLAG => QUEEN,
            _ => NONE,
        })
    }
}

// Helper IMPL
impl Move {
    /**
    Converts a UCI move name into internal move representation
    * Promotions can be written with or without equals sign
    * Examples: "e7e8=q", "e7e8q"
    */
    pub fn from_uci(board: &Board, move_name: String) -> Self {
        let start_coord = Coord::from_string(move_name[0..3].to_string());
        let start_square: i32 = start_coord.index();
        let target_coord = Coord::from_string(move_name[2..5].to_string());
        let target_square: i32 = target_coord.index();

        let moved_piece_type = Piece::from(board.squares[start_square as usize]).get_type();
        let mut flag = NO_FLAG;

        if moved_piece_type == piece::PAWN {
            if move_name.len() > 4 {
                flag = match move_name.chars().nth(move_name.len() - 1) {
                    Some('q') => PROMOTE_TO_QUEEN_FLAG,
                    Some('r') => PROMOTE_TO_ROOK_FLAG,
                    Some('n') => PROMOTE_TO_KNIGHT_FLAG,
                    Some('b') => PROMOTE_TO_BISHOP_FLAG,
                    _ => NO_FLAG,
                };
            } else if (target_coord.rank_idx - start_coord.rank_idx).abs() == 2 {
                flag = PAWN_TWO_UP_FLAG;
            } else if start_coord.file_idx != target_coord.file_idx
                && board.squares[target_square as usize] == piece::NONE
            {
                flag = EN_PASSANT_CAPTURE_FLAG;
            }
        } else if moved_piece_type == piece::KING {
            if (start_coord.file_idx - target_coord.file_idx).abs() > 1 {
                flag = CASTLE_FLAG;
            }
        }

        Self::from((start_square, target_square, flag))
    }

    /**
    Get algebraic name of move (with promotion specified)
    * Examples: "e2e4", "e7e8q"
    */
    pub fn to_uci(&self) -> String {
        let start_square_name = Coord::new(self.start_square()).to_string();
        let target_square_name = Coord::new(self.target_square()).to_string();
        let mut move_name = start_square_name + &target_square_name;

        if self.is_promotion() {
            match self.move_flag() {
                PROMOTE_TO_ROOK_FLAG => move_name.push('r'),
                PROMOTE_TO_KNIGHT_FLAG => move_name.push('n'),
                PROMOTE_TO_BISHOP_FLAG => move_name.push('b'),
                PROMOTE_TO_QUEEN_FLAG => move_name.push('q'),
                _ => {}
            }
        }

        move_name
    }

    /**
    Get move from the given name in Standard Algebraic Notation (SAN)
    * The given board must contain the position from before the move was made
    * Examples: "Nxf3", "Rad1", "O-O"
    * I am a never nesters worst nightmare
    */
    pub fn from_san(board: &mut Board, algebraic_move: String) -> Self {
        let algebraic_move = algebraic_move
            .replace("+", "")
            .replace("#", "")
            .replace("x", "to")
            .replace("-", "");
        let (all_moves, _) = board.generate_moves(false);
        let mut mv = Self::default();

        for move_to_test in all_moves {
            mv = move_to_test;

            let move_from_idx = mv.start_square();
            let move_to_idx = mv.target_square();
            let move_piece = piece::Piece::from(board.squares[move_from_idx as usize]);
            let move_piece_type = move_piece.get_type();

            let from_coord = Coord::new(move_from_idx);
            let to_coord = Coord::new(move_to_idx);

            let algebraic_move_chars: Vec<char> = algebraic_move.chars().collect();

            if algebraic_move == "OO" {
                // Kingside castle
                if move_piece_type == piece::KING && move_to_idx - move_from_idx == 2 {
                    return mv;
                }
            } else if algebraic_move == "OOO" {
                // Queenside castle
                if move_piece_type == piece::KING && move_to_idx - move_from_idx == -2 {
                    return mv;
                }
            } else if board::FILE_NAMES.contains(&algebraic_move_chars[0]) {
                // Is pawn move if starts with any file indicator
                if move_piece_type != piece::PAWN {
                    continue;
                }

                if let Some(idx) = board::FILE_NAMES
                    .iter()
                    .position(|c| c == &algebraic_move_chars[0])
                {
                    // Are we in the right spot?
                    if idx as i32 != from_coord.file_idx {
                        continue;
                    }

                    if algebraic_move.contains('=') {
                        if to_coord.rank_idx == 0 || to_coord.rank_idx == 7 {
                            if algebraic_move.len() == 5 {
                                let target_file = algebraic_move_chars[1];
                                if let Some(idx) =
                                    board::FILE_NAMES.iter().position(|c| c == &target_file)
                                {
                                    if idx as i32 != to_coord.file_idx {
                                        continue;
                                    }
                                }
                            }
                            let promotion_char =
                                algebraic_move_chars[algebraic_move_chars.len() - 1];

                            if mv.promotion_type() != piece::Piece::from(promotion_char) {
                                continue;
                            }

                            return mv;
                        }
                    } else {
                        let target_file = algebraic_move_chars[algebraic_move_chars.len() - 2];
                        let target_rank = algebraic_move_chars[algebraic_move_chars.len() - 1];

                        if let Some(idx) = board::FILE_NAMES.iter().position(|c| c == &target_file)
                        {
                            if idx as i32 == to_coord.file_idx
                                && target_rank.to_string() == (to_coord.rank_idx + 1).to_string()
                            {
                                break;
                            }
                        }
                    }
                }
            } else {
                // Regular piece move
                let move_piece_char = algebraic_move_chars[0];
                if piece::Piece::from(move_piece_char).value != move_piece_type {
                    continue;
                }

                let target_file = algebraic_move_chars[algebraic_move_chars.len() - 2];
                let target_rank = algebraic_move_chars[algebraic_move_chars.len() - 1];
                if let Some(idx) = board::FILE_NAMES.iter().position(|c| c == &target_file) {
                    if idx as i32 != to_coord.file_idx {
                        continue;
                    }

                    if target_rank.to_string() == (to_coord.rank_idx + 1).to_string() {
                        if algebraic_move_chars.len() == 4 {
                            let disambiguation_char = algebraic_move_chars[1];
                            if board::FILE_NAMES.contains(&disambiguation_char) {
                                if let Some(idx) = board::FILE_NAMES
                                    .iter()
                                    .position(|c| c == &disambiguation_char)
                                {
                                    if idx as i32 != from_coord.file_idx {
                                        continue;
                                    }
                                }
                            } else if disambiguation_char.to_string()
                                != (from_coord.rank_idx + 1).to_string()
                            {
                                continue;
                            }
                        }
                        break;
                    }
                }
            }
        }

        mv
    }

    /**
    Get name of move in Standard Algebraic Notation (SAN)
    * The move must not yet have been made on the board
    * Examples: "Nxf3", "Rad1", "O-O"
    */
    pub fn to_san(&self, board: &mut Board) -> String {
        if self.is_null() {
            return "Null".to_string();
        }
        let move_piece = piece::Piece::from(board.squares[self.start_square() as usize]);
        let move_piece_type = move_piece.get_type();
        let captured_piece = piece::Piece::from(board.squares[self.target_square() as usize]);
        let captured_piece_type = captured_piece.get_type();

        if self.move_flag() == r#move::CASTLE_FLAG {
            let delta = self.target_square() - self.start_square();
            if delta == 2 {
                return "O-O".to_string();
            } else if delta == -2 {
                return "O-O-O".to_string();
            }
        }
        let mut move_notation = move_piece.get_symbol().to_uppercase().to_string();

        if move_piece_type != piece::PAWN && move_piece_type != piece::KING {
            let (all_moves, _) = board.generate_moves(false);

            for alt_move in all_moves {
                if alt_move.start_square() != self.start_square()
                    && alt_move.target_square() == self.target_square()
                {
                    let alt_move_piece =
                        piece::Piece::from(board.squares[alt_move.start_square() as usize]);
                    if alt_move_piece.get_type() == move_piece_type {
                        let from_file_idx = Coord::file_of_square(self.start_square());
                        let alternate_from_file_idx =
                            Coord::file_of_square(alt_move.target_square());
                        let from_rank_idx = Coord::rank_of_square(self.start_square());
                        let alternate_from_rank_idx =
                            Coord::rank_of_square(alt_move.target_square());

                        if from_file_idx != alternate_from_file_idx {
                            move_notation.push(board::FILE_NAMES[from_file_idx as usize]);
                            break;
                        } else if from_rank_idx != alternate_from_rank_idx {
                            move_notation.push(board::RANK_NAMES[from_rank_idx as usize]);
                            break;
                        }
                    }
                }
            }
        }

        if captured_piece_type != 0 {
            if move_piece_type == piece::PAWN {
                move_notation
                    .push(board::FILE_NAMES[Coord::file_of_square(self.start_square()) as usize]);
            }
            move_notation.push('x');
        } else {
            if self.move_flag() == r#move::EN_PASSANT_CAPTURE_FLAG {
                move_notation
                    .push(board::FILE_NAMES[Coord::file_of_square(self.start_square()) as usize]);
                move_notation.push('x');
            }
        }

        move_notation.push(board::FILE_NAMES[Coord::file_of_square(self.target_square()) as usize]);
        move_notation.push(board::RANK_NAMES[Coord::rank_of_square(self.target_square()) as usize]);

        if self.is_promotion() {
            let promotion_piece_type = self.promotion_type();
            move_notation.push('=');
            move_notation.push_str(&promotion_piece_type.get_symbol().to_uppercase().to_string());
        }

        board.make_move(*self, true);
        let (legal_responses, mg) = board.generate_moves(false);
        if mg.in_check {
            if legal_responses.len() == 0 {
                move_notation.push('#');
            } else {
                move_notation.push('+');
            }
        }

        board.unmake_move(*self, true);

        move_notation
    }
}

pub fn pgn_from_moves(moves: &Vec<Move>) -> String {
    board::create_pgn(
        moves,
        super::arbiter::GameResult::InProgress,
        fen::STARTING_FEN,
        "",
        "",
    )
}
