use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum VoiceNoteError {
    #[error("Audio duration exceeded maximum limit (15 minutes)")]
    DurationExceeded,
    #[error("Empty audio payload")]
    EmptyAudioPayload,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioCodec {
    Opus48kHz,
    AacLc,
}

/// WhatsApp-parity Push-to-Talk Voice Message Metadata & Waveform
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceNotePayload {
    pub note_id: String,
    pub duration_ms: u32,
    pub codec: AudioCodec,
    /// 50 normalized amplitude points between 0.05 and 1.0 for visual waveform rendering
    pub waveform_bars: Vec<f32>,
    /// BLAKE3 CID of the encrypted audio chunk stored in sovereign cluster
    pub audio_content_cid: String,
    pub sender_did: String,
    pub timestamp: u64,
}

impl VoiceNotePayload {
    pub const MAX_DURATION_MS: u32 = 15 * 60 * 1000; // 15 minutes max
    pub const STANDARD_WAVEFORM_BARS: usize = 50;

    /// Generates normalized 50-bar waveform samples from raw audio PCM amplitude levels
    pub fn compute_waveform_from_pcm(raw_amplitudes: &[i16]) -> Vec<f32> {
        if raw_amplitudes.is_empty() {
            return vec![0.05; Self::STANDARD_WAVEFORM_BARS];
        }

        let chunk_size = (raw_amplitudes.len() as f32 / Self::STANDARD_WAVEFORM_BARS as f32).ceil() as usize;
        let mut bars = Vec::with_capacity(Self::STANDARD_WAVEFORM_BARS);

        for chunk in raw_amplitudes.chunks(chunk_size.max(1)) {
            let max_val = chunk.iter().map(|&s| (s as f32).abs()).fold(0.0f32, f32::max);
            // Normalize against i16::MAX (32767.0) with a minimum floor of 0.05 for visual aesthetics
            let normalized = (max_val / 32767.0).clamp(0.05, 1.0);
            bars.push(normalized);
        }

        while bars.len() < Self::STANDARD_WAVEFORM_BARS {
            bars.push(0.05);
        }
        bars.truncate(Self::STANDARD_WAVEFORM_BARS);
        bars
    }

    pub fn new(
        note_id: &str,
        duration_ms: u32,
        codec: AudioCodec,
        raw_amplitudes: &[i16],
        audio_content_cid: &str,
        sender_did: &str,
        timestamp: u64,
    ) -> Result<Self, VoiceNoteError> {
        if duration_ms > Self::MAX_DURATION_MS {
            return Err(VoiceNoteError::DurationExceeded);
        }

        let waveform_bars = Self::compute_waveform_from_pcm(raw_amplitudes);

        Ok(Self {
            note_id: note_id.to_string(),
            duration_ms,
            codec,
            waveform_bars,
            audio_content_cid: audio_content_cid.to_string(),
            sender_did: sender_did.to_string(),
            timestamp,
        })
    }
}
