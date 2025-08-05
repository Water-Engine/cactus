use eframe::egui::{Pos2, Response};

use crate::gui::launch::Cactus;

impl Cactus {
    pub fn handle_pointer_pressed(&mut self, pos: Pos2, response: &Response) {
        if let Some((rank, file)) = self.get_square_at_pos(pos, response.rect) {
            if let Some(piece_kind) = self.board.piece_at((rank, file)) {
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
                .set_piece((target_rank, target_file), Some(piece_kind));
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

                    if selected_piece.is_some() {
                        if target_piece.is_none() {
                            if self
                                .board
                                .move_piece((sel_rank, sel_file), (rank, file))
                                .is_ok()
                            {
                                self.drag_pos = self.board.centers[rank][file];
                                self.clear_selection = true;
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
