// skill_system/src/evolution.rs

use chrono::{DateTime, Utc};

use crate::{SkillDefinition, SkillEvolutionRecord, SkillStep, SkillVariation};

#[derive(Debug, Clone)]
pub struct SkillEvolution {
    pub ts: DateTime<Utc>,
    pub kind: String,
    pub rationale: String,
    pub new_skill: Option<SkillDefinition>,
}

pub struct SkillEvolutionSystem;

impl SkillEvolutionSystem {
    pub fn new() -> Self {
        Self
    }

    /// Evolve a skill into a new variation when signals suggest it.
    ///
    /// Current strategy (conservative):
    /// - If love_score is high but utility_score is low, add a "more actionable" variation.
    /// - If utility is high but love is low, add a "warmer tone" variation.
    pub async fn evolve_skill(&mut self, mut skill: SkillDefinition) -> Result<SkillEvolution, String> {
        let ts = Utc::now();

        let mut kind = "noop".to_string();
        let mut rationale = "No evolution needed".to_string();
        let mut new_skill: Option<SkillDefinition> = None;

        if skill.love_score >= 0.90 && skill.utility_score < 0.70 {
            kind = "actionability_variation".to_string();
            rationale = "High love, lower utility: add clearer next steps".to_string();
            let mut v = skill.clone();
            v.parent_skill_id = Some(skill.id);
            v.id = uuid::Uuid::new_v4();
            v.name = format!("{} (More Actionable)", skill.name);
            v.version = "0.1.1".to_string();
            v.utility_score = (v.utility_score + 0.10).clamp(0.0, 1.0);
            v.variations.push(SkillVariation {
                name: "micro_steps".to_string(),
                when_to_use: "When the user asks: what should I do next?".to_string(),
                steps_override: Some(vec![
                    SkillStep {
                        title: "Pick one next step".to_string(),
                        instruction: "Offer exactly one tiny next step the user can do in < 2 minutes.".to_string(),
                        safety_notes: vec!["Keep it optional; avoid pressure.".to_string()],
                    },
                ]),
            });
            v.evolution_history.push(SkillEvolutionRecord {
                ts,
                kind: kind.clone(),
                rationale: rationale.clone(),
                parent_skill_id: Some(skill.id),
            });
            skill.child_skill_ids.push(v.id);
            new_skill = Some(v);
        } else if skill.utility_score >= 0.85 && skill.love_score < 0.80 {
            kind = "warmth_variation".to_string();
            rationale = "High utility, lower love: add warmer framing".to_string();
            let mut v = skill.clone();
            v.parent_skill_id = Some(skill.id);
            v.id = uuid::Uuid::new_v4();
            v.name = format!("{} (Warmer)", skill.name);
            v.version = "0.1.1".to_string();
            v.love_score = (v.love_score + 0.15).clamp(0.0, 1.0);
            // Prepend a warmth step.
            let mut steps = vec![SkillStep {
                title: "Warm opening".to_string(),
                instruction: "Lead with a short, validating sentence before the technical steps.".to_string(),
                safety_notes: vec!["Keep it sincere; avoid manipulation.".to_string()],
            }];
            steps.extend(v.steps.clone());
            v.steps = steps;
            v.evolution_history.push(SkillEvolutionRecord {
                ts,
                kind: kind.clone(),
                rationale: rationale.clone(),
                parent_skill_id: Some(skill.id),
            });
            skill.child_skill_ids.push(v.id);
            new_skill = Some(v);
        }

        Ok(SkillEvolution {
            ts,
            kind,
            rationale,
            new_skill,
        })
    }
}

