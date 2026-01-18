//! Audio playback for notifications

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::io::Cursor;

/// Built-in sound types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundType {
    WorkEnd,
    BreakEnd,
    Tick,
    Notification,
}

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

    /// Play a built-in sound
    pub fn play(&self, sound_type: SoundType) {
        let sound_data = match sound_type {
            SoundType::WorkEnd => Self::generate_work_end_sound(),
            SoundType::BreakEnd => Self::generate_break_end_sound(),
            SoundType::Tick => Self::generate_tick_sound(),
            SoundType::Notification => Self::generate_notification_sound(),
        };

        self.play_wav_data(&sound_data);
    }

    /// Play raw WAV data
    fn play_wav_data(&self, data: &[u8]) {
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

    /// Generate a simple bell/chime sound (WAV format)
    fn generate_work_end_sound() -> Vec<u8> {
        Self::generate_tone_wav(880.0, 0.3, 0.8) // A5, 300ms
    }

    fn generate_break_end_sound() -> Vec<u8> {
        Self::generate_tone_wav(659.25, 0.3, 0.8) // E5, 300ms
    }

    fn generate_tick_sound() -> Vec<u8> {
        Self::generate_tone_wav(1000.0, 0.05, 0.3) // 1kHz, 50ms, quiet
    }

    fn generate_notification_sound() -> Vec<u8> {
        Self::generate_tone_wav(523.25, 0.2, 0.7) // C5, 200ms
    }

    /// Generate a simple sine wave tone as WAV data
    fn generate_tone_wav(frequency: f32, duration: f32, amplitude: f32) -> Vec<u8> {
        let sample_rate = 44100u32;
        let num_samples = (sample_rate as f32 * duration) as usize;

        let mut samples = Vec::with_capacity(num_samples);

        for i in 0..num_samples {
            let t = i as f32 / sample_rate as f32;

            // Simple sine wave with fade in/out
            let fade_samples = (sample_rate as f32 * 0.02) as usize; // 20ms fade
            let fade = if i < fade_samples {
                i as f32 / fade_samples as f32
            } else if i > num_samples - fade_samples {
                (num_samples - i) as f32 / fade_samples as f32
            } else {
                1.0
            };

            let sample = (t * frequency * 2.0 * std::f32::consts::PI).sin() * amplitude * fade;
            let sample_i16 = (sample * 32767.0) as i16;
            samples.push(sample_i16);
        }

        Self::samples_to_wav(&samples, sample_rate)
    }

    /// Convert i16 samples to WAV format
    fn samples_to_wav(samples: &[i16], sample_rate: u32) -> Vec<u8> {
        let num_channels = 1u16;
        let bits_per_sample = 16u16;
        let byte_rate = sample_rate * num_channels as u32 * bits_per_sample as u32 / 8;
        let block_align = num_channels * bits_per_sample / 8;
        let data_size = (samples.len() * 2) as u32;
        let file_size = 36 + data_size;

        let mut wav = Vec::with_capacity(44 + data_size as usize);

        // RIFF header
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&file_size.to_le_bytes());
        wav.extend_from_slice(b"WAVE");

        // fmt chunk
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes()); // Chunk size
        wav.extend_from_slice(&1u16.to_le_bytes()); // Audio format (PCM)
        wav.extend_from_slice(&num_channels.to_le_bytes());
        wav.extend_from_slice(&sample_rate.to_le_bytes());
        wav.extend_from_slice(&byte_rate.to_le_bytes());
        wav.extend_from_slice(&block_align.to_le_bytes());
        wav.extend_from_slice(&bits_per_sample.to_le_bytes());

        // data chunk
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_size.to_le_bytes());

        for sample in samples {
            wav.extend_from_slice(&sample.to_le_bytes());
        }

        wav
    }
}
