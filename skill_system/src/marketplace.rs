// skill_system/src/marketplace.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{SkillCategory, SkillDefinition};

/// A marketplace listing (metadata + optional payload).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMarketplaceEntry {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub description: String,
    pub category: SkillCategory,
    pub published_at: DateTime<Utc>,
    pub love_score: f32,
    pub utility_score: f32,
    pub success_rate: f32,
    pub tags: Vec<String>,

    /// If true, skill should not be shared publicly (e.g., relationship-private preferences).
    pub private: bool,

    /// Optional embedded skill payload.
    pub skill: Option<SkillDefinition>,
}

/// Placeholder for a future marketplace implementation.
pub struct SkillMarketplace;

impl SkillMarketplace {
    pub fn new() -> Self {
        Self
    }

    pub fn publish(&self, skill: &SkillDefinition, private: bool) -> SkillMarketplaceEntry {
        SkillMarketplaceEntry {
            id: skill.id,
            name: skill.name.clone(),
            version: skill.version.clone(),
            description: skill.description.clone(),
            category: skill.category,
            published_at: Utc::now(),
            love_score: skill.love_score,
            utility_score: skill.utility_score,
            success_rate: skill.success_rate,
            tags: skill.tags.clone(),
            private,
            // Keep payload optional; in a real marketplace this might be gated.
            skill: if private { None } else { Some(skill.clone()) },
        }
    }
}

