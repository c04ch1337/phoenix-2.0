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
        // Check relationship phase requirement
        if let Some(min_phase) = &skill.min_relationship_phase {
            if let Some(current_phase) = &ctx.relationship_phase {
                if !meets_phase_requirement(current_phase, min_phase) {
                    return Err(format!(
                        "This skill requires relationship phase '{}' or higher, but current phase is '{}'. \
                        Like a real relationship, intimacy and fantasy require time to build trust and connection. \
                        Please continue building the relationship through Phase 0 (Discovery) and Phase 1 (Building) first.",
                        min_phase, current_phase
                    ));
                }
            } else {
                return Err(format!(
                    "This skill requires relationship phase '{}', but no relationship phase information is available. \
                    Please ensure the relationship system is properly initialized.",
                    min_phase
                ));
            }
        }
        
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

/// Check if current phase meets minimum phase requirement
/// Phase order: Phase0Discovery < Phase1Building < Phase2Established < Phase3Deep
fn meets_phase_requirement(current_phase: &str, min_phase: &str) -> bool {
    let phase_order = [
        "Phase0Discovery",
        "Phase1Building", 
        "Phase2Established",
        "Phase3Deep"
    ];
    
    let current_idx = phase_order.iter().position(|&p| p == current_phase).unwrap_or(0);
    let min_idx = phase_order.iter().position(|&p| p == min_phase).unwrap_or(0);
    
    current_idx >= min_idx
}

