use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedGoal {
    pub name: String,
    /// 0.0..=1.0
    pub progress: f64,
    pub last_update: DateTime<Utc>,
}

impl SharedGoal {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            progress: 0.0,
            last_update: Utc::now(),
        }
    }

    pub fn update(&mut self, delta: f64) {
        self.progress = (self.progress + delta).clamp(0.0, 1.0);
        self.last_update = Utc::now();
    }

    pub fn is_complete(&self) -> bool {
        self.progress >= 1.0
    }

    pub fn progress_bar(&self, width: usize) -> String {
        let w = width.max(4);
        let filled = ((self.progress.clamp(0.0, 1.0) * w as f64).round() as usize).min(w);
        let empty = w - filled;
        format!("[{}{}] {:>3}%", "█".repeat(filled), "░".repeat(empty), (self.progress * 100.0).round() as i32)
    }
}

