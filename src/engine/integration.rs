use crate::{
    core::{
        Color,
        board::{Board, State},
        piece::{PieceKind, PieceType},
    },
    gui::launch::Cactus,
    moves::moves::Move,
};

impl Board {
    pub fn parse_uci_move(&self, uci: &str) -> Option<Move> {
        let bytes = uci.as_bytes();
        if bytes.len() < 4 {
            return None;
        }

        let f1 = (bytes[0] - b'a') as usize;
        let r1 = 8 - (bytes[1] - b'0') as usize;
        let f2 = (bytes[2] - b'a') as usize;
        let r2 = 8 - (bytes[3] - b'0') as usize;

        let from = (r1, f1);
        let to = (r2, f2);

        let piece = self.piece_at(from)?;

        let promotion = if bytes.len() == 5 {
            Some(match bytes[4] as char {
                'q' => PieceType::Queen,
                'r' => PieceType::Rook,
                'b' => PieceType::Bishop,
                'n' => PieceType::Knight,
                _ => return None,
            })
        } else {
            None
        };

        Some(Move {
            from,
            to,
            promotion,
            piece,
        })
    }

    pub fn move_history_uci(&self) -> Vec<String> {
        self.moves.iter().map(|m| m.to_uci()).collect()
    }

    pub fn apply_uci_move(&mut self, uci: &str) -> Option<PieceKind> {
        if let Some(mv) = self.parse_uci_move(uci) {
            let promotion = mv.promotion.map(|pt| PieceKind::new(pt, mv.piece.color()));
            match self.move_piece(mv.from, mv.to, promotion) {
                Ok((_, captured)) => {
                    return captured;
                }
                Err(e) => {
                    eprintln!("Failed to apply UCI move `{}`: {}", uci, e);
                }
            }
        } else {
            eprintln!("Invalid UCI move: `{}`", uci);
        }

        None
    }
}

impl Cactus {
    pub fn try_engine_turn(&mut self) {
        if self.is_engine_turn() && !self.waiting_for_engine_move {
            if let Some(engine) = &self.engine {
                let uci_moves = self.board.move_history_uci();
                let position_cmd = format!("position startpos moves {}", uci_moves.join(" "));
                engine.send_command(position_cmd);

                engine.send_command("go movetime 50".to_string());

                self.waiting_for_engine_move = true;
            }
        }

        if let Some(engine) = &self.engine {
            if let Some(bestmove) = engine.try_receive_response() {
                let captured = self.board.apply_uci_move(&bestmove);
                match captured {
                    Some(_) => self.capture_sound(),
                    None => self.move_sound(),
                }
                self.board.update_state();
                match self.board.state {
                    State::Checkmate { .. } | State::Stalemate | State::Draw => {
                        self.handle_game_over();
                    }
                    _ => {}
                }
                self.waiting_for_engine_move = false;
            }
        }
    }

    fn is_engine_turn(&self) -> bool {
        match (self.engine_is_black, &self.board.state) {
            (Some(true), State::Playing { turn: Color::Black }) => true,
            (Some(false), State::Playing { turn: Color::White }) => true,
            _ => false,
        }
    }
}
