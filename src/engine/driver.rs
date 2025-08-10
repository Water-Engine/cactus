use crate::coupling::EngineHandle;
use crate::engine::utils;

use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::{env, io, thread};

use directories::BaseDirs;

pub struct CactusEngine {
    thinking: bool,
    log: Option<File>,
}

struct StdoutSender;

pub trait SenderLike {
    fn send(&self, msg: String);
}

impl SenderLike for Sender<String> {
    fn send(&self, msg: String) {
        let _ = self.send(msg);
    }
}

impl SenderLike for StdoutSender {
    fn send(&self, msg: String) {
        println!("{}", msg);
        let _ = io::stdout().flush();
    }
}

impl CactusEngine {
    /// For use with the games gui with thread safe logic
    pub fn spawn_threaded() -> EngineHandle {
        let (cmd_sender, cmd_receiver) = std::sync::mpsc::channel::<String>();
        let (response_sender, response_receiver) = std::sync::mpsc::channel::<String>();

        thread::spawn(move || {
            let mut engine = CactusEngine::default();
            engine.run(cmd_receiver, response_sender);
        });

        EngineHandle {
            cmd_sender,
            response_receiver,
        }
    }

    fn run<S: SenderLike>(&mut self, cmd_receiver: Receiver<String>, response_sender: S) {
        for cmd in cmd_receiver.iter() {
            self.handle_cmd(&cmd, &response_sender);
            if cmd == "quit" {
                break;
            }
        }
    }

    /// For use in non-gui environments. Simply a command line engine
    pub fn start() {
        let stdin = io::stdin();
        let mut engine = CactusEngine::default();

        for line in stdin.lock().lines() {
            match line {
                Ok(cmd) => {
                    engine.handle_cmd(&cmd, &StdoutSender);
                    if cmd == "quit" {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    }

    fn handle_cmd<S: SenderLike>(&mut self, cmd: &str, sender: &S) {
        self.log(format!("Received Command: {}\n", cmd));

        let cmd = cmd.trim().to_lowercase();
        let mut parts: VecDeque<&str> = cmd.split_whitespace().collect();
        let cmd_lead = parts.pop_front();
        let cmd_args: Vec<String> = parts.iter().map(|s| s.to_string()).collect();

        match cmd_lead {
            Some("uci") => {
                sender.send("id name CactusEngine".to_string());
                sender.send("id author Trevor Swan".to_string());
                sender.send("uciok".to_string());
            }
            Some("isready") => sender.send("readyok".to_string()),
            Some("ucinewgame") => {}
            Some("position") => match utils::parser::position(cmd_args) {
                Ok(moves) => {
                    dbg!(moves);
                }
                Err(msg) => self.log(msg),
            },
            Some("go") => {}
            Some("stop") => {
                self.stop_thinking();
            }
            Some("d") => {}
            Some("quit") => {}
            _ => self.log(format!("Unknown Command: {}\n", cmd)),
        }
    }

    pub fn log(&mut self, msg: String) {
        let Some(log) = &mut self.log else {
            return;
        };
        let _ = log.write(msg.as_bytes());
    }

    pub fn stop_thinking(&mut self) {
        self.thinking = false;
    }
}

impl Default for CactusEngine {
    fn default() -> Self {
        let log_path: PathBuf = if let Some(base_dirs) = BaseDirs::new() {
            let cactus_dir = base_dirs.data_local_dir().join("Cactus");
            let _ = fs::create_dir_all(&cactus_dir);
            cactus_dir.join("cactus.log")
        } else if let Ok(current_dir) = env::current_dir() {
            current_dir.join("cactus.log")
        } else {
            PathBuf::from("cactus.log")
        };

        Self {
            thinking: false,
            log: File::create(log_path).ok(),
        }
    }
}
