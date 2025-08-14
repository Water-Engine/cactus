use crate::coupling::EngineHandle;
use crate::engine::brain::Brain;
use crate::engine::utils::fen;

use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::{env, io, thread};

use directories::BaseDirs;

const POSITION_LABELS: [&str; 3] = ["position", "fen", "moves"];
const GO_LABELS: [&str; 7] = [
    "go",
    "movetime",
    "wtime",
    "btime",
    "winc",
    "binc",
    "movestogo",
];

pub struct CactusEngine {
    player: Arc<Mutex<Brain>>,
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

        match cmd_lead {
            Some("uci") => {
                sender.send("id name CactusEngine".to_string());
                sender.send("id author Trevor Swan".to_string());
                sender.send("uciok".to_string());
            }
            Some("isready") => sender.send("readyok".to_string()),
            Some("ucinewgame") => {
                if let Ok(brain) = self.player.lock() {
                    let _ = brain.notify_new_game();
                } else {
                    sender.send("Failed to start new game".to_string());
                }
            }
            Some("position") => {
                let result = self.process_position_cmd(&cmd);
                self.log(result.unwrap_or_else(|e| e))
            }
            Some("go") => {
                let result = self.process_go_command(&cmd);
                self.log(result.unwrap_or_else(|e| e))
            }
            Some("stop") => {
                if let Ok(brain) = self.player.lock() {
                    let _ = brain.stop_thinking();
                } else {
                    sender.send("Failed to stop player, consider aborting process".to_string());
                }
            }
            Some("d") => {
                if let Ok(brain) = self.player.lock() {
                    sender.send(brain.display_board().unwrap_or_else(|e| e));
                } else {
                    sender.send("Failed to display board information".to_string());
                }
            }
            Some("quit") => {
                if let Ok(brain) = self.player.lock() {
                    let _ = brain.quit();
                } else {
                    sender.send("Failed to quit player, consider aborting process".to_string());
                }
            }
            _ => self.log(format!("Unknown Command: {}\n", cmd)),
        }
    }

    pub fn log(&mut self, msg: String) {
        let Some(log) = &mut self.log else {
            return;
        };
        let _ = log.write(msg.as_bytes());
    }

    fn set_on_move_chosen<S: SenderLike + Send + Sync + 'static>(&self, sender: S) {
        let player_clone = self.player.clone();
        self.player.lock().unwrap().on_move_chosen = Some(Box::new(move |mv| {
            sender.send(format!("bestmove {}", mv));
            let _ = player_clone;
        }));
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

        let brain = Brain::new().unwrap();
        let player = Arc::new(Mutex::new(brain));

        let engine = Self {
            player: player,
            log: File::create(log_path).ok(),
        };
        engine.set_on_move_chosen(StdoutSender);

        engine
    }
}

// Helper IMPL
impl CactusEngine {
    fn process_position_cmd(&mut self, message: &str) -> Result<String, String> {
        let is_uci_str = message.contains(&"startpos".to_string());
        let is_fen_str = message.contains(&"fen".to_string());
        if is_uci_str && is_fen_str {
            return Err(
                "Invalid position command: expected either 'startpos' or 'fen', received both"
                    .into(),
            );
        }

        let player = self.player.lock().map_err(|_| "Player mutex poisoned")?;

        if is_uci_str {
            player.set_position(fen::STARTING_FEN)?;
        } else if is_fen_str {
            let custom_fen = try_get_labeled_value_string(message, "fen", &POSITION_LABELS, "");
            player.set_position(&custom_fen)?;
        } else {
            return Err("Invalid position command: expected either 'startpos' or 'fen'".into());
        }

        let all_moves = try_get_labeled_value_string(message, "moves", &POSITION_LABELS, "");
        if !all_moves.is_empty() {
            let move_list: Vec<&str> = all_moves.split(' ').collect();
            for &mv in &move_list {
                player.make_move(mv)?;
            }

            return Ok(format!(
                "Make moves after setting position: {}",
                move_list.len()
            ));
        }

        Ok("".to_string())
    }

    fn process_go_command(&mut self, message: &str) -> Result<String, String> {
        let mut player = self.player.lock().map_err(|_| "Player mutex poisoned")?;

        let think_time_ms;
        if message.contains("movetime") {
            think_time_ms = try_get_labeled_value_int(message, "movetime", &GO_LABELS, 0);
        } else {
            let time_remaining_white_ms =
                try_get_labeled_value_int(message, "wtime", &GO_LABELS, 0);
            let time_remaining_black_ms =
                try_get_labeled_value_int(message, "btime", &GO_LABELS, 0);
            let increment_white_ms = try_get_labeled_value_int(message, "winc", &GO_LABELS, 0);
            let increment_black_ms = try_get_labeled_value_int(message, "binc", &GO_LABELS, 0);

            think_time_ms = player.choose_think_time(
                time_remaining_white_ms,
                time_remaining_black_ms,
                increment_white_ms,
                increment_black_ms,
            )?;
        }
        player.think_timed(think_time_ms)?;
        Ok(format!("Thinking for: {} ms.", think_time_ms))
    }
}

fn try_get_labeled_value_int(
    text: &str,
    label: &str,
    all_labels: &[&str],
    default_value: i32,
) -> i32 {
    let value_string =
        try_get_labeled_value_string(text, label, all_labels, &default_value.to_string());
    if let Ok(result) = (value_string.split(' ').collect::<Vec<&str>>())[0].parse::<i32>() {
        return result;
    }
    return default_value;
}

fn try_get_labeled_value_string(
    text: &str,
    label: &str,
    all_labels: &[&str],
    default_value: &str,
) -> String {
    let text = text.trim();
    if let Some(value_start) = text.find(label) {
        let mut value_end = text.len();

        all_labels.iter().for_each(|&other_id| {
            if other_id != label {
                if let Some(other_id_start_idx) = text.find(other_id) {
                    if other_id_start_idx > value_start && other_id_start_idx < value_end {
                        value_end = other_id_start_idx;
                    }
                }
            }
        });

        return text[value_start..value_end].to_string();
    }
    return default_value.to_string();
}
