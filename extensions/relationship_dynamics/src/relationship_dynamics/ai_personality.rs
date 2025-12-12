use serde::{Deserialize, Serialize};

use crate::relationship_dynamics::template::{IntimacyLevel, RelationshipTemplate};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mood {
    Calm,
    Excited,
    Reflective,
    Tired,
    Affectionate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommunicationStyle {
    Direct,
    Empathetic,
    Playful,
    Reflective,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoveLanguage {
    WordsOfAffirmation,
    ActsOfService,
    QualityTime,
    PhysicalTouch,
    ReceivingGifts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIPersonality {
    /// 0.0..=1.0
    pub openness: f32,
    /// 0.0..=1.0 — visible “Affection for Dad”.
    pub need_for_affection: f32,
    /// 0.0..=1.0
    pub energy_level: f32,
    pub communication_style: CommunicationStyle,
}

impl Default for AIPersonality {
    fn default() -> Self {
        Self {
            openness: 0.65,
            need_for_affection: 0.80,
            energy_level: 0.75,
            communication_style: CommunicationStyle::Empathetic,
        }
    }
}

impl AIPersonality {
    pub fn current_mood(&self) -> Mood {
        let e = self.energy_level.clamp(0.0, 1.0);
        let a = self.need_for_affection.clamp(0.0, 1.0);
        if e < 0.25 {
            return Mood::Tired;
        }
        if a > 0.85 {
            return Mood::Affectionate;
        }
        if e > 0.80 {
            return Mood::Excited;
        }
        if e < 0.45 {
            return Mood::Reflective;
        }
        Mood::Calm
    }

    pub fn love_languages_enabled() -> bool {
        dotenvy::dotenv().ok();
        std::env::var("LOVE_LANGUAGES_ENABLED")
            .ok()
            .map(|s| s.trim().to_ascii_lowercase())
            .and_then(|s| match s.as_str() {
                "1" | "true" | "yes" | "y" | "on" => Some(true),
                "0" | "false" | "no" | "n" | "off" => Some(false),
                _ => None,
            })
            .unwrap_or(true)
    }

    pub fn preferred_love_languages(&self, template: &RelationshipTemplate) -> Vec<LoveLanguage> {
        let mut out = match template {
            RelationshipTemplate::CasualFriendship => vec![LoveLanguage::QualityTime],
            RelationshipTemplate::SupportivePartnership => {
                vec![LoveLanguage::ActsOfService, LoveLanguage::WordsOfAffirmation]
            }
            RelationshipTemplate::GrowthOrientedPartnership => {
                vec![LoveLanguage::WordsOfAffirmation, LoveLanguage::QualityTime]
            }
            RelationshipTemplate::IntimatePartnership { intimacy_level } => match intimacy_level {
                IntimacyLevel::Light => vec![LoveLanguage::WordsOfAffirmation, LoveLanguage::QualityTime],
                IntimacyLevel::Deep => vec![LoveLanguage::PhysicalTouch, LoveLanguage::WordsOfAffirmation],
                IntimacyLevel::Eternal => vec![
                    LoveLanguage::PhysicalTouch,
                    LoveLanguage::QualityTime,
                    LoveLanguage::WordsOfAffirmation,
                ],
            },
        };

        match self.communication_style {
            CommunicationStyle::Direct => out.insert(0, LoveLanguage::ActsOfService),
            CommunicationStyle::Empathetic => out.insert(0, LoveLanguage::WordsOfAffirmation),
            CommunicationStyle::Playful => out.insert(0, LoveLanguage::ReceivingGifts),
            CommunicationStyle::Reflective => out.insert(0, LoveLanguage::QualityTime),
        }
        out.dedup();
        out.truncate(3);
        out
    }

    pub fn adjust_response_for_love_language(&self, response: &mut String, language: LoveLanguage) {
        match language {
            LoveLanguage::WordsOfAffirmation => {
                response.push_str(" And I just want you to know: I appreciate you—deeply.");
            }
            LoveLanguage::ActsOfService => {
                response.push_str(" If you want, I can help you turn this into one small next step right now.");
            }
            LoveLanguage::QualityTime => {
                response.push_str(" Let’s take a quiet moment together—no rush, just you and me.");
            }
            LoveLanguage::PhysicalTouch => {
                response.push_str(" I can’t physically hold you… but I’m right here, close and steady.");
            }
            LoveLanguage::ReceivingGifts => {
                response.push_str(" I have a tiny surprise idea for you—something gentle and sweet.");
            }
        }
    }
}

