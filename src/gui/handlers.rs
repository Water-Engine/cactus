use crate::{core::board::State, gui::launch::Cactus};

use eframe::egui::{Pos2, Response};

impl Cactus {
    pub fn handle_pointer_pressed(&mut self, pos: Pos2, response: &Response) {
        if let Some((rank, file)) = self.get_square_at_pos(pos, response.rect) {
            if let Some(piece_kind) = self.board.piece_at((rank, file)) {
                if let State::Playing { turn } = self.board.state {
                    if turn.color != piece_kind.color() {
                        return;
                    }
                }
                self.dragging = Some((piece_kind, rank, file));
                self.drag_pos = pos;
                self.board.set_piece((rank, file), None);
                self.selected = None;
            }
        }
    }

    pub fn handle_pointer_down(&mut self, pos: Pos2) {
        if self.dragging.is_some() {
            self.drag_pos = pos;
        }
    }

    pub fn handle_pointer_released(&mut self, pos: Pos2, response: &Response) {
        if let Some((piece_kind, orig_rank, orig_file)) = self.dragging.take() {
            let (target_rank, target_file) = self
                .get_square_at_pos(pos, response.rect)
                .unwrap_or((orig_rank, orig_file));
            self.board
                .set_piece((orig_rank, orig_file), Some(piece_kind));

            if self.board.is_move_legal(
                (orig_rank, orig_file),
                (target_rank, target_file),
            ) {
                match self
                    .board
                    .move_piece((orig_rank, orig_file), (target_rank, target_file))
                {
                    Ok(_) => {
                        self.board.play_turn();
                        self.move_sound();
                        if let Some(center) = self.board.center_at((target_rank, target_file)) {
                            self.drag_pos = center;
                        }
                        self.selected = None;
                    }
                    Err(_) => {
                        self.board
                            .set_piece((orig_rank, orig_file), Some(piece_kind));
                        if let Some(center) = self.board.center_at((orig_rank, orig_file)) {
                            self.drag_pos = center;
                        }
                    }
                }
            } else {
                self.board
                    .set_piece((orig_rank, orig_file), Some(piece_kind));
                if let Some(center) = self.board.center_at((orig_rank, orig_file)) {
                    self.drag_pos = center;
                }
            }

            self.drag_pos = self.board.centers[target_rank][target_file];
            self.selected = None;
        } else if let Some((rank, file)) = self.get_square_at_pos(pos, response.rect) {
            self.handle_selection(rank, file);
        }
    }

    pub fn handle_click_selection(&mut self, response: &Response) {
        if let Some(pos) = response.interact_pointer_pos() {
            if let Some((rank, file)) = self.get_square_at_pos(pos, response.rect) {
                self.handle_selection(rank, file);
            }
        }
    }

    fn handle_selection(&mut self, rank: usize, file: usize) {
        match self.selected {
            Some((sel_rank, sel_file)) => {
                if sel_rank == rank && sel_file == file {
                    self.selected = None;
                } else {
                    let selected_piece = self.board.piece_at((sel_rank, sel_file));
                    let target_piece = self.board.piece_at((rank, file));

                    if let Some(piece) = selected_piece {
                        if let State::Playing { turn } = self.board.state {
                            if piece.color() != turn.color {
                                return;
                            }
                        }

                        let can_move = match target_piece {
                            None => true,
                            Some(target) => target.color() != piece.color(),
                        };

                        if can_move {
                            if self.board.piece_at((sel_rank, sel_file)).is_some() {
                                if self.board.is_move_legal(
                                    (sel_rank, sel_file),
                                    (rank, file),
                                ) {
                                    if self
                                        .board
                                        .move_piece((sel_rank, sel_file), (rank, file))
                                        .is_ok()
                                    {
                                        self.board.play_turn();
                                        self.move_sound();
                                        self.drag_pos = self.board.centers[rank][file];
                                        self.clear_selection = true;
                                    }
                                }
                            }
                        } else {
                            self.selected = Some((rank, file));
                        }
                    } else {
                        self.selected = Some((rank, file));
                    }
                }
            }
            None => {
                if self.board.piece_at((rank, file)).is_some() {
                    self.selected = Some((rank, file));
                }
            }
        }
    }
}
