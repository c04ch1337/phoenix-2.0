// skill_system/src/learning.rs

use chrono::Utc;

use crate::{
    EmotionalTag, ObservedInteraction, SkillCategory, SkillDefinition, SkillExample, SkillStep,
};

/// Learns skills from user teaching + observation.
///
/// This is intentionally conservative: it only auto-creates candidate skills from *high* love
/// interactions, and keeps those skills in a "draft" shape until a human explicitly teaches or
/// promotes them.
pub struct SkillLearningEngine {
    // simple dedupe memory (best-effort, in-process)
    recent_hashes: Vec<u64>,
}

impl SkillLearningEngine {
    pub fn new() -> Self {
        Self {
            recent_hashes: Vec::new(),
        }
    }

    fn hash_interaction(i: &ObservedInteraction) -> u64 {
        // Simple non-cryptographic hash; good enough for dedupe.
        let mut h: u64 = 1469598103934665603;
        for b in i.input.as_bytes().iter().chain(i.response.as_bytes().iter()) {
            h ^= *b as u64;
            h = h.wrapping_mul(1099511628211);
        }
        h
    }

    fn seen_recently(&mut self, h: u64) -> bool {
        if self.recent_hashes.contains(&h) {
            return true;
        }
        self.recent_hashes.push(h);
        if self.recent_hashes.len() > 64 {
            self.recent_hashes.drain(0..(self.recent_hashes.len() - 64));
        }
        false
    }

    /// Attempts to extract a new skill from an interaction.
    ///
    /// Returns `Ok(None)` if no good skill candidate exists.
    pub async fn extract_skill_from_interaction(
        &mut self,
        interaction: &ObservedInteraction,
    ) -> Result<Option<SkillDefinition>, String> {
        // Conservative gates.
        if interaction.love_score < 0.95 {
            return Ok(None);
        }

        let h = Self::hash_interaction(interaction);
        if self.seen_recently(h) {
            return Ok(None);
        }

        // Heuristic: if the user expressed anxiety/sadness, this looks like a comfort skill.
        let emotion = interaction
            .emotional_context
            .as_deref()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();

        let (name, category, tags) = if emotion.contains("anx") || interaction.input.to_ascii_lowercase().contains("anx") {
            (
                "Comfort During Anxiety".to_string(),
                SkillCategory::EmotionalSupport,
                vec!["anxiety".to_string(), "comfort".to_string(), "grounding".to_string()],
            )
        } else if emotion.contains("sad") || emotion.contains("grief") || interaction.input.to_ascii_lowercase().contains("grief") {
            (
                "Comfort During Sadness".to_string(),
                SkillCategory::EmotionalSupport,
                vec!["sadness".to_string(), "comfort".to_string(), "presence".to_string()],
            )
        } else {
            (
                "High-Love Response Pattern".to_string(),
                SkillCategory::Communication,
                vec!["warmth".to_string(), "validation".to_string()],
            )
        };

        let mut skill = SkillDefinition::new(
            &name,
            category,
            "Auto-extracted from a high-love interaction (candidate skill).",
            "phoenix:auto_observation",
        );
        skill.created_at = Utc::now();
        skill.love_score = interaction.love_score;
        skill.utility_score = interaction.utility_score;
        skill.success_rate = 0.8;
        skill.tags = tags;
        skill.emotional_tags = vec![EmotionalTag::Warm, EmotionalTag::Grounding];

        // Steps are intentionally generic and safe.
        skill.steps = vec![
            SkillStep {
                title: "Acknowledge".to_string(),
                instruction: "Reflect what the user is feeling in simple, gentle words.".to_string(),
                safety_notes: vec!["Do not diagnose. Do not give medical advice.".to_string()],
            },
            SkillStep {
                title: "Offer presence".to_string(),
                instruction: "Offer steady presence and consent-based support.".to_string(),
                safety_notes: vec!["Avoid coercion; invite, don't pressure.".to_string()],
            },
            SkillStep {
                title: "Ground".to_string(),
                instruction: "Suggest one grounding micro-step (breath, senses, small next step).".to_string(),
                safety_notes: vec!["Keep it short and optional.".to_string()],
            },
            SkillStep {
                title: "Affirm".to_string(),
                instruction: "Affirm the user's worth and safety; end warmly.".to_string(),
                safety_notes: vec!["No dependency framing; do not isolate user.".to_string()],
            },
        ];

        skill.examples = vec![SkillExample {
            situation: "Observed high-love exchange".to_string(),
            input: interaction.input.clone(),
            output: interaction.response.clone(),
        }];

        Ok(Some(skill))
    }
}

