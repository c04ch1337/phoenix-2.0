// skill_system/src/library.rs

use std::collections::{HashMap, HashSet};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{SkillCategory, SkillContext, SkillDefinition, SkillResult, SkillSuggestion};

/// A simple in-memory skill library.
///
/// Persistence is handled by higher layers (e.g., [`skill_system::marketplace`](skill_system/src/marketplace.rs:1)
/// and the Phoenix vaults).
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SkillLibrary {
    skills: HashMap<Uuid, SkillDefinition>,

    /// Lowercased tag -> skill ids.
    #[serde(default)]
    tag_index: HashMap<String, HashSet<Uuid>>,
}

impl SkillLibrary {
    pub fn new() -> Self {
        let mut lib = Self::default();
        // Best-effort: seed built-ins. Ignore errors to keep initialization infallible.
        let _ = crate::examples::seed_builtin_skills(&mut lib);
        
        // Best-effort: load skills from folder structure
        if let Some(skills_dir) = crate::folder_loader::find_skills_directory() {
            match crate::folder_loader::load_skills_from_folder(&mut lib, skills_dir.to_str().unwrap_or("skills")) {
                Ok(result) => {
                    if result.loaded > 0 {
                        println!("Loaded {} skills from folder structure", result.loaded);
                    }
                    if result.failed > 0 {
                        eprintln!("Failed to load {} skills: {:?}", result.failed, result.errors);
                    }
                }
                Err(e) => {
                    eprintln!("Error loading skills from folder: {}", e);
                }
            }
        }
        
        lib
    }

    pub fn total_skills(&self) -> usize {
        self.skills.len()
    }

    pub fn add_skill(&mut self, mut skill: SkillDefinition) -> Result<(), String> {
        if skill.name.trim().is_empty() {
            return Err("skill name cannot be empty".to_string());
        }
        skill.clamp_metrics();
        let id = skill.id;
        self.skills.insert(id, skill.clone());

        // Index tags.
        for tag in skill
            .tags
            .iter()
            .map(|t| t.trim().to_ascii_lowercase())
            .filter(|t| !t.is_empty())
        {
            self.tag_index.entry(tag).or_default().insert(id);
        }
        Ok(())
    }

    pub fn get_skill(&self, id: &Uuid) -> Option<&SkillDefinition> {
        self.skills.get(id)
    }

    pub fn get_skill_mut(&mut self, id: &Uuid) -> Option<&mut SkillDefinition> {
        self.skills.get_mut(id)
    }

    pub fn get_skills_by_categories(&self, categories: &[SkillCategory]) -> Vec<SkillDefinition> {
        if categories.is_empty() {
            return self.skills.values().cloned().collect();
        }
        let wanted: HashSet<SkillCategory> = categories.iter().copied().collect();
        self.skills
            .values()
            .filter(|s| wanted.contains(&s.category))
            .cloned()
            .collect()
    }

    pub fn update_skill_metrics(&mut self, skill_id: &Uuid, result: &SkillResult) -> Result<(), String> {
        let Some(skill) = self.get_skill_mut(skill_id) else {
            return Err("skill not found".to_string());
        };
        skill.usage_count = skill.usage_count.saturating_add(1);
        skill.last_used = Some(Utc::now());
        // Exponential moving average, soft update.
        let alpha = 0.12;
        skill.love_score = (1.0 - alpha) * skill.love_score + alpha * result.love_score;
        skill.utility_score = (1.0 - alpha) * skill.utility_score + alpha * result.utility_score;
        // success_rate tracks success boolean.
        let s = if result.success { 1.0 } else { 0.0 };
        skill.success_rate = (1.0 - alpha) * skill.success_rate + alpha * s;
        skill.clamp_metrics();
        Ok(())
    }

    pub fn find_relevant_skills(&self, context: &SkillContext) -> Vec<SkillSuggestion> {
        let input_lc = context.user_input.to_ascii_lowercase();

        // Basic tag matching + weighted by metrics.
        let mut candidate_ids: HashSet<Uuid> = HashSet::new();
        for token in input_lc
            .split(|c: char| !c.is_ascii_alphanumeric())
            .map(|s| s.trim())
            .filter(|s| s.len() >= 3)
        {
            if let Some(ids) = self.tag_index.get(token) {
                candidate_ids.extend(ids.iter().copied());
            }
        }

        // Fallback: if no tag matches, return top skills by love+utility.
        if candidate_ids.is_empty() {
            let mut skills = self.skills.values().collect::<Vec<_>>();
            skills.sort_by(|a, b| {
                (b.love_score + b.utility_score)
                    .partial_cmp(&(a.love_score + a.utility_score))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            return skills
                .into_iter()
                .take(6)
                .map(|s| SkillSuggestion {
                    skill_id: s.id,
                    skill_name: s.name.clone(),
                    relevance_score: (s.love_score * 0.55 + s.utility_score * 0.45).clamp(0.0, 1.0),
                    reason: "top_skill_by_score".to_string(),
                })
                .collect();
        }

        let mut scored = Vec::new();
        for id in candidate_ids {
            let Some(s) = self.skills.get(&id) else { continue };
            let base = 0.35;
            let metric = (s.love_score * 0.55 + s.utility_score * 0.45).clamp(0.0, 1.0);
            let relevance = (base + (metric * 0.65)).clamp(0.0, 1.0);
            scored.push((relevance, s));
        }

        scored.sort_by(|(a, _), (b, _)| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        scored
            .into_iter()
            .take(8)
            .map(|(score, s)| SkillSuggestion {
                skill_id: s.id,
                skill_name: s.name.clone(),
                relevance_score: score,
                reason: "tag_match".to_string(),
            })
            .collect()
    }
}

