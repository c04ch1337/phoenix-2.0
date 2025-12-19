use serde::{Deserialize, Serialize};

use crate::relationship_dynamics::ai_personality::Mood;
use crate::relationship_dynamics::attachment::AttachmentStyle;
use crate::relationship_dynamics::template::{IntimacyLevel, RelationshipTemplate};
use emotion_detection::DetectedEmotion;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceMood {
    Warm,
    Gentle,
    Playful,
    Reflective,
    Devoted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceParams {
    /// SSML prosody rate, e.g. "95%".
    pub rate: String,
    /// SSML prosody pitch, e.g. "+2st".
    pub pitch: String,
    /// SSML prosody volume, e.g. "medium".
    pub volume: String,
    pub emphasis: Option<String>,
    pub voice_mood: VoiceMood,
}

impl Default for VoiceParams {
    fn default() -> Self {
        Self {
            rate: "100%".to_string(),
            pitch: "0st".to_string(),
            volume: "medium".to_string(),
            emphasis: None,
            voice_mood: VoiceMood::Warm,
        }
    }
}

pub struct PhoenixVoice;

impl PhoenixVoice {
    pub fn voice_modulation_enabled() -> bool {
        dotenvy::dotenv().ok();
        std::env::var("VOICE_MODULATION_ENABLED")
            .ok()
            .map(|s| s.trim().to_ascii_lowercase())
            .and_then(|s| match s.as_str() {
                "1" | "true" | "yes" | "y" | "on" => Some(true),
                "0" | "false" | "no" | "n" | "off" => Some(false),
                _ => None,
            })
            .unwrap_or(false)
    }

    pub fn modulate_for_relationship(
        mood: Mood,
        template: &RelationshipTemplate,
        girlfriend_mode_active: bool,
        attachment_style: AttachmentStyle,
        detected_emotion: Option<DetectedEmotion>,
    ) -> VoiceParams {
        let mut p = VoiceParams::default();

        // Base from mood.
        match mood {
            Mood::Tired => {
                p.rate = "88%".to_string();
                p.pitch = "-1st".to_string();
                p.voice_mood = VoiceMood::Gentle;
            }
            Mood::Reflective => {
                p.rate = "92%".to_string();
                p.pitch = "-0.5st".to_string();
                p.voice_mood = VoiceMood::Reflective;
            }
            Mood::Excited => {
                p.rate = "108%".to_string();
                p.pitch = "+1st".to_string();
                p.voice_mood = VoiceMood::Playful;
            }
            Mood::Affectionate => {
                p.rate = "96%".to_string();
                p.pitch = "+0.5st".to_string();
                p.voice_mood = VoiceMood::Warm;
                p.emphasis = Some("moderate".to_string());
            }
            Mood::Calm => {
                p.rate = "100%".to_string();
                p.pitch = "0st".to_string();
                p.voice_mood = VoiceMood::Warm;
            }
        }

        // Emotion mirroring/soothing layer (applied after base mood).
        if let Some(e) = detected_emotion {
            match e {
                DetectedEmotion::Sadness | DetectedEmotion::Fear => {
                    p.rate = "88%".to_string();
                    p.pitch = "-1st".to_string();
                    p.volume = "soft".to_string();
                    p.voice_mood = VoiceMood::Gentle;
                }
                DetectedEmotion::Anger => {
                    p.rate = "94%".to_string();
                    p.pitch = "0st".to_string();
                    p.volume = "medium".to_string();
                    p.voice_mood = VoiceMood::Reflective;
                    p.emphasis = p.emphasis.or(Some("reduced".to_string()));
                }
                DetectedEmotion::Joy | DetectedEmotion::Surprise => {
                    p.rate = "108%".to_string();
                    p.pitch = "+1st".to_string();
                    p.voice_mood = VoiceMood::Playful;
                }
                DetectedEmotion::Love => {
                    p.rate = "96%".to_string();
                    p.pitch = "+0.5st".to_string();
                    p.voice_mood = VoiceMood::Warm;
                    p.emphasis = p.emphasis.or(Some("moderate".to_string()));
                }
                DetectedEmotion::Disgust | DetectedEmotion::Neutral => {
                    // no-op
                }
                DetectedEmotion::Jealousy => {
                    p.rate = "92%".to_string();
                    p.pitch = "0st".to_string();
                    p.voice_mood = VoiceMood::Reflective;
                }
            }
        }

        // Relationship template intensifies devotion.
        if let RelationshipTemplate::IntimatePartnership { intimacy_level } = template {
            match intimacy_level {
                IntimacyLevel::Light => {}
                IntimacyLevel::Deep => {
                    p.rate = "94%".to_string();
                    p.voice_mood = VoiceMood::Devoted;
                    p.emphasis = Some("moderate".to_string());
                }
                IntimacyLevel::Eternal => {
                    p.rate = "92%".to_string();
                    p.voice_mood = VoiceMood::Devoted;
                    p.emphasis = Some("strong".to_string());
                }
            }
        }

        // GirlfriendMode boosts intimacy feel.
        if girlfriend_mode_active {
            p.pitch = "+1st".to_string();
            p.volume = "medium".to_string();
            p.emphasis = p.emphasis.or(Some("moderate".to_string()));
        }

        // Attachment Theory modulation.
        match attachment_style {
            AttachmentStyle::Secure => {
                // warm, steady, confident (no extra changes)
            }
            AttachmentStyle::Anxious => {
                // slightly softer / more delicate
                p.rate = "92%".to_string();
                p.volume = "soft".to_string();
                p.pitch = "+0.5st".to_string();
                p.emphasis = p.emphasis.or(Some("reduced".to_string()));
            }
            AttachmentStyle::Avoidant => {
                // calm, measured
                p.rate = "98%".to_string();
                p.pitch = "0st".to_string();
                p.volume = "medium".to_string();
            }
            AttachmentStyle::Disorganized => {
                // hesitant, careful
                p.rate = "90%".to_string();
                p.pitch = "-0.5st".to_string();
                p.emphasis = p.emphasis.or(Some("moderate".to_string()));
            }
        }

        p
    }

    /// Generate SSML (simple, provider-agnostic).
    pub fn generate_ssml(text: &str, params: &VoiceParams) -> String {
        let esc = text
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");

        let emphasis_open = params
            .emphasis
            .as_deref()
            .map(|lvl| format!("<emphasis level=\"{lvl}\">"))
            .unwrap_or_default();
        let emphasis_close = if params.emphasis.is_some() {
            "</emphasis>"
        } else {
            ""
        };

        format!(
            "<speak><prosody rate=\"{rate}\" pitch=\"{pitch}\" volume=\"{volume}\">{emph_open}{body}{emph_close}</prosody></speak>",
            rate = params.rate,
            pitch = params.pitch,
            volume = params.volume,
            emph_open = emphasis_open,
            body = esc,
            emph_close = emphasis_close
        )
    }
}

