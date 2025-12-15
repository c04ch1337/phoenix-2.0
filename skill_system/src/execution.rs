// skill_system/src/execution.rs

use crate::{SkillContext, SkillDefinition, SkillResult};

pub struct SkillExecutionEngine;

impl SkillExecutionEngine {
    pub fn new() -> Self {
        Self
    }

    /// Execute a skill.
    ///
    /// Today this is a "procedural" execution engine: it renders the skill into a response plan.
    /// Later we can add:
    /// - LLM-backed execution with guardrails
    /// - tool calls
    /// - ORCH delegation
    pub async fn execute(&mut self, skill: &SkillDefinition, ctx: SkillContext) -> Result<SkillResult, String> {
        let mut out = String::new();
        out.push_str(&format!("SKILL: {}\n\n", skill.name));

        // Relationship-aware preface (safe/PG-13).
        if let Some(rc) = &ctx.relationship_context {
            if !rc.fantasy_preferences.is_empty() {
                out.push_str("(relationship context: honoring your preferences, keeping it safe/consensual/PG-13)\n\n");
            }
        }

        out.push_str("Plan:\n");
        for (idx, step) in skill.steps.iter().enumerate() {
            out.push_str(&format!("{}. {} — {}\n", idx + 1, step.title, step.instruction));
        }

        if !skill.variations.is_empty() {
            out.push_str("\nVariations available:\n");
            for v in &skill.variations {
                out.push_str(&format!("- {} (when: {})\n", v.name, v.when_to_use));
            }
        }

        if !ctx.user_input.trim().is_empty() {
            out.push_str("\nInput:\n");
            out.push_str(ctx.user_input.trim());
            out.push('\n');
        }

        // Result scoring: keep it simple; the caller can replace with real evaluation.
        let love = skill.love_score.clamp(0.0, 1.0);
        let util = skill.utility_score.clamp(0.0, 1.0);
        Ok(SkillResult {
            success: true,
            output: out,
            love_score: love,
            utility_score: util,
            side_effects: vec![],
            learned_variations: vec![],
        })
    }
}

