use crate::gui::launch::Cactus;

use std::io::Cursor;

use rodio::{OutputStream, OutputStreamBuilder};

pub static MOVE_SOUND: &[u8] = include_bytes!("../../assets/standard/Move.mp3");
pub static CAPTURE_SOUND: &[u8] = include_bytes!("../../assets/standard/Capture.mp3");
pub static CHECK_SOUND: &[u8] = include_bytes!("../../assets/standard/Check.mp3");

impl Cactus {
    fn play(handle: &OutputStream, bytes: &'static [u8]) {
        let mixer = handle.mixer();
        let sink = rodio::play(mixer, Cursor::new(bytes)).expect("Failed to play audio");
        sink.detach();
    }

    pub fn move_sound(&self) {
        if let Some(handle) = &self.audio_stream {
            Self::play(handle, MOVE_SOUND);
        }
    }

    pub fn capture_sound(&self) {
        if let Some(handle) = &self.audio_stream {
            Self::play(handle, CAPTURE_SOUND);
        }
    }

    pub fn check_sound(&self) {
        if let Some(handle) = &self.audio_stream {
            Self::play(handle, CHECK_SOUND);
        }
    }
}
