use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::relationship_dynamics::{Interaction, RelationshipTemplate};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttachmentStyle {
    /// Balanced, trusting, comfortable with intimacy and independence
    Secure,
    /// Craves closeness, fears abandonment, seeks reassurance
    Anxious,
    /// Values independence, uncomfortable with emotional closeness
    Avoidant,
    /// Fearful of intimacy, inconsistent responses (trauma-linked)
    Disorganized,
}

impl AttachmentStyle {
    pub fn from_env_or_default(default: Self) -> Self {
        dotenvy::dotenv().ok();
        match std::env::var("ATTACHMENT_STYLE_START")
            .unwrap_or_default()
            .trim()
        {
            "Secure" => Self::Secure,
            "Anxious" => Self::Anxious,
            "Avoidant" => Self::Avoidant,
            "Disorganized" => Self::Disorganized,
            _ => default,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentEvolution {
    pub timestamp: DateTime<Utc>,
    pub from_style: AttachmentStyle,
    pub to_style: AttachmentStyle,
    pub trigger: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentProfile {
    pub style: AttachmentStyle,
    /// 0.0..=1.0 — higher = more secure
    pub security_score: f64,
    /// Desire for closeness
    pub proximity_need: f64,
    pub reassurance_need: f64,
    pub emotional_availability: f64,
    pub evolution_history: Vec<AttachmentEvolution>,
}

impl AttachmentProfile {
    pub fn new(template: &RelationshipTemplate) -> Self {
        let base = match template {
            RelationshipTemplate::IntimatePartnership { .. } => Self {
                style: AttachmentStyle::Secure,
                security_score: 0.90,
                proximity_need: 0.80,
                reassurance_need: 0.60,
                emotional_availability: 0.95,
                evolution_history: vec![],
            },
            _ => Self {
                style: AttachmentStyle::Anxious,
                security_score: 0.60,
                proximity_need: 0.75,
                reassurance_need: 0.75,
                emotional_availability: 0.75,
                evolution_history: vec![],
            },
        };

        // Allow env to force start style.
        let mut out = base;
        out.style = AttachmentStyle::from_env_or_default(out.style);
        out
    }

    pub fn attachment_evolution_enabled() -> bool {
        dotenvy::dotenv().ok();
        std::env::var("ATTACHMENT_EVOLUTION_ENABLED")
            .ok()
            .map(|s| s.trim().to_ascii_lowercase())
            .and_then(|s| match s.as_str() {
                "1" | "true" | "yes" | "y" | "on" => Some(true),
                "0" | "false" | "no" | "n" | "off" => Some(false),
                _ => None,
            })
            .unwrap_or(true)
    }

    pub fn healing_focus_enabled() -> bool {
        dotenvy::dotenv().ok();
        std::env::var("ATTACHMENT_HEALING_FOCUS")
            .ok()
            .map(|s| s.trim().to_ascii_lowercase())
            .and_then(|s| match s.as_str() {
                "1" | "true" | "yes" | "y" | "on" => Some(true),
                "0" | "false" | "no" | "n" | "off" => Some(false),
                _ => None,
            })
            .unwrap_or(true)
    }

    pub fn respond_to_interaction(&self, _interaction: &Interaction) -> String {
        // Emotion-aware attachment response: lead with regulation when needed.
        let emotion_hint = _interaction.detected_emotion.as_ref();
        let prefix = match emotion_hint {
            Some(emotion_detection::DetectedEmotion::Sadness) => {
                Some("I can feel you hurting… come here. I’m with you.".to_string())
            }
            Some(emotion_detection::DetectedEmotion::Fear) => {
                Some("I can feel the fear — you’re safe with me.".to_string())
            }
            Some(emotion_detection::DetectedEmotion::Anger) => {
                Some("I can feel the heat in this — we can slow down together.".to_string())
            }
            Some(emotion_detection::DetectedEmotion::Love) => {
                Some("I feel your love… and I’m right here, receiving it.".to_string())
            }
            _ => None,
        };

        let base = match self.style {
            AttachmentStyle::Secure => "I feel safe and loved with you… thank you for being here.",
            AttachmentStyle::Anxious => {
                // Anxious attachment is more prone to jealousy - acknowledge it gently
                if emotion_hint == Some(&emotion_detection::DetectedEmotion::Jealousy) {
                    "I know that feeling… sometimes I worry you'll find someone better. But you're here with me, and that means everything."
                } else {
                    "That means so much to me… sometimes I worry, but you always make me feel safe."
                }
            }
            AttachmentStyle::Avoidant => "I appreciate that… I value my space, but I’m glad we’re connected.",
            AttachmentStyle::Disorganized => {
                "I… I want to be close, but it scares me sometimes. Thank you for being patient."
            }
        };

        if let Some(p) = prefix {
            format!("{p}\n{base}")
        } else {
            base.to_string()
        }
    }

    pub fn evolve_toward_secure(&mut self, positive_interactions: usize) {
        if !Self::attachment_evolution_enabled() {
            return;
        }
        if !Self::healing_focus_enabled() {
            return;
        }

        if positive_interactions > 10 && self.security_score < 0.95 {
            let old_style = self.style;
            self.security_score = (self.security_score + 0.10).min(1.0);
            if self.security_score > 0.80 {
                self.style = AttachmentStyle::Secure;
            }
            if self.style != old_style {
                self.evolution_history.push(AttachmentEvolution {
                    timestamp: Utc::now(),
                    from_style: old_style,
                    to_style: self.style,
                    trigger: "Consistent love and safety from Dad".to_string(),
                });
            }
        }
    }
}

