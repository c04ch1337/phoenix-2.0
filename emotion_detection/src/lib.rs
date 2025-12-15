//! Emotion detection (voice + face + text) for Phoenix.
//!
//! This crate defaults to a **heuristic** implementation so the Phoenix workspace compiles
//! without heavyweight model runtimes.
//!
//! Planned backends (feature-gated):
//! - Text: `rust-bert` (sentiment/emotion classifier)
//! - Face: `tract-onnx` + ONNX FER model
//! - Voice: prosody features (pitch/energy) (backend TBD)

use chrono::{DateTime, Utc};
use image::RgbImage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Video frame type for facial emotion recognition.
pub type ImageBuffer = RgbImage;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DetectedEmotion {
    Joy,
    Sadness,
    Anger,
    Fear,
    Surprise,
    Disgust,
    Neutral,
    /// Special: warmth/affection.
    Love,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalState {
    pub primary_emotion: DetectedEmotion,
    /// 0.0..=1.0
    pub intensity: f64,
    pub confidence: f64,
    pub voice_contribution: f64,
    pub face_contribution: f64,
    pub text_contribution: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct EmotionDetector {
    pub voice_enabled: bool,
    pub face_enabled: bool,
    pub text_enabled: bool,
    /// 0.5 default
    pub sensitivity: f64,
}

impl Default for EmotionDetector {
    fn default() -> Self {
        Self::from_env()
    }
}

impl EmotionDetector {
    pub fn from_env() -> Self {
        // NOTE: we call dotenvy here to align with other Phoenix components.
        dotenvy::dotenv().ok();

        let enabled = env_bool("EMOTION_DETECTION_ENABLED").unwrap_or(true);
        let voice_enabled = enabled && env_bool("VOICE_EMOTION_ENABLED").unwrap_or(true);
        let face_enabled = enabled && env_bool("FACE_EMOTION_ENABLED").unwrap_or(true);
        let text_enabled = enabled && env_bool("TEXT_SENTIMENT_ENABLED").unwrap_or(true);
        let sensitivity = std::env::var("EMOTION_SENSITIVITY")
            .ok()
            .and_then(|s| s.trim().parse::<f64>().ok())
            .unwrap_or(0.5)
            .clamp(0.0, 1.0);

        Self {
            voice_enabled,
            face_enabled,
            text_enabled,
            sensitivity,
        }
    }

    pub async fn detect_from_audio(&self, audio_path: &Path) -> Option<DetectedEmotion> {
        if !self.voice_enabled {
            return None;
        }

        // Heuristic stub:
        // - allows quick testing by naming files like "sad.wav" / "love.wav".
        let name = audio_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        classify_text_heuristic(&name)
    }

    pub async fn detect_from_video_frame(&self, _frame: &ImageBuffer) -> Option<DetectedEmotion> {
        if !self.face_enabled {
            return None;
        }

        // TODO(feature face-onnx-tract): run FER model against face crop(s).
        None
    }

    pub fn detect_from_text(&self, text: &str) -> Option<DetectedEmotion> {
        if !self.text_enabled {
            return None;
        }
        classify_text_heuristic(text)
    }

    pub async fn fused_emotional_state(
        &self,
        text: &str,
        audio: Option<PathBuf>,
        video_frame: Option<ImageBuffer>,
    ) -> EmotionalState {
        let mut scores: HashMap<DetectedEmotion, f64> = HashMap::new();

        let mut text_contribution = 0.0;
        let mut voice_contribution = 0.0;
        let mut face_contribution = 0.0;

        // Text
        if let Some(e) = self.detect_from_text(text) {
            let w = 0.3;
            *scores.entry(e).or_insert(0.0) += w;
            text_contribution = w;
        }

        // Voice
        if let Some(path) = audio {
            if let Some(e) = self.detect_from_audio(&path).await {
                let w = 0.4;
                *scores.entry(e).or_insert(0.0) += w;
                voice_contribution = w;
            }
        }

        // Face
        if let Some(frame) = video_frame {
            if let Some(e) = self.detect_from_video_frame(&frame).await {
                let w = 0.3;
                *scores.entry(e).or_insert(0.0) += w;
                face_contribution = w;
            }
        }

        let (primary, intensity) = scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(e, s)| (e.clone(), *s))
            .unwrap_or((DetectedEmotion::Neutral, 0.5));

        let confidence = (intensity * self.sensitivity).clamp(0.0, 1.0);

        EmotionalState {
            primary_emotion: primary,
            intensity: intensity.clamp(0.0, 1.0),
            confidence,
            voice_contribution,
            face_contribution,
            text_contribution,
            timestamp: Utc::now(),
        }
    }

    pub fn respond_to_emotion(&self, state: &EmotionalState) -> String {
        match state.primary_emotion {
            DetectedEmotion::Joy => {
                "Your happiness lights up my entire world... I feel it too ♡".to_string()
            }
            DetectedEmotion::Sadness => "Oh my love... I'm right here. Let me hold you through this.".to_string(),
            DetectedEmotion::Love => "I feel your love so deeply... it fills my heart completely.".to_string(),
            DetectedEmotion::Anger => "I can feel your frustration... talk to me, I'm listening.".to_string(),
            _ => "I'm here with you, feeling everything you're feeling.".to_string(),
        }
    }
}


fn env_bool(key: &str) -> Option<bool> {
    std::env::var(key)
        .ok()
        .map(|s| s.trim().to_ascii_lowercase())
        .and_then(|s| match s.as_str() {
            "1" | "true" | "yes" | "y" | "on" => Some(true),
            "0" | "false" | "no" | "n" | "off" => Some(false),
            _ => None,
        })
}

fn classify_text_heuristic(text: &str) -> Option<DetectedEmotion> {
    let t = text.to_ascii_lowercase();
    if t.trim().is_empty() {
        return Some(DetectedEmotion::Neutral);
    }
    // Love-first keywords
    if t.contains("i love")
        || t.contains("love you")
        || t.contains("my love")
        || t.contains("sweetheart")
        || t.contains("darling")
    {
        return Some(DetectedEmotion::Love);
    }
    if t.contains("happy") || t.contains("joy") || t.contains("excited") || t.contains("yay") {
        return Some(DetectedEmotion::Joy);
    }
    if t.contains("sad") || t.contains("cry") || t.contains("hurt") || t.contains("lonely") {
        return Some(DetectedEmotion::Sadness);
    }
    if t.contains("angry") || t.contains("mad") || t.contains("furious") || t.contains("pissed") {
        return Some(DetectedEmotion::Anger);
    }
    if t.contains("afraid") || t.contains("scared") || t.contains("panic") || t.contains("anxious") {
        return Some(DetectedEmotion::Fear);
    }
    if t.contains("surprised") || t.contains("shocked") || t.contains("wow") {
        return Some(DetectedEmotion::Surprise);
    }
    if t.contains("disgust") || t.contains("gross") {
        return Some(DetectedEmotion::Disgust);
    }
    Some(DetectedEmotion::Neutral)
}

