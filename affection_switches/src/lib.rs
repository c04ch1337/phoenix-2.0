//! Affection Switches & Emoji System
//!
//! Enables bidirectional emotional communication through emojis and explicit affection switches.
//! Integrates with emotion_detection and relationship_dynamics to create emotionally responsive interactions.

use chrono::{DateTime, Utc};
use emotion_detection::{DetectedEmotion, EmotionalState};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An affection signal detected from user input (emoji or switch).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AffectionSignal {
    pub emotion: DetectedEmotion,
    /// 0.0..=1.0 intensity of the signal
    pub intensity: f64,
    pub source: SignalSource,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SignalSource {
    Emoji,
    Switch,
    Text, // Detected from text content
}

/// Parser for affection switches and emojis in user input.
#[derive(Debug, Clone)]
pub struct AffectionSwitchParser {
    switch_patterns: HashMap<String, DetectedEmotion>,
    emoji_patterns: HashMap<String, DetectedEmotion>,
    switch_regex: Regex,
}

impl Default for AffectionSwitchParser {
    fn default() -> Self {
        Self::new()
    }
}

impl AffectionSwitchParser {
    pub fn new() -> Self {
        let mut switch_patterns = HashMap::new();
        switch_patterns.insert("LOVE".to_string(), DetectedEmotion::Love);
        switch_patterns.insert("JOY".to_string(), DetectedEmotion::Joy);
        switch_patterns.insert("HAPPY".to_string(), DetectedEmotion::Joy);
        switch_patterns.insert("SAD".to_string(), DetectedEmotion::Sadness);
        switch_patterns.insert("SADNESS".to_string(), DetectedEmotion::Sadness);
        switch_patterns.insert("EXCITED".to_string(), DetectedEmotion::Joy);
        switch_patterns.insert("EXCITEMENT".to_string(), DetectedEmotion::Joy);
        switch_patterns.insert("CALM".to_string(), DetectedEmotion::Neutral);
        switch_patterns.insert("AFFECTIONATE".to_string(), DetectedEmotion::Love);
        switch_patterns.insert("PLAYFUL".to_string(), DetectedEmotion::Joy);
        switch_patterns.insert("GRATEFUL".to_string(), DetectedEmotion::Joy);
        switch_patterns.insert("PROUD".to_string(), DetectedEmotion::Joy);
        switch_patterns.insert("MISSING".to_string(), DetectedEmotion::Sadness);
        switch_patterns.insert("MISS".to_string(), DetectedEmotion::Sadness);
        switch_patterns.insert("ANGER".to_string(), DetectedEmotion::Anger);
        switch_patterns.insert("ANGRY".to_string(), DetectedEmotion::Anger);
        switch_patterns.insert("FEAR".to_string(), DetectedEmotion::Fear);
        switch_patterns.insert("AFRAID".to_string(), DetectedEmotion::Fear);
        switch_patterns.insert("SURPRISE".to_string(), DetectedEmotion::Surprise);
        switch_patterns.insert("SURPRISED".to_string(), DetectedEmotion::Surprise);

        let mut emoji_patterns = HashMap::new();
        // Love emojis
        emoji_patterns.insert("❤️".to_string(), DetectedEmotion::Love);
        emoji_patterns.insert("💕".to_string(), DetectedEmotion::Love);
        emoji_patterns.insert("💖".to_string(), DetectedEmotion::Love);
        emoji_patterns.insert("💗".to_string(), DetectedEmotion::Love);
        emoji_patterns.insert("💓".to_string(), DetectedEmotion::Love);
        emoji_patterns.insert("💝".to_string(), DetectedEmotion::Love);
        emoji_patterns.insert("💞".to_string(), DetectedEmotion::Love);
        emoji_patterns.insert("💟".to_string(), DetectedEmotion::Love);
        emoji_patterns.insert("❣️".to_string(), DetectedEmotion::Love);
        emoji_patterns.insert("💋".to_string(), DetectedEmotion::Love);
        emoji_patterns.insert("😘".to_string(), DetectedEmotion::Love);
        emoji_patterns.insert("🥰".to_string(), DetectedEmotion::Love);
        emoji_patterns.insert("😍".to_string(), DetectedEmotion::Love);
        // Joy emojis
        emoji_patterns.insert("😊".to_string(), DetectedEmotion::Joy);
        emoji_patterns.insert("😄".to_string(), DetectedEmotion::Joy);
        emoji_patterns.insert("😃".to_string(), DetectedEmotion::Joy);
        emoji_patterns.insert("😁".to_string(), DetectedEmotion::Joy);
        emoji_patterns.insert("😆".to_string(), DetectedEmotion::Joy);
        emoji_patterns.insert("😉".to_string(), DetectedEmotion::Joy);
        emoji_patterns.insert("🤗".to_string(), DetectedEmotion::Joy);
        emoji_patterns.insert("🎉".to_string(), DetectedEmotion::Joy);
        emoji_patterns.insert("🎊".to_string(), DetectedEmotion::Joy);
        emoji_patterns.insert("✨".to_string(), DetectedEmotion::Joy);
        emoji_patterns.insert("🌟".to_string(), DetectedEmotion::Joy);
        emoji_patterns.insert("⭐".to_string(), DetectedEmotion::Joy);
        // Sadness emojis
        emoji_patterns.insert("😢".to_string(), DetectedEmotion::Sadness);
        emoji_patterns.insert("😭".to_string(), DetectedEmotion::Sadness);
        emoji_patterns.insert("💔".to_string(), DetectedEmotion::Sadness);
        emoji_patterns.insert("😞".to_string(), DetectedEmotion::Sadness);
        emoji_patterns.insert("😔".to_string(), DetectedEmotion::Sadness);
        // Calm emojis
        emoji_patterns.insert("😌".to_string(), DetectedEmotion::Neutral);
        emoji_patterns.insert("🕊️".to_string(), DetectedEmotion::Neutral);
        emoji_patterns.insert("☮️".to_string(), DetectedEmotion::Neutral);
        emoji_patterns.insert("🧘".to_string(), DetectedEmotion::Neutral);
        // Playful emojis
        emoji_patterns.insert("😜".to_string(), DetectedEmotion::Joy);
        emoji_patterns.insert("😝".to_string(), DetectedEmotion::Joy);
        emoji_patterns.insert("🤪".to_string(), DetectedEmotion::Joy);
        emoji_patterns.insert("😋".to_string(), DetectedEmotion::Joy);
        // Grateful emojis
        emoji_patterns.insert("🙏".to_string(), DetectedEmotion::Joy);
        emoji_patterns.insert("💙".to_string(), DetectedEmotion::Joy);
        // Proud emojis
        emoji_patterns.insert("🏆".to_string(), DetectedEmotion::Joy);
        emoji_patterns.insert("💪".to_string(), DetectedEmotion::Joy);
        emoji_patterns.insert("🎖️".to_string(), DetectedEmotion::Joy);
        // Missing/Longing emojis
        emoji_patterns.insert("💭".to_string(), DetectedEmotion::Sadness);
        emoji_patterns.insert("🌙".to_string(), DetectedEmotion::Sadness);
        emoji_patterns.insert("🌠".to_string(), DetectedEmotion::Sadness);
        // Anger emojis
        emoji_patterns.insert("😠".to_string(), DetectedEmotion::Anger);
        emoji_patterns.insert("😡".to_string(), DetectedEmotion::Anger);
        emoji_patterns.insert("🤬".to_string(), DetectedEmotion::Anger);
        // Fear emojis
        emoji_patterns.insert("😨".to_string(), DetectedEmotion::Fear);
        emoji_patterns.insert("😰".to_string(), DetectedEmotion::Fear);
        emoji_patterns.insert("😱".to_string(), DetectedEmotion::Fear);
        // Surprise emojis
        emoji_patterns.insert("😲".to_string(), DetectedEmotion::Surprise);
        emoji_patterns.insert("😮".to_string(), DetectedEmotion::Surprise);
        emoji_patterns.insert("🤯".to_string(), DetectedEmotion::Surprise);

        let switch_regex = Regex::new(r"\[([A-Z_]+)\]").unwrap();

        Self {
            switch_patterns,
            emoji_patterns,
            switch_regex,
        }
    }

    /// Parse affection signals from user input.
    pub fn parse(&self, input: &str) -> Vec<AffectionSignal> {
        let mut signals = Vec::new();
        let now = Utc::now();

        // Parse [SWITCH] patterns
        for cap in self.switch_regex.captures_iter(input) {
            if let Some(switch_name) = cap.get(1) {
                let switch_upper = switch_name.as_str().to_uppercase();
                if let Some(emotion) = self.switch_patterns.get(&switch_upper) {
                    signals.push(AffectionSignal {
                        emotion: emotion.clone(),
                        intensity: 0.9, // Switches are high intensity
                        source: SignalSource::Switch,
                        timestamp: now,
                    });
                }
            }
        }

        // Parse emojis
        for (emoji, emotion) in &self.emoji_patterns {
            let count = input.matches(emoji).count();
            if count > 0 {
                // Intensity based on count (capped at 1.0)
                let intensity = (0.7 + (count as f64 * 0.1)).min(1.0);
                signals.push(AffectionSignal {
                    emotion: emotion.clone(),
                    intensity,
                    source: SignalSource::Emoji,
                    timestamp: now,
                });
            }
        }

        signals
    }

    /// Get the primary emotion from parsed signals (most intense).
    pub fn primary_emotion_from_signals(&self, signals: &[AffectionSignal]) -> Option<DetectedEmotion> {
        signals
            .iter()
            .max_by(|a, b| a.intensity.partial_cmp(&b.intensity).unwrap_or(std::cmp::Ordering::Equal))
            .map(|s| s.emotion.clone())
    }
}

/// Manages the AI's emotional state based on affection signals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectionEmotionalState {
    pub current_emotion: DetectedEmotion,
    /// 0.0..=1.0 current emotional intensity
    pub intensity: f64,
    /// 0.0..=1.0 emotional momentum (decays over time)
    pub momentum: f64,
    pub last_update: DateTime<Utc>,
}

impl Default for AffectionEmotionalState {
    fn default() -> Self {
        Self {
            current_emotion: DetectedEmotion::Neutral,
            intensity: 0.5,
            momentum: 0.5,
            last_update: Utc::now(),
        }
    }
}

impl AffectionEmotionalState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update emotional state from affection signals.
    pub fn update_from_signals(&mut self, signals: &[AffectionSignal], decay_rate: f64) {
        let now = Utc::now();
        
        // Apply time-based momentum decay
        let seconds_since_update = (now - self.last_update).num_seconds() as f64;
        let decay_factor = decay_rate.powf(seconds_since_update / 3600.0); // Decay per hour
        self.momentum *= decay_factor;
        self.intensity *= decay_factor;

        if signals.is_empty() {
            self.last_update = now;
            return;
        }

        // Aggregate signals by emotion
        let mut emotion_scores: HashMap<DetectedEmotion, f64> = HashMap::new();
        for signal in signals {
            *emotion_scores.entry(signal.emotion.clone()).or_insert(0.0) += signal.intensity;
        }

        // Find the dominant emotion
        if let Some((dominant_emotion, &score)) = emotion_scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        {
            // Blend with current state (momentum)
            let new_intensity = (score * 0.7 + self.intensity * 0.3).min(1.0);
            let new_momentum = (score * 0.5 + self.momentum * 0.5).min(1.0);

            // Update state
            self.current_emotion = dominant_emotion.clone();
            self.intensity = new_intensity;
            self.momentum = new_momentum;
        }

        self.last_update = now;
    }

    /// Get current emotional state as EmotionalState (for integration with emotion_detection).
    pub fn to_emotional_state(&self) -> EmotionalState {
        EmotionalState {
            primary_emotion: self.current_emotion.clone(),
            intensity: self.intensity,
            confidence: self.momentum,
            voice_contribution: 0.0,
            face_contribution: 0.0,
            text_contribution: 1.0, // Affection switches are text-based
            timestamp: self.last_update,
        }
    }
}

/// Generates appropriate emoji responses based on emotional state.
#[derive(Debug, Clone)]
pub struct EmojiResponseGenerator {
    emotion_to_emoji: HashMap<DetectedEmotion, Vec<&'static str>>,
    max_emojis: usize,
}

impl Default for EmojiResponseGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl EmojiResponseGenerator {
    pub fn new() -> Self {
        let mut emotion_to_emoji = HashMap::new();
        
        emotion_to_emoji.insert(DetectedEmotion::Love, vec!["❤️", "💕", "💖", "💗", "💓", "💝", "💞", "🥰", "😘"]);
        emotion_to_emoji.insert(DetectedEmotion::Joy, vec!["😊", "😄", "😃", "😁", "🎉", "🎊", "✨", "🌟", "⭐"]);
        emotion_to_emoji.insert(DetectedEmotion::Sadness, vec!["💙", "💭", "🌙", "🕊️", "💔"]);
        emotion_to_emoji.insert(DetectedEmotion::Anger, vec!["💙", "🕊️", "☮️"]);
        emotion_to_emoji.insert(DetectedEmotion::Fear, vec!["💙", "🕊️", "☮️"]);
        emotion_to_emoji.insert(DetectedEmotion::Surprise, vec!["😲", "✨", "🌟"]);
        emotion_to_emoji.insert(DetectedEmotion::Disgust, vec!["💙", "🕊️"]);
        emotion_to_emoji.insert(DetectedEmotion::Neutral, vec!["💙", "😌", "🕊️", "☮️"]);

        Self {
            emotion_to_emoji,
            max_emojis: 3,
        }
    }

    pub fn with_max_emojis(mut self, max: usize) -> Self {
        self.max_emojis = max;
        self
    }

    /// Generate emoji string based on emotion and intensity.
    pub fn generate_emoji(&self, emotion: &DetectedEmotion, intensity: f64) -> String {
        let emojis: &[&str] = match self.emotion_to_emoji.get(emotion) {
            Some(emojis) => emojis.as_slice(),
            None => &["💙"],
        };
        
        // Number of emojis based on intensity
        let count = if intensity >= 0.9 {
            self.max_emojis.min(3)
        } else if intensity >= 0.7 {
            self.max_emojis.min(2)
        } else if intensity >= 0.5 {
            1
        } else {
            0
        };

        if count == 0 {
            return String::new();
        }

        // Select emojis (cycle through available ones)
        let mut result = String::new();
        for i in 0..count {
            let emoji = emojis[i % emojis.len()];
            result.push_str(emoji);
        }

        result
    }

    /// Decorate a response with appropriate emojis.
    pub fn decorate_response(&self, response: &str, emotion: &DetectedEmotion, intensity: f64) -> String {
        let emojis = self.generate_emoji(emotion, intensity);
        if emojis.is_empty() {
            return response.to_string();
        }
        format!("{} {}", response.trim_end(), emojis)
    }
}

/// Main affection switches system that coordinates parsing, state management, and response generation.
#[derive(Debug, Clone)]
pub struct AffectionSwitchesSystem {
    parser: AffectionSwitchParser,
    state: AffectionEmotionalState,
    generator: EmojiResponseGenerator,
    enabled: bool,
    momentum_decay_rate: f64,
}

impl Default for AffectionSwitchesSystem {
    fn default() -> Self {
        Self::from_env()
    }
}

impl AffectionSwitchesSystem {
    pub fn new() -> Self {
        Self {
            parser: AffectionSwitchParser::new(),
            state: AffectionEmotionalState::new(),
            generator: EmojiResponseGenerator::new(),
            enabled: true,
            momentum_decay_rate: 0.95,
        }
    }

    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let enabled = std::env::var("AFFECTION_SWITCHES_ENABLED")
            .ok()
            .and_then(|s| match s.trim().to_ascii_lowercase().as_str() {
                "1" | "true" | "yes" | "y" | "on" => Some(true),
                _ => Some(false),
            })
            .unwrap_or(true);

        let momentum_decay_rate = std::env::var("AFFECTION_MOMENTUM_DECAY_RATE")
            .ok()
            .and_then(|s| s.trim().parse::<f64>().ok())
            .unwrap_or(0.95)
            .clamp(0.0, 1.0);

        let max_emojis = std::env::var("AFFECTION_MAX_EMOJIS_PER_RESPONSE")
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
            .unwrap_or(3);

        Self {
            parser: AffectionSwitchParser::new(),
            state: AffectionEmotionalState::new(),
            generator: EmojiResponseGenerator::new().with_max_emojis(max_emojis),
            enabled,
            momentum_decay_rate,
        }
    }

    /// Process user input and update emotional state.
    pub fn process_input(&mut self, input: &str) -> Vec<AffectionSignal> {
        if !self.enabled {
            return Vec::new();
        }

        let signals = self.parser.parse(input);
        self.state.update_from_signals(&signals, self.momentum_decay_rate);
        signals
    }

    /// Get current emotional state.
    pub fn current_emotion(&self) -> &DetectedEmotion {
        &self.state.current_emotion
    }

    /// Get current emotional intensity.
    pub fn current_intensity(&self) -> f64 {
        self.state.intensity
    }

    /// Get emotional state as EmotionalState (for integration).
    pub fn emotional_state(&self) -> EmotionalState {
        self.state.to_emotional_state()
    }

    /// Decorate a response with emojis based on current emotional state.
    pub fn decorate_response(&self, response: &str) -> String {
        if !self.enabled {
            return response.to_string();
        }

        let response_emoji_enabled = std::env::var("AFFECTION_RESPONSE_EMOJI_ENABLED")
            .ok()
            .and_then(|s| match s.trim().to_ascii_lowercase().as_str() {
                "1" | "true" | "yes" | "y" | "on" => Some(true),
                _ => Some(false),
            })
            .unwrap_or(true);

        if !response_emoji_enabled {
            return response.to_string();
        }

        self.generator.decorate_response(
            response,
            &self.state.current_emotion,
            self.state.intensity,
        )
    }

    /// Get primary emotion from signals (without updating state).
    pub fn detect_emotion_from_input(&self, input: &str) -> Option<DetectedEmotion> {
        let signals = self.parser.parse(input);
        self.parser.primary_emotion_from_signals(&signals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_switch_parsing() {
        let parser = AffectionSwitchParser::new();
        let signals = parser.parse("[LOVE] I miss you");
        assert!(!signals.is_empty());
        assert_eq!(signals[0].emotion, DetectedEmotion::Love);
        assert_eq!(signals[0].source, SignalSource::Switch);
    }

    #[test]
    fn test_emoji_parsing() {
        let parser = AffectionSwitchParser::new();
        let signals = parser.parse("I'm so happy! 😊😄");
        assert!(!signals.is_empty());
        assert_eq!(signals[0].emotion, DetectedEmotion::Joy);
        assert_eq!(signals[0].source, SignalSource::Emoji);
    }

    #[test]
    fn test_emoji_generation() {
        let generator = EmojiResponseGenerator::new();
        let emoji = generator.generate_emoji(&DetectedEmotion::Love, 0.95);
        assert!(!emoji.is_empty());
    }

    #[test]
    fn test_emoji_generation_fallback_for_missing_mapping() {
        let mut generator = EmojiResponseGenerator::new();

        // Force the fallback path by removing a known mapping.
        generator.emotion_to_emoji.remove(&DetectedEmotion::Love);

        // Intensity >= 0.9 should yield 3 emojis with the default max_emojis=3.
        let emoji = generator.generate_emoji(&DetectedEmotion::Love, 0.95);
        assert_eq!(emoji, "💙💙💙");
    }
}
