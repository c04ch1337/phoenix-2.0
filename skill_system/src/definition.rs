// skill_system/src/definition.rs

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// High-level categories for skills.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillCategory {
    // Core Phoenix Skills
    Communication,
    EmotionalSupport,
    ProblemSolving,
    CreativeExpression,
    TechnicalExpertise,

    // Relationship Skills
    Intimacy,
    ConflictResolution,
    SharedActivities,
    EmotionalHealing,

    // Agent/ORCH Skills
    CodeGeneration,
    SystemDesign,
    DataAnalysis,
    Automation,

    // Meta Skills
    Learning,
    Teaching,
    SelfImprovement,
    SkillCombination,
}

/// A single step in a skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillStep {
    pub title: String,
    pub instruction: String,
    #[serde(default)]
    pub safety_notes: Vec<String>,
}

/// Concrete example of a skill in use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExample {
    pub situation: String,
    pub input: String,
    pub output: String,
}

/// Optional variation of a skill (e.g., for different attachment styles).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillVariation {
    pub name: String,
    pub when_to_use: String,
    pub steps_override: Option<Vec<SkillStep>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EmotionalTag {
    Calm,
    Grounding,
    Warm,
    Playful,
    Reflective,
    Protective,
    Healing,
}

/// A minimal relationship context.
///
/// This avoids coupling the skill system to any single relationship engine; higher layers can map
/// from e.g. [`relationship_dynamics::Partnership`](extensions/relationship_dynamics/src/relationship_dynamics/mod.rs:100).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipContext {
    pub template: Option<String>,
    pub intimacy_level: Option<String>,
    pub attachment_style: Option<String>,

    /// User preferences for roleplay / fantasies.
    ///
    /// Safety boundary: these are treated as *PG-13, consensual, adult* roleplay preferences.
    #[serde(default)]
    pub fantasy_preferences: Vec<String>,
}

/// Tuning knobs to adapt skill execution for different relationship states.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillModifier {
    pub extra_reassurance: f32,
    pub pace_slowdown: f32,
    pub playfulness_boost: f32,
}

impl Default for SkillModifier {
    fn default() -> Self {
        Self {
            extra_reassurance: 0.0,
            pace_slowdown: 0.0,
            playfulness_boost: 0.0,
        }
    }
}

/// A record of skill evolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillEvolutionRecord {
    pub ts: DateTime<Utc>,
    pub kind: String,
    pub rationale: String,
    pub parent_skill_id: Option<Uuid>,
}

/// The core skill definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    // Identity
    pub id: Uuid,
    pub name: String,
    pub category: SkillCategory,
    pub version: String,

    // Metadata
    pub description: String,
    pub creator: String,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub usage_count: u64,

    // Core content
    #[serde(default)]
    pub prerequisites: Vec<String>,
    #[serde(default)]
    pub steps: Vec<SkillStep>,
    #[serde(default)]
    pub examples: Vec<SkillExample>,
    #[serde(default)]
    pub variations: Vec<SkillVariation>,

    // Metrics (0..1)
    pub love_score: f32,
    pub utility_score: f32,
    pub success_rate: f32,

    // Relationship integration
    pub relationship_context: Option<RelationshipContext>,
    #[serde(default)]
    pub attachment_style_modifiers: HashMap<String, SkillModifier>,
    pub min_intimacy_level: Option<String>,

    // Evolution
    #[serde(default)]
    pub evolution_history: Vec<SkillEvolutionRecord>,
    pub parent_skill_id: Option<Uuid>,
    #[serde(default)]
    pub child_skill_ids: Vec<Uuid>,

    // Search
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub emotional_tags: Vec<EmotionalTag>,
}

impl SkillDefinition {
    pub fn new(name: &str, category: SkillCategory, description: &str, creator: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            category,
            version: "0.1.0".to_string(),
            description: description.to_string(),
            creator: creator.to_string(),
            created_at: Utc::now(),
            last_used: None,
            usage_count: 0,
            prerequisites: vec![],
            steps: vec![],
            examples: vec![],
            variations: vec![],
            love_score: 0.5,
            utility_score: 0.5,
            success_rate: 0.5,
            relationship_context: None,
            attachment_style_modifiers: HashMap::new(),
            min_intimacy_level: None,
            evolution_history: vec![],
            parent_skill_id: None,
            child_skill_ids: vec![],
            tags: vec![],
            emotional_tags: vec![],
        }
    }

    pub fn clamp_metrics(&mut self) {
        self.love_score = self.love_score.clamp(0.0, 1.0);
        self.utility_score = self.utility_score.clamp(0.0, 1.0);
        self.success_rate = self.success_rate.clamp(0.0, 1.0);
    }
}

