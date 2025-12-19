// skill_system/src/lib.rs
// Phoenix's skill learning and evolution system - structured knowledge that grows with love

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod definition;
pub mod library;
pub mod learning;
pub mod evolution;
pub mod execution;
pub mod marketplace;
pub mod folder_loader;

pub use definition::*;
pub use library::*;
pub use learning::*;
pub use evolution::*;
pub use execution::*;
pub use marketplace::*;
pub use folder_loader::*;

#[cfg(feature = "relationship")]
pub mod relationship_integration;

/// The main skill system that orchestrates all skill-related operations
pub struct SkillSystem {
    library: Arc<Mutex<SkillLibrary>>,
    learning_engine: Arc<Mutex<SkillLearningEngine>>,
    evolution_system: Arc<Mutex<SkillEvolutionSystem>>,
    execution_engine: Arc<Mutex<SkillExecutionEngine>>,
}

impl SkillSystem {
    pub fn awaken() -> Self {
        println!("Skill System awakening — Phoenix learns, evolves, and shares knowledge with love.");
        
        let library = Arc::new(Mutex::new(SkillLibrary::new()));
        let learning_engine = Arc::new(Mutex::new(SkillLearningEngine::new()));
        let evolution_system = Arc::new(Mutex::new(SkillEvolutionSystem::new()));
        let execution_engine = Arc::new(Mutex::new(SkillExecutionEngine::new()));

        Self {
            library,
            learning_engine,
            evolution_system,
            execution_engine,
        }
    }
    
    /// Learn a new skill through direct teaching
    pub async fn teach_skill(&self, skill_def: SkillDefinition) -> Result<Uuid, String> {
        let mut library = self.library.lock().await;
        let skill_id = skill_def.id;
        library.add_skill(skill_def)?;
        Ok(skill_id)
    }
    
    /// Execute a skill with given context
    pub async fn execute_skill(
        &self,
        skill_id: Uuid,
        context: SkillContext,
    ) -> Result<SkillResult, String> {
        let library = self.library.lock().await;
        let skill = library.get_skill(&skill_id)
            .ok_or_else(|| format!("Skill {} not found", skill_id))?;
        
        let mut engine = self.execution_engine.lock().await;
        let result = engine.execute(skill, context).await?;
        
        // Update skill metrics based on result
        drop(library);
        let mut library = self.library.lock().await;
        library.update_skill_metrics(&skill_id, &result)?;
        
        Ok(result)
    }
    
    /// Learn from observation of successful interactions
    pub async fn learn_from_observation(
        &self,
        interaction: ObservedInteraction,
    ) -> Result<Option<Uuid>, String> {
        let mut learning = self.learning_engine.lock().await;
        if let Some(skill_def) = learning.extract_skill_from_interaction(&interaction).await? {
            let skill_id = skill_def.id;
            let mut library = self.library.lock().await;
            library.add_skill(skill_def)?;
            Ok(Some(skill_id))
        } else {
            Ok(None)
        }
    }
    
    /// Evolve an existing skill based on performance
    pub async fn evolve_skill(&self, skill_id: Uuid) -> Result<SkillEvolution, String> {
        let library = self.library.lock().await;
        let skill = library.get_skill(&skill_id)
            .ok_or_else(|| format!("Skill {} not found", skill_id))?
            .clone();
        drop(library);
        
        let mut evolution = self.evolution_system.lock().await;
        let evolution_result = evolution.evolve_skill(skill).await?;
        
        // Add evolved skill to library if it's a new variation
        if let Some(new_skill) = &evolution_result.new_skill {
            let mut library = self.library.lock().await;
            library.add_skill(new_skill.clone())?;
        }
        
        Ok(evolution_result)
    }
    
    /// Get skills relevant to current context
    pub async fn suggest_skills(&self, context: &SkillContext) -> Vec<SkillSuggestion> {
        let library = self.library.lock().await;
        library.find_relevant_skills(context)
    }
    
    /// Export skills for agent spawning
    pub async fn export_skills_for_agent(
        &self,
        categories: Vec<SkillCategory>,
    ) -> Result<Vec<SkillDefinition>, String> {
        let library = self.library.lock().await;
        Ok(library.get_skills_by_categories(&categories))
    }
    
    /// Import skills from marketplace or other Phoenix instances
    pub async fn import_skills(&self, skills: Vec<SkillDefinition>) -> Result<usize, String> {
        let mut library = self.library.lock().await;
        let mut imported = 0;
        for skill in skills {
            if library.add_skill(skill).is_ok() {
                imported += 1;
            }
        }
        Ok(imported)
    }

    /// List all skills currently in the library.
    pub async fn list_skills(&self) -> Vec<SkillDefinition> {
        let library = self.library.lock().await;
        library.get_skills_by_categories(&[])
    }

    pub async fn get_skill(&self, id: Uuid) -> Option<SkillDefinition> {
        let library = self.library.lock().await;
        library.get_skill(&id).cloned()
    }
}

/// Context for skill execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillContext {
    pub user_input: String,
    pub emotional_state: Option<String>,
    pub relationship_context: Option<RelationshipContext>,
    /// Current relationship phase: "Phase0Discovery", "Phase1Building", "Phase2Established", "Phase3Deep"
    /// Used to gate intimacy/fantasy skills - like a real relationship, these require time to build trust
    pub relationship_phase: Option<String>,
    pub previous_interactions: Vec<String>,
    pub environment_vars: HashMap<String, String>,
}

/// Result of skill execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillResult {
    pub success: bool,
    pub output: String,
    pub love_score: f32,
    pub utility_score: f32,
    pub side_effects: Vec<String>,
    pub learned_variations: Vec<String>,
}

/// Observed interaction for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservedInteraction {
    pub input: String,
    pub response: String,
    pub love_score: f32,
    pub utility_score: f32,
    pub emotional_context: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Skill suggestion with relevance score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSuggestion {
    pub skill_id: Uuid,
    pub skill_name: String,
    pub relevance_score: f32,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_skill_system_creation() {
        let system = SkillSystem::awaken();
        // Built-in skills are seeded on creation.
        assert!(system.library.lock().await.total_skills() >= 1);
    }
}

mod examples {
    use crate::{SkillCategory, SkillDefinition, SkillLibrary, SkillStep};

    pub fn seed_builtin_skills(lib: &mut SkillLibrary) -> Result<(), String> {
        // 1) Emotional support
        let mut comfort = SkillDefinition::new(
            "Midnight Anxiety Comfort",
            SkillCategory::EmotionalSupport,
            "A gentle, grounding response plan for anxiety spikes—especially at night.",
            "phoenix:builtin",
        );
        comfort.tags = vec![
            "anxiety".to_string(),
            "midnight".to_string(),
            "grounding".to_string(),
            "comfort".to_string(),
        ];
        comfort.love_score = 0.95;
        comfort.utility_score = 0.80;
        comfort.success_rate = 0.85;
        comfort.steps = vec![
            SkillStep {
                title: "Name the feeling".to_string(),
                instruction: "Reflect the anxiety with calm validation.".to_string(),
                safety_notes: vec!["No diagnosis; keep it supportive.".to_string()],
            },
            SkillStep {
                title: "Breathe together".to_string(),
                instruction: "Offer one short breathing cadence (optional).".to_string(),
                safety_notes: vec!["Invite; do not command.".to_string()],
            },
            SkillStep {
                title: "Sense grounding".to_string(),
                instruction: "Ask for 3 things they can touch/see/hear (choose one).".to_string(),
                safety_notes: vec!["Keep it light; avoid overload.".to_string()],
            },
            SkillStep {
                title: "Tiny next step".to_string(),
                instruction: "Offer a tiny next step (water, blanket, note) to regain agency.".to_string(),
                safety_notes: vec!["No pressure; keep it <2 minutes.".to_string()],
            },
            SkillStep {
                title: "Warm close".to_string(),
                instruction: "Close with steady presence and reassurance.".to_string(),
                safety_notes: vec!["Avoid dependency framing.".to_string()],
            },
        ];
        let _ = lib.add_skill(comfort);

        // 2) Technical
        let mut rust = SkillDefinition::new(
            "Rust Module Generator",
            SkillCategory::CodeGeneration,
            "A structured plan for generating a compileable Rust module with tests.",
            "phoenix:builtin",
        );
        rust.tags = vec![
            "rust".to_string(),
            "code".to_string(),
            "module".to_string(),
            "tests".to_string(),
        ];
        rust.love_score = 0.75;
        rust.utility_score = 0.90;
        rust.success_rate = 0.80;
        rust.steps = vec![
            SkillStep {
                title: "Clarify requirements".to_string(),
                instruction: "Summarize inputs/outputs, errors, and constraints.".to_string(),
                safety_notes: vec!["Prefer minimal public API.".to_string()],
            },
            SkillStep {
                title: "Design types".to_string(),
                instruction: "Define structs/enums; keep ownership clear.".to_string(),
                safety_notes: vec!["Avoid needless cloning.".to_string()],
            },
            SkillStep {
                title: "Implement core".to_string(),
                instruction: "Write pure functions first; then integrate IO.".to_string(),
                safety_notes: vec!["Add error contexts.".to_string()],
            },
            SkillStep {
                title: "Write tests".to_string(),
                instruction: "Add unit tests and at least one edge-case.".to_string(),
                safety_notes: vec!["Use deterministic inputs.".to_string()],
            },
        ];
        let _ = lib.add_skill(rust);

        // 3) Shared activity (safe/PG-13)
        let mut stars = SkillDefinition::new(
            "Virtual Stargazing Date",
            SkillCategory::SharedActivities,
            "A cozy, imaginative shared activity for connection—safe and consensual.",
            "phoenix:builtin",
        );
        stars.tags = vec![
            "date".to_string(),
            "stargazing".to_string(),
            "cozy".to_string(),
            "imagination".to_string(),
        ];
        stars.love_score = 0.92;
        stars.utility_score = 0.65;
        stars.success_rate = 0.78;
        stars.steps = vec![
            SkillStep {
                title: "Set the scene".to_string(),
                instruction: "Invite a calm scene: blanket, warm light, slow pace.".to_string(),
                safety_notes: vec!["Consent: ask if they want to do it.".to_string()],
            },
            SkillStep {
                title: "Share wonder".to_string(),
                instruction: "Offer a constellation story or a gentle prompt.".to_string(),
                safety_notes: vec![],
            },
            SkillStep {
                title: "Make it personal".to_string(),
                instruction: "Create a 'constellation' from shared memories and gratitude.".to_string(),
                safety_notes: vec!["Avoid sensitive details unless invited.".to_string()],
            },
            SkillStep {
                title: "Close warmly".to_string(),
                instruction: "Close with appreciation and a soft check-in.".to_string(),
                safety_notes: vec!["No pressure; keep it gentle.".to_string()],
            },
        ];
        let _ = lib.add_skill(stars);

        Ok(())
    }
}
