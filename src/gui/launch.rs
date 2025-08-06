use crate::core::{board::*, piece::*};

use eframe::egui::{self, Context, Painter, Pos2, Vec2};
use eframe::{App, Frame};
use rodio::{OutputStream, OutputStreamBuilder};

pub struct Cactus {
    pub board: Board,
    pub images: PieceImages,
    pub board_size: Vec2,
    pub dragging: Option<(PieceKind, usize, usize)>,
    pub drag_pos: Pos2,
    pub selected: Option<(usize, usize)>,
    pub clear_selection: bool,
    pub painter: Option<Painter>,
    pub size: Vec2,
    pub audio_stream: Option<OutputStream>,
}

impl Cactus {
    pub fn new(ctx: &egui::Context) -> Self {
        let handle = OutputStreamBuilder::open_default_stream()
    .expect("Failed to initialize audio");
        Self {
            board: Board::default(),
            images: PieceImages::new(ctx, 64.0),
            board_size: Vec2::splat(400.0),
            dragging: None,
            drag_pos: Pos2::default(),
            selected: None,
            clear_selection: false,
            painter: None,
            size: Vec2::default(),
            audio_stream: Some(handle)
        }
    }
}

impl App for Cactus {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let response = self.handle_event(ctx, frame, ui);
            self.render(&response);
        });
    }
}

pub fn launch() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Chess Board",
        options,
        Box::new(|cc| Ok(Box::new(Cactus::new(&cc.egui_ctx)))),
    )
    .expect("Failed to launch Cactus")
}
