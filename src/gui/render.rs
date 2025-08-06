use eframe::egui::{
    Align, Align2, Color32, Context, Id, ImageButton, Layout, Pos2, Rect, Response, RichText,
    Stroke, StrokeKind, Ui, Vec2, Window, vec2,
};

use crate::{
    core::{
        Color,
        board::State,
        piece::{PieceKind, PieceType},
    },
    gui::{DEFAULT_PANEL_SIZE, launch::Cactus},
};

impl Cactus {
    pub fn render(&mut self, response: &Response, ctx: &Context) {
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

                    let texture = self.images.get_texture(kind);
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

        if let Some((piece_kind, _orig_rank, _orig_file)) = self.dragging {
            let texture = self.images.get_texture(piece_kind);
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

        self.render_promotion_popup(ctx);
        self.render_side_panel(ctx, rect);
        if self.show_game_over_popup {
            self.render_game_over_popup(ctx);
        }
    }

    pub fn render_promotion_popup(&mut self, ctx: &Context) {
        if let Some(((from_r, from_f), (to_r, to_f))) = self.promotion_pending {
            let color = self.board.piece_at((from_r, from_f)).unwrap().color();

            Window::new("Promotion")
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Promote to:");

                    ui.horizontal(|ui| {
                        let promotion_pieces = [
                            PieceType::Queen,
                            PieceType::Rook,
                            PieceType::Bishop,
                            PieceType::Knight,
                        ];

                        for &pt in promotion_pieces.iter() {
                            let piece_kind = PieceKind::new(pt, color);
                            let texture_id = self.images.get_texture(piece_kind);

                            let image_button = ImageButton::new(texture_id);

                            if ui.add(image_button).clicked() {
                                if self.board.is_move_legal(
                                    (from_r, from_f),
                                    (to_r, to_f),
                                    Some(piece_kind),
                                ) {
                                    if let Ok((_, captured)) = self.board.move_piece(
                                        (from_r, from_f),
                                        (to_r, to_f),
                                        Some(piece_kind),
                                    ) {
                                        match captured {
                                            Some(_) => self.capture_sound(),
                                            None => self.confirmation_sound(),
                                        }
                                        self.board.update_state();

                                        match self.board.state {
                                            State::Checkmate { .. }
                                            | State::Stalemate
                                            | State::Draw => {
                                                self.handle_game_over();
                                                return;
                                            }
                                            _ => {}
                                        }

                                        self.drag_pos = self.board.centers[to_r][to_f];
                                    }
                                }

                                self.promotion_pending = None;
                                self.selected = None;
                            }
                        }
                    });
                });
        }
    }

    pub fn render_side_panel(&self, ctx: &eframe::egui::Context, board_rect: Rect) {
        let panel_width = DEFAULT_PANEL_SIZE;
        let spacing = 12.0;
        let panel_pos = Pos2::new(board_rect.right() + DEFAULT_PANEL_SIZE, board_rect.top());

        eframe::egui::Area::new(Id::new("custom_right_panel"))
            .fixed_pos(panel_pos)
            .show(ctx, |ui| {
                let eval_bar_height = 150.0;
                let eval_bar_width = 24.0;

                ui.set_width(panel_width);
                ui.add_space(spacing);
                ui.vertical_centered(|ui| {
                    self.render_player_label_and_captures(ui, Color::Black);
                });

                ui.add_space(spacing);
                ui.vertical_centered(|ui| {
                    self.render_evaluation_bar(ui, eval_bar_width, eval_bar_height);
                });

                ui.add_space(spacing);
                ui.with_layout(Layout::bottom_up(eframe::egui::Align::Center), |ui| {
                    self.render_player_label_and_captures(ui, Color::White);
                });
            });
    }

    fn render_player_label_and_captures(&self, ui: &mut Ui, color: Color) {
        let player = match color {
            Color::White => &self.board.players.white,
            Color::Black => &self.board.players.black,
        };

        let name = match color {
            Color::White => "White",
            Color::Black => "Black",
        };

        let text_color = Color32::from_rgb(230, 230, 230);

        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new(format!("{name}: {}", player.score))
                    .strong()
                    .color(text_color)
                    .size(32.0),
            );
            if !player.captures.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    for piece in &player.captures {
                        let tex = self.images.get_capture(*piece);
                        ui.image(tex);
                    }
                });
            }
        });
    }

    fn render_evaluation_bar(&self, ui: &mut Ui, width: f32, height: f32) {
        let bar_size = Vec2::new(width, height);
        let (rect, _) = ui.allocate_exact_size(bar_size, eframe::egui::Sense::hover());

        let painter = ui.painter();
        let outline = Stroke::new(1.0, Color32::GRAY);
        painter.rect_stroke(rect, 2.0, outline, StrokeKind::Middle);

        let eval = 0.68;
        let fill_height = bar_size.y * eval;

        let fill_rect = Rect::from_min_size(
            rect.left_bottom() - vec2(0.0, fill_height),
            vec2(bar_size.x, fill_height),
        );

        painter.rect_filled(fill_rect, 0.0, Color32::WHITE);
    }

    pub fn render_game_over_popup(&mut self, ctx: &Context) {
        if !self.show_game_over_popup {
            return;
        }

        let (title, subtitle) = match self.board.state {
            State::Checkmate { winner } => (
                "Checkmate",
                Some(match winner {
                    Color::White => "White Wins",
                    Color::Black => "Black Wins",
                }),
            ),
            State::Stalemate => ("Stalemate", None),
            State::Draw => ("Draw", None),
            _ => return,
        };

        Window::new("Game Over")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                    ui.label(
                        RichText::new(title)
                            .strong()
                            .color(Color32::WHITE)
                            .size(28.0),
                    );

                    if let Some(sub) = subtitle {
                        ui.add_space(4.0);
                        ui.label(RichText::new(sub).color(Color32::LIGHT_GRAY).size(20.0));
                    }

                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                            if ui.button("Reset").clicked() {
                                self.reset_game();
                                self.show_game_over_popup = false;
                            }
                        });

                        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                            if ui.button("Exit").clicked() {
                                std::process::exit(0);
                            }
                        });
                    });
                });
            });
    }
}
