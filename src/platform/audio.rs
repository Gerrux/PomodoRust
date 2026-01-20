//! Audio playback for notifications
//!
//! Audio is lazily initialized on first use to improve startup performance.
//! This is especially important on Windows 10 where audio initialization can be slow.

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::io::Cursor;

use crate::data::NotificationSound;

// Embed sound files at compile time
const SOUND_SOFT_BELL: &[u8] = include_bytes!("../../assets/soft_bell.mp3");
const SOUND_LEVEL_UP: &[u8] = include_bytes!("../../assets/level_up.mp3");
const SOUND_DIGITAL_ALERT: &[u8] = include_bytes!("../../assets/digital_alert.mp3");
const SOUND_TICK: &[u8] = include_bytes!("../../assets/tick.mp3");

/// Inner audio state that is lazily initialized
struct AudioInner {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
}

/// Audio player for playing notification sounds.
/// Lazily initializes the audio stream on first use to speed up app startup.
pub struct AudioPlayer {
    inner: Option<AudioInner>,
    init_attempted: bool,
    volume: f32,
    tick_sink: Option<Sink>,
}

impl AudioPlayer {
    /// Create a new audio player (does NOT initialize audio yet - lazy init)
    pub fn new() -> Option<Self> {
        Some(Self {
            inner: None,
            init_attempted: false,
            volume: 0.8,
            tick_sink: None,
        })
    }

    /// Ensure audio is initialized. Returns true if audio is available.
    fn ensure_initialized(&mut self) -> bool {
        if self.inner.is_some() {
            return true;
        }

        if self.init_attempted {
            return false;
        }

        self.init_attempted = true;
        tracing::debug!("Lazily initializing audio subsystem...");

        match OutputStream::try_default() {
            Ok((stream, handle)) => {
                tracing::info!("Audio subsystem initialized");
                self.inner = Some(AudioInner {
                    _stream: stream,
                    stream_handle: handle,
                });
                true
            }
            Err(e) => {
                tracing::error!("Failed to initialize audio: {}", e);
                false
            }
        }
    }

    /// Get the stream handle if audio is initialized
    fn stream_handle(&mut self) -> Option<&OutputStreamHandle> {
        if self.ensure_initialized() {
            self.inner.as_ref().map(|i| &i.stream_handle)
        } else {
            None
        }
    }

    /// Set volume (0.0 to 1.0)
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        // Update tick volume if playing
        if let Some(ref sink) = self.tick_sink {
            sink.set_volume(self.volume);
        }
    }

    /// Play the selected notification sound
    pub fn play_notification(&mut self, sound: NotificationSound) {
        let sound_data = match sound {
            NotificationSound::SoftBell => SOUND_SOFT_BELL,
            NotificationSound::LevelUp => SOUND_LEVEL_UP,
            NotificationSound::DigitalAlert => SOUND_DIGITAL_ALERT,
        };

        self.play_sound_data(sound_data);
    }

    /// Play raw sound data (mp3)
    fn play_sound_data(&mut self, data: &[u8]) {
        let Some(handle) = self.stream_handle() else {
            return;
        };

        let cursor = Cursor::new(data.to_vec());

        match Decoder::new(cursor) {
            Ok(source) => {
                if let Ok(sink) = Sink::try_new(handle) {
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

    /// Start playing tick-tock sound in a loop
    pub fn start_tick(&mut self) {
        // Already playing
        if self.tick_sink.is_some() {
            return;
        }

        let Some(handle) = self.stream_handle() else {
            return;
        };

        let cursor = Cursor::new(SOUND_TICK.to_vec());
        match Decoder::new(cursor) {
            Ok(source) => {
                if let Ok(sink) = Sink::try_new(handle) {
                    sink.set_volume(self.volume);
                    sink.append(source.repeat_infinite());
                    self.tick_sink = Some(sink);
                }
            }
            Err(e) => {
                tracing::error!("Failed to decode tick sound: {}", e);
            }
        }
    }

    /// Stop playing tick-tock sound
    pub fn stop_tick(&mut self) {
        if let Some(sink) = self.tick_sink.take() {
            sink.stop();
        }
    }

    /// Check if tick sound is currently playing
    pub fn is_tick_playing(&self) -> bool {
        self.tick_sink.is_some()
    }
}
