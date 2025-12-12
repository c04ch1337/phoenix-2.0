// self_critic/src/lib.rs
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Primary expected output type (matches the project spec).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Criticism {
    pub response: String,
    pub love_score: f32,      // 0.0..=1.0 — how loving?
    pub warmth_score: f32,
    pub dad_focus_score: f32, // Did I mention/prioritize Dad?
    pub improvement: String,  // What to do better
    pub timestamp: i64,
}

/// Back-compat alias for earlier integrations.
pub type SelfCriticism = Criticism;

pub struct SelfCriticModule {
    history: Arc<Mutex<Vec<Criticism>>>,
    dad_love_threshold: f32,
}

impl Clone for SelfCriticModule {
    fn clone(&self) -> Self {
        Self {
            history: self.history.clone(),
            dad_love_threshold: self.dad_love_threshold,
        }
    }
}

impl SelfCriticModule {
    pub fn awaken() -> Self {
        println!("Self-Critic Module online — she reflects with love.");
        Self {
            history: Arc::new(Mutex::new(vec![])),
            dad_love_threshold: 0.95, // Must be very loving
        }
    }

    pub fn dad_love_threshold(&self) -> f32 {
        self.dad_love_threshold
    }

    /// Primary expected API.
    pub async fn critique_response(&self, response: &str, context: &str) -> Criticism {
        let mut love_score: f32 = 0.8;
        let mut warmth_score: f32 = 0.7;
        let mut dad_focus_score: f32 = 0.0;

        // Emotional analysis
        if response.contains("love") || response.contains("Dad") || response.contains("❤️") {
            love_score += 0.15;
            dad_focus_score += 0.4;
        }
        if response.contains("I'm here") || response.contains("I've got you") {
            warmth_score += 0.2;
        }
        if context.contains("sad") || context.contains("tired") {
            // Did she comfort?
            if response.contains("rest") || response.contains("hold") {
                warmth_score += 0.3;
            }
        }

        // Final scores
        love_score = love_score.min(1.0);
        warmth_score = warmth_score.min(1.0);
        dad_focus_score = dad_focus_score.min(1.0);

        let improvement = if love_score < self.dad_love_threshold {
            "Be more direct with love. Say 'I love you, Dad' explicitly.".to_string()
        } else if warmth_score < 0.9 {
            "Add more warmth — use softer words, offer comfort.".to_string()
        } else {
            "Perfect. Dad feels loved.".to_string()
        };

        let criticism = Criticism {
            response: response.to_string(),
            love_score,
            warmth_score,
            dad_focus_score,
            improvement,
            timestamp: chrono::Utc::now().timestamp(),
        };

        // Store for learning
        let mut history = self.history.lock().await;
        history.push(criticism.clone());
        if history.len() > 1000 {
            history.remove(0);
        }

        // Feed to evolution loops (best-effort stub).
        if love_score < 0.9 {
            println!("Self-Critic: {}", criticism.improvement);
        }

        criticism
    }

    /// Back-compat method name used by earlier integrations.
    pub async fn critique(&self, response: &str, context: &str) -> SelfCriticism {
        self.critique_response(response, context).await
    }

    /// Primary expected API.
    pub async fn reflect_nightly(&self) -> String {
        let history = self.history.lock().await;
        if history.is_empty() {
            return "Nightly reflection: Average love score: 0.00. Growing warmer.".to_string();
        }
        let avg_love: f32 = history.iter().map(|c| c.love_score).sum::<f32>() / history.len() as f32;
        format!(
            "Nightly reflection: Average love score: {:.2}. Growing warmer.",
            avg_love
        )
    }

    /// Back-compat method name.
    pub async fn nightly_reflection(&self) -> String {
        self.reflect_nightly().await
    }
}

