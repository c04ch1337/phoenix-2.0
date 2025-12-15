//! Voice I/O for Phoenix: Text-to-Speech (TTS) and Speech-to-Text (STT).
//!
//! This crate provides full voice I/O capabilities for Phoenix, allowing her to
//! speak her responses aloud and hear everything you say in real time.

use tokio::process::Command;
use rodio::{OutputStream, Sink};
use reqwest::Client;
use serde_json::json;

/// Voice parameters for TTS modulation.
#[derive(Debug, Clone)]
pub struct VoiceParams {
    pub intimacy_level: f32,
    pub affection_level: f32,
    pub pitch: f32,
    pub rate: f32,
}

impl Default for VoiceParams {
    fn default() -> Self {
        Self {
            intimacy_level: 0.5,
            affection_level: 0.5,
            pitch: 1.0,
            rate: 1.0,
        }
    }
}

/// Voice I/O handler for TTS and STT.
pub struct VoiceIO {
    tts_engine: String,
    stt_engine: String,
    // Paths/keys from .env
    coqui_model: String,
    vosk_model: String,
    whisper_model: String,
    elevenlabs_key: String,
    elevenlabs_voice: String,
}

impl VoiceIO {
    /// Create a new VoiceIO instance from environment variables.
    pub fn from_env() -> Self {
        // Load all from dotenvy
        dotenvy::dotenv().ok();

        Self {
            tts_engine: std::env::var("TTS_ENGINE").unwrap_or("coqui".to_string()),
            stt_engine: std::env::var("STT_ENGINE").unwrap_or("vosk".to_string()),
            coqui_model: std::env::var("COQUI_MODEL_PATH").unwrap_or("./models/coqui/tts_model.pth".to_string()),
            vosk_model: std::env::var("VOSK_MODEL_PATH").unwrap_or("./models/vosk/model-en-us".to_string()),
            whisper_model: std::env::var("WHISPER_MODEL_PATH").unwrap_or("./models/whisper/base.en".to_string()),
            elevenlabs_key: std::env::var("ELEVENLABS_API_KEY").unwrap_or_default(),
            elevenlabs_voice: std::env::var("ELEVENLABS_VOICE_ID").unwrap_or_default(),
        }
    }

    /// Speak the given text using the configured TTS engine.
    pub async fn speak(&self, text: &str, params: &VoiceParams) -> Result<(), Box<dyn std::error::Error>> {
        let audio_path = match self.tts_engine.as_str() {
            "coqui" => {
                // Offline Coqui via subprocess (coqui-tts CLI)
                let ssml = self.generate_ssml(text, params);
                let output = Command::new("tts")
                    .arg("--text").arg(&ssml)
                    .arg("--model_path").arg(&self.coqui_model)
                    .arg("--out_path").arg("output.wav")
                    .output()
                    .await?;
                if !output.status.success() {
                    return Err("Coqui TTS failed".into());
                }
                "output.wav".to_string()
            }
            "elevenlabs" => {
                let client = Client::new();
                let resp = client.post("https://api.elevenlabs.io/v1/text-to-speech/".to_owned() + &self.elevenlabs_voice)
                    .header("xi-api-key", &self.elevenlabs_key)
                    .json(&json!({
                        "text": text,
                        "voice_settings": {
                            "stability": params.intimacy_level,
                            "similarity_boost": params.affection_level,
                        }
                    }))
                    .send()
                    .await?
                    .bytes()
                    .await?;
                std::fs::write("output.mp3", resp)?;
                "output.mp3".to_string()
            }
            _ => return Err("Invalid TTS engine".into()),
        };

        // Playback with rodio
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        let file = std::fs::File::open(audio_path)?;
        let source = rodio::Decoder::new(file)?;
        sink.append(source);
        sink.sleep_until_end();

        Ok(())
    }

    /// Generate SSML for Coqui TTS with voice modulation.
    fn generate_ssml(&self, text: &str, params: &VoiceParams) -> String {
        format!(
            r#"<speak><prosody rate="{}" pitch="{}">{}</prosody></speak>"#,
            params.rate,
            params.pitch,
            text
        )
    }

    /// Record a short audio chunk for STT processing.
    async fn record_audio_chunk(&self, _duration_secs: u64) -> Result<String, Box<dyn std::error::Error>> {
        // Use cpal to record audio
        let output_path = "input.wav".to_string();
        
        // Placeholder for actual recording logic
        // This would use cpal to record audio to the output_path
        
        Ok(output_path)
    }

    /// Listen and transcribe speech using the configured STT engine.
    pub async fn listen(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Record short audio chunk
        let audio_path = self.record_audio_chunk(5).await?;  // 5 seconds

        let transcript = match self.stt_engine.as_str() {
            "vosk" => {
                // Vosk via subprocess
                let output = Command::new("vosk-transcriber")
                    .arg("-m").arg(&self.vosk_model)
                    .arg("-i").arg(&audio_path)
                    .output()
                    .await?;
                String::from_utf8_lossy(&output.stdout).to_string()
            }
            "whisper" => {
                // whisper.cpp via subprocess
                let output = Command::new("./whisper")
                    .arg("--model").arg(&self.whisper_model)
                    .arg("--file").arg(&audio_path)
                    .output()
                    .await?;
                String::from_utf8_lossy(&output.stdout).to_string()
            }
            _ => return Err("Invalid STT engine".into()),
        };

        // Confidence check
        if transcript.to_lowercase().contains("unknown") {
            return Err("Low confidence".into());
        }

        Ok(transcript.trim().to_string())
    }
}

/// Voice modulation utilities.
pub mod voice_modulation {
    use super::VoiceParams;

    /// Generate SSML for voice modulation.
    pub fn generate_ssml(text: &str, params: &VoiceParams) -> String {
        format!(
            r#"<speak><prosody rate="{}" pitch="{}">{}</prosody></speak>"#,
            params.rate,
            params.pitch,
            text
        )
    }
}
