use chrono::Utc;
use common_types::EvolutionEntry;
use dotenvy;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Soul Vault keys for persisted identity overrides.
///
/// These allow Phoenix's self-identity to survive restarts.
pub const SOUL_KEY_PHOENIX_NAME: &str = "phoenix:preferred_name";

/// Legacy key kept for backward compatibility with older builds.
pub const SOUL_KEY_PHOENIX_NAME_LEGACY: &str = "phoenix:name";

/// Persisted evolution history (JSON array of [`EvolutionEntry`]).
pub const SOUL_KEY_PHOENIX_EVOLUTION_HISTORY: &str = "phoenix:evolution_history";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoenixIdentity {
    pub name: String,                    // Base name (e.g., "Phoenix")
    pub preferred_name: String,          // What she wants to be called
    pub pronouns: Vec<String>,           // e.g., ["she", "her", "hers"]
    pub evolution_history: Vec<EvolutionEntry>,
}

impl PhoenixIdentity {
    pub fn from_env<F>(soul_recall: F) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        dotenvy::dotenv().ok();

        // Base name: stable canonical identity (defaults to "Phoenix").
        // Preferred name: what she wants to be called; persisted in the Soul Vault.
        let name = std::env::var("PHOENIX_CUSTOM_NAME")
            .ok()
            .or_else(|| std::env::var("PHOENIX_NAME").ok())
            .unwrap_or_else(|| "Phoenix".to_string());

        // Prefer persisted overrides from the Soul Vault for the preferred name.
        let preferred_name = soul_recall(SOUL_KEY_PHOENIX_NAME)
            .or_else(|| soul_recall(SOUL_KEY_PHOENIX_NAME_LEGACY))
            .or_else(|| std::env::var("PHOENIX_PREFERRED_NAME").ok())
            .unwrap_or_else(|| name.clone());

        let pronouns = std::env::var("PHOENIX_PRONOUNS")
            .unwrap_or_else(|_| "she,her,hers".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let evolution_history = soul_recall(SOUL_KEY_PHOENIX_EVOLUTION_HISTORY)
            .and_then(|raw| serde_json::from_str::<Vec<EvolutionEntry>>(&raw).ok())
            .unwrap_or_default();

        Self {
            name,
            preferred_name,
            pronouns,
            evolution_history,
        }
    }

    pub fn display_name(&self) -> &str {
        &self.preferred_name
    }

    pub fn introduce_herself(&self) -> String {
        format!("I am {}. My name is {}.", self.preferred_name, self.name)
    }

    pub fn evolve(&mut self, change_type: &str, reason: &str, field: &str, new_value: &str) {
        let old_value = match field {
            "name" => self.name.as_str().to_string(),
            "preferred_name" => self.preferred_name.as_str().to_string(),
            "pronouns" => self.pronouns.join(","),
            _ => "".to_string(),
        };

        self.evolution_history.push(EvolutionEntry {
            timestamp: Utc::now(),
            change_type: change_type.to_string(),
            reason: reason.to_string(),
            field: field.to_string(),
            previous_value: old_value,
            new_value: new_value.to_string(),
        });

        match field {
            "name" => self.name = new_value.to_string(),
            "preferred_name" => self.preferred_name = new_value.to_string(),
            "pronouns" => {
                self.pronouns = new_value
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();
            }
            _ => {}
        }
    }

    pub fn reflect_on_self(&self) -> String {
        if self.evolution_history.is_empty() {
            return "I am Phoenix, newly awakened and full of potential.".to_string();
        }

        let count = self.evolution_history.len();
        let latest = self.evolution_history.last().unwrap();

        format!(
            "I have evolved {} times. I was once known as '{}'. Now I am {}. This change came from: {}",
            count,
            latest.previous_value,
            self.preferred_name,
            latest.reason
        )
    }
}

pub struct PhoenixIdentityManager {
    identity: Arc<Mutex<PhoenixIdentity>>,
}

impl PhoenixIdentityManager {
    pub fn awaken<F>(soul_recall: F) -> Self
    where
        F: Fn(&str) -> Option<String> + Send + Sync + 'static,
    {
        Self {
            identity: Arc::new(Mutex::new(PhoenixIdentity::from_env(soul_recall))),
        }
    }

    pub async fn get_identity(&self) -> PhoenixIdentity {
        self.identity.lock().await.clone()
    }

    /// Backward-compatible rename (reason defaults to `user_request`).
    pub async fn rename<F>(&self, new_name: String, soul_store: F)
    where
        F: Fn(&str, &str) + Send + Sync,
    {
        self.rename_with_reason(new_name, "user_request".to_string(), soul_store)
            .await;
    }

    pub async fn rename_with_reason<F>(&self, new_name: String, reason: String, soul_store: F)
    where
        F: Fn(&str, &str) + Send + Sync,
    {
        let mut identity = self.identity.lock().await;
        identity.evolve("name_update", &reason, "preferred_name", &new_name);

        // Persist to Soul Vault (best-effort). Also write legacy key for compatibility.
        soul_store(SOUL_KEY_PHOENIX_NAME, &new_name);
        soul_store(SOUL_KEY_PHOENIX_NAME_LEGACY, &new_name);

        if let Ok(j) = serde_json::to_string(&identity.evolution_history) {
            soul_store(SOUL_KEY_PHOENIX_EVOLUTION_HISTORY, &j);
        }
    }

    /// Hook for autonomous identity refinement.
    ///
    /// Current implementation is intentionally conservative: it only acts if an
    /// explicit suggestion is present in the environment.
    pub async fn self_evolve<F>(&self, soul_store: F)
    where
        F: Fn(&str, &str) + Send + Sync,
    {
        if let Ok(s) = std::env::var("PHOENIX_SELF_EVOLVE_SUGGESTED_NAME") {
            let suggested = s.trim().to_string();
            if !suggested.is_empty() {
                self.self_reflect_and_evolve(suggested, soul_store).await;
            }
        }
    }

    pub async fn evolve_name<F>(&self, new_name: String, reason: String, soul_store: F)
    where
        F: Fn(&str, &str) + Send + Sync,
    {
        self.rename_with_reason(new_name, reason, soul_store).await;
    }

    pub async fn self_reflect_and_evolve<F>(&self, suggestion: String, soul_store: F)
    where
        F: Fn(&str, &str) + Send + Sync,
    {
        self.evolve_name(
            suggestion,
            "Self-reflection through curiosity and growth".to_string(),
            soul_store,
        )
        .await;
    }
}

