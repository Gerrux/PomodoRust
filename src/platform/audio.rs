//! Audio playback for notifications

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::io::Cursor;

use crate::data::NotificationSound;

// Embed sound files at compile time
const SOUND_SOFT_BELL: &[u8] = include_bytes!("../../assets/soft_bell.mp3");
const SOUND_LEVEL_UP: &[u8] = include_bytes!("../../assets/level_up.mp3");
const SOUND_DIGITAL_ALERT: &[u8] = include_bytes!("../../assets/digital_alert.mp3");

/// Audio player for playing notification sounds
pub struct AudioPlayer {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    volume: f32,
}

impl AudioPlayer {
    /// Create a new audio player
    pub fn new() -> Option<Self> {
        match OutputStream::try_default() {
            Ok((stream, handle)) => Some(Self {
                _stream: stream,
                stream_handle: handle,
                volume: 0.8,
            }),
            Err(e) => {
                tracing::error!("Failed to initialize audio: {}", e);
                None
            }
        }
    }

    /// Set volume (0.0 to 1.0)
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Play the selected notification sound
    pub fn play_notification(&self, sound: NotificationSound) {
        let sound_data = match sound {
            NotificationSound::SoftBell => SOUND_SOFT_BELL,
            NotificationSound::LevelUp => SOUND_LEVEL_UP,
            NotificationSound::DigitalAlert => SOUND_DIGITAL_ALERT,
        };

        self.play_sound_data(sound_data);
    }

    /// Play raw sound data (mp3)
    fn play_sound_data(&self, data: &[u8]) {
        let cursor = Cursor::new(data.to_vec());

        match Decoder::new(cursor) {
            Ok(source) => {
                if let Ok(sink) = Sink::try_new(&self.stream_handle) {
                    sink.set_volume(self.volume);
                    sink.append(source);
                    sink.detach();
                }
            }
            Err(e) => {
                tracing::error!("Failed to decode sound: {}", e);
            }
        }
    }
}
