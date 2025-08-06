use crate::core::{Color, STARTING_COLOR, piece::*};

use eframe::egui::{Pos2, Rect};

#[derive(Copy, Clone)]
pub struct Square {
    pub piece: Option<PieceKind>,
}

#[derive(Copy, Clone)]
pub struct Board {
    pub squares: [[Square; 8]; 8],
    pub centers: [[Pos2; 8]; 8],
    pub state: State,
    pub players: Players,
}

#[derive(Copy, Clone)]
pub enum State {
    Playing { turn: Player },
    Checkmate { winner: Player },
    Stalemate,
    Draw,
}

impl Default for State {
    fn default() -> Self {
        State::Playing {
            turn: Player::from_color(STARTING_COLOR),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Player {
    pub color: Color,
}

impl Player {
    pub fn from_color(color: Color) -> Self {
        Self { color }
    }
}

#[derive(Copy, Clone)]
pub struct Players {
    pub white: Player,
    pub black: Player,
}

impl Default for Players {
    fn default() -> Self {
        Self {
            white: Player::from_color(Color::White),
            black: Player::from_color(Color::Black),
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        use PieceKind::*;

        let empty_square = Square { piece: None };
        let squares = [[empty_square; 8]; 8];
        let centers = [[Pos2::ZERO; 8]; 8];

        let mut board = Board {
            squares,
            centers,
            state: State::default(),
            players: Players::default(),
        };

        for i in 0..8 {
            board.squares[1][i].piece = Some(BlackPawn);
            board.squares[6][i].piece = Some(WhitePawn);
        }
        let back_rank = [
            BlackRook,
            BlackKnight,
            BlackBishop,
            BlackQueen,
            BlackKing,
            BlackBishop,
            BlackKnight,
            BlackRook,
        ];
        let front_rank = [
            WhiteRook,
            WhiteKnight,
            WhiteBishop,
            WhiteQueen,
            WhiteKing,
            WhiteBishop,
            WhiteKnight,
            WhiteRook,
        ];

        for i in 0..8 {
            board.squares[0][i].piece = Some(back_rank[i]);
            board.squares[7][i].piece = Some(front_rank[i]);
        }

        board
    }
}

impl Board {
    pub fn is_valid_pos(pos: (usize, usize)) -> bool {
        let (r, f) = pos;
        r < 8 && f < 8
    }

    pub fn piece_at(&self, pos: (usize, usize)) -> Option<PieceKind> {
        if Self::is_valid_pos(pos) {
            let (r, f) = pos;
            self.squares[r][f].piece
        } else {
            None
        }
    }

    pub fn center_at(&self, (rank, file): (usize, usize)) -> Option<Pos2> {
        if Self::is_valid_pos((rank, file)) {
            Some(self.centers[rank][file])
        } else {
            None
        }
    }

    pub fn set_piece(&mut self, pos: (usize, usize), piece: Option<PieceKind>) {
        if Self::is_valid_pos(pos) {
            let (r, f) = pos;
            self.squares[r][f].piece = piece;
        }
    }

    pub fn move_piece(
        &mut self,
        from: (usize, usize),
        to: (usize, usize),
    ) -> Result<PieceKind, String> {
        if !Self::is_valid_pos(from) || !Self::is_valid_pos(to) {
            return Err("Position out of bounds".into());
        }

        let piece = self.piece_at(from).ok_or("No piece at from-position")?;

        if let State::Playing { turn } = self.state {
            if piece.color() != turn.color {
                return Err("Not your turn".into());
            }
        } else {
            return Err("Game is not in playing state".into());
        }

        if let Some(dest_piece) = self.piece_at(to) {
            if dest_piece.color() == piece.color() {
                return Err("Cannot capture your own piece".into());
            }
        }

        self.set_piece(to, Some(piece));
        self.set_piece(from, None);

        Ok(piece)
    }

    pub fn play_turn(&mut self) {
        match self.state {
            State::Playing { turn } => {
                self.state = State::Playing {
                    turn: if turn.color == Color::White {
                        self.players.black
                    } else {
                        self.players.white
                    },
                }
            }
            _ => todo!("Not implemented yet!"),
        }
    }

    pub fn refresh(rect: Rect, old_board: Board) -> Self {
        use PieceKind::*;
        let square_size = rect.width() / 8.0;

        let mut centers = [[Pos2::ZERO; 8]; 8];
        for rank in 0..8 {
            for file in 0..8 {
                let x = rect.left() + (file as f32 + 0.5) * square_size;
                let y = rect.top() + (rank as f32 + 0.5) * square_size;
                centers[rank][file] = Pos2::new(x, y);
            }
        }

        let mut board = Board {
            squares: [[Square { piece: None }; 8]; 8],
            centers,
            state: old_board.state,
            players: old_board.players,
        };
        for i in 0..8 {
            board.squares[1][i].piece = Some(BlackPawn);
            board.squares[6][i].piece = Some(WhitePawn);
        }
        let back_rank = [
            BlackRook,
            BlackKnight,
            BlackBishop,
            BlackQueen,
            BlackKing,
            BlackBishop,
            BlackKnight,
            BlackRook,
        ];
        let front_rank = [
            WhiteRook,
            WhiteKnight,
            WhiteBishop,
            WhiteQueen,
            WhiteKing,
            WhiteBishop,
            WhiteKnight,
            WhiteRook,
        ];

        for i in 0..8 {
            board.squares[0][i].piece = Some(back_rank[i]);
            board.squares[7][i].piece = Some(front_rank[i]);
        }

        board
    }
}
