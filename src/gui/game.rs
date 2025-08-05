use crate::{core::board::Board, gui::launch::Cactus};

use eframe::{
    Frame,
    egui::{Color32, Context, Pos2, Rect, Response, Sense, Ui, Vec2},
};

impl Cactus {
    pub fn handle_event(&mut self, ctx: &Context, _frame: &mut Frame, ui: &mut Ui) -> Response {
        let max_size = ui.available_size();
        let size = Vec2::splat(max_size.x.min(max_size.y));
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        let response = ui.interact(rect, ui.id().with("chessboard"), Sense::click_and_drag());
        let painter = ui.painter_at(rect);
        self.painter = Some(painter);
        self.size = size;

        if self.board.center_at((0, 0)) == Some(Pos2::ZERO)
            || self.board_size != response.rect.size()
        {
            let mut new_board = Board::refresh(response.rect);
            for rank in 0..8 {
                for file in 0..8 {
                    let pos = (rank, file);
                    new_board.set_piece(pos, self.board.piece_at(pos));
                }
            }
            self.board = new_board;
            self.board_size = response.rect.size();
        }

        let square_size = size.x / 8.0;
        self.maybe_update_textures(ctx, square_size);

        self.clear_selection = false;

        let pointer = ctx.input(|i| i.pointer.clone());
        if let Some(pos) = pointer.interact_pos() {
            if pointer.primary_pressed() {
                self.handle_pointer_pressed(pos, &response);
            }
            if pointer.primary_down() {
                self.handle_pointer_down(pos);
            }
            if pointer.primary_released() {
                self.handle_pointer_released(pos, &response);
            }
            if response.clicked() {
                self.handle_click_selection(&response);
            }

            if self.clear_selection {
                self.selected = None;
            }
        }

        response
    }

    pub fn render(&mut self, response: &Response) {
        let rect = response.rect;
        let square_size = rect.width() / 8.0;
        let painter = self
            .painter
            .as_mut()
            .expect("Painter required for rendering");

        for rank in 0..8 {
            for file in 0..8 {
                let x = rect.left() + file as f32 * square_size;
                let y = rect.top() + rank as f32 * square_size;
                let square_rect = Rect::from_min_size(Pos2::new(x, y), Vec2::splat(square_size));

                let is_light = (rank + file) % 2 == 0;
                painter.rect_filled(
                    square_rect,
                    0.0,
                    if is_light {
                        Color32::from_rgb(240, 217, 181)
                    } else {
                        Color32::from_rgb(181, 136, 99)
                    },
                );

                if self.dragging.is_none() && !self.clear_selection {
                    if let Some((sel_rank, sel_file)) = self.selected {
                        if sel_rank == rank && sel_file == file {
                            painter.rect_filled(
                                square_rect,
                                0.0,
                                Color32::from_rgba_unmultiplied(255, 255, 0, 80),
                            );
                        }
                    }
                }

                if let Some(kind) = self.board.piece_at((rank, file)) {
                    if let Some((_, drag_rank, drag_file)) = self.dragging {
                        if drag_rank == rank && drag_file == file {
                            continue;
                        }
                    }

                    if let Some(texture) = self.images.get(kind) {
                        let center = self
                            .board
                            .center_at((rank, file))
                            .expect("Position out of bounds");
                        let texture_size = texture.size_vec2();
                        let scale = (square_size * 0.9) / texture_size.x.min(texture_size.y);
                        let image_size = texture_size * scale;
                        let top_left = center - image_size / 2.0;

                        let image_rect = Rect::from_min_size(top_left, image_size);

                        painter.image(
                            texture.id(),
                            image_rect,
                            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    }
                }
            }
        }

        if let Some((piece_kind, _orig_rank, _orig_file)) = self.dragging {
            if let Some(texture) = self.images.get(piece_kind) {
                let texture_size = texture.size_vec2();
                let scale = (self.size.x / 8.0 * 0.9) / texture_size.x.min(texture_size.y);
                let image_size = texture_size * scale;
                let top_left = self.drag_pos - image_size / 2.0;
                let image_rect = Rect::from_min_size(top_left, image_size);
                painter.image(
                    texture.id(),
                    image_rect,
                    Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
        }
    }

    pub fn maybe_update_textures(&mut self, ctx: &Context, square_size: f32) {
        let threshold = 25.0;
        if (square_size - self.board_size.x).abs() > threshold {
            self.images.update_textures(ctx, square_size * 0.9);
            self.board_size = Vec2::splat(square_size);
        }
    }

    pub fn get_square_at_pos(&self, pos: Pos2, board_rect: Rect) -> Option<(usize, usize)> {
        let square_size = board_rect.width() / 8.0;
        let col = ((pos.x - board_rect.left()) / square_size).floor() as usize;
        let row = ((pos.y - board_rect.top()) / square_size).floor() as usize;
        if Board::is_valid_pos((row, col)) {
            Some((row, col))
        } else {
            None
        }
    }
}
