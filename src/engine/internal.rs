use crate::engine::EngineHandle;

use std::sync::mpsc::{Receiver, Sender};
use std::thread;

pub struct CactusEngine {}

impl CactusEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&mut self, cmd_receiver: Receiver<String>, response_sender: Sender<String>) {
        for cmd in cmd_receiver.iter() {
            if cmd == "uci" {
                let _ = response_sender.send("id name CactusEngine".to_string());
                let _ = response_sender.send("id author YourName".to_string());
                let _ = response_sender.send("uciok".to_string());
            } else if cmd == "isready" {
                let _ = response_sender.send("readyok".to_string());
            } else if cmd.starts_with("position") {
            } else if cmd.starts_with("go") {
                let best_move = "e2e4".to_string();
                let _ = response_sender.send(format!("bestmove {}", best_move));
            } else if cmd == "quit" {
                break;
            }
        }
    }
}

pub fn spawn_cactus_engine() -> EngineHandle {
    let (cmd_sender, cmd_receiver) = std::sync::mpsc::channel::<String>();
    let (response_sender, response_receiver) = std::sync::mpsc::channel::<String>();

    thread::spawn(move || {
        let mut engine = CactusEngine::new();
        engine.run(cmd_receiver, response_sender);
    });

    EngineHandle {
        cmd_sender,
        response_receiver,
    }
}
