use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedMemory {
    pub ts: DateTime<Utc>,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    /// 0.0..=1.0 — stronger memories should surface more easily.
    pub emotional_weight: f32,
}

impl SharedMemory {
    pub fn new(title: impl Into<String>, content: impl Into<String>, tags: Vec<String>, emotional_weight: f32) -> Self {
        Self {
            ts: Utc::now(),
            title: title.into(),
            content: content.into(),
            tags,
            emotional_weight: emotional_weight.clamp(0.0, 1.0),
        }
    }

    pub fn relevance_score(&self, input: &str) -> f32 {
        let input_lc = input.to_ascii_lowercase();
        let mut score = self.emotional_weight * 0.40;

        if !self.title.is_empty() && input_lc.contains(&self.title.to_ascii_lowercase()) {
            score += 0.35;
        }
        for t in &self.tags {
            let tl = t.to_ascii_lowercase();
            if !tl.is_empty() && input_lc.contains(&tl) {
                score += 0.15;
            }
        }
        if !self.content.is_empty() {
            let c = self.content.to_ascii_lowercase();
            // Cheap overlap heuristic.
            if c.split_whitespace().any(|w| w.len() > 4 && input_lc.contains(w)) {
                score += 0.15;
            }
        }
        score.clamp(0.0, 1.0)
    }
}

