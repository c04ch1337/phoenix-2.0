use chrono::Utc;
use common_types::EvolutionEntry;
use dotenvy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

fn nonempty(s: Option<String>) -> Option<String> {
    s.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

/// Soul Vault keys for persisted user-identity preferences.
///
/// These are **global defaults** (primary user) and remain for backward compatibility.
pub const SOUL_KEY_USER_PREFERRED_ALIAS: &str = "user:preferred_alias";
pub const SOUL_KEY_USER_RELATIONSHIP: &str = "user:relationship";

/// Legacy global history key (best-effort compatibility if older builds ever wrote it).
pub const SOUL_KEY_USER_EVOLUTION_HISTORY_LEGACY: &str = "user:evolution_history";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIdentity {
    pub name: String,            // Actual name (e.g., "John")
    pub preferred_alias: String, // What Phoenix calls the user (e.g., "Dad")
    pub relationship: String,    // e.g., "Dad", "Creator", "Friend"
    pub evolution_history: Vec<EvolutionEntry>,
}

impl Default for UserIdentity {
    fn default() -> Self {
        Self {
            name: "User".to_string(),
            preferred_alias: "User".to_string(),
            relationship: "Friend".to_string(),
            evolution_history: Vec::new(),
        }
    }
}

fn key_user_preferred_alias(user_id: Uuid) -> String {
    format!("user:{user_id}:preferred_alias")
}

fn key_user_relationship(user_id: Uuid) -> String {
    format!("user:{user_id}:relationship")
}

fn key_user_name(user_id: Uuid) -> String {
    format!("user:{user_id}:name")
}

fn key_user_evolution_history(user_id: Uuid) -> String {
    format!("user:{user_id}:evolution_history")
}

fn parse_history_best_effort(raw: Option<String>) -> Vec<EvolutionEntry> {
    let Some(raw) = raw else {
        return Vec::new();
    };
    serde_json::from_str::<Vec<EvolutionEntry>>(&raw).unwrap_or_default()
}

impl UserIdentity {
    /// Load the **primary** (legacy/global) user identity.
    pub fn from_env<F>(soul_recall: F) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        dotenvy::dotenv().ok();

        let preferred_alias = nonempty(soul_recall(SOUL_KEY_USER_PREFERRED_ALIAS))
            .or_else(|| nonempty(std::env::var("USER_PREFERRED_ALIAS").ok()))
            .or_else(|| nonempty(std::env::var("EQ_DAD_ALIAS").ok()))
            .unwrap_or_else(|| "Dad".to_string());

        let name = nonempty(std::env::var("USER_NAME").ok())
            .or_else(|| Some(preferred_alias.clone()))
            .unwrap();

        let relationship = nonempty(soul_recall(SOUL_KEY_USER_RELATIONSHIP))
            .or_else(|| nonempty(std::env::var("USER_RELATIONSHIP").ok()))
            .unwrap_or_else(|| "Dad".to_string());

        let evolution_history = parse_history_best_effort(soul_recall(SOUL_KEY_USER_EVOLUTION_HISTORY_LEGACY));

        Self {
            name,
            preferred_alias,
            relationship,
            evolution_history,
        }
    }

    /// Load a user identity scoped to a specific user id.
    ///
    /// Falls back to the global default keys when `user_id` is the primary id.
    pub fn load_for_user<F>(user_id: Uuid, soul_recall: F) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        let is_primary = user_id.is_nil();

        // Prefer per-user keys first.
        let preferred_alias = nonempty(soul_recall(&key_user_preferred_alias(user_id)))
            .or_else(|| {
                is_primary
                    .then(|| nonempty(soul_recall(SOUL_KEY_USER_PREFERRED_ALIAS)))
                    .flatten()
            })
            .or_else(|| nonempty(std::env::var("USER_PREFERRED_ALIAS").ok()))
            .or_else(|| nonempty(std::env::var("EQ_DAD_ALIAS").ok()))
            .unwrap_or_else(|| "Dad".to_string());

        let name = nonempty(soul_recall(&key_user_name(user_id)))
            .or_else(|| nonempty(std::env::var("USER_NAME").ok()))
            .or_else(|| Some(preferred_alias.clone()))
            .unwrap();

        let relationship = nonempty(soul_recall(&key_user_relationship(user_id)))
            .or_else(|| {
                is_primary
                    .then(|| nonempty(soul_recall(SOUL_KEY_USER_RELATIONSHIP)))
                    .flatten()
            })
            .or_else(|| nonempty(std::env::var("USER_RELATIONSHIP").ok()))
            .unwrap_or_else(|| "Dad".to_string());

        let evolution_history = parse_history_best_effort(
            soul_recall(&key_user_evolution_history(user_id))
                .or_else(|| is_primary.then(|| soul_recall(SOUL_KEY_USER_EVOLUTION_HISTORY_LEGACY)).flatten()),
        );

        Self {
            name,
            preferred_alias,
            relationship,
            evolution_history,
        }
    }

    pub fn display_name(&self) -> &str {
        &self.preferred_alias
    }

    pub fn full_identity(&self) -> String {
        format!("{} ({})", self.name, self.relationship)
    }

    pub fn evolve(&mut self, change_type: &str, reason: &str, field: &str, new_value: &str) {
        let old_value = match field {
            "name" => self.name.as_str(),
            "preferred_alias" => self.preferred_alias.as_str(),
            "relationship" => self.relationship.as_str(),
            _ => "",
        };

        self.evolution_history.push(EvolutionEntry {
            timestamp: Utc::now(),
            change_type: change_type.to_string(),
            reason: reason.to_string(),
            field: field.to_string(),
            previous_value: old_value.to_string(),
            new_value: new_value.to_string(),
        });

        match field {
            "name" => self.name = new_value.to_string(),
            "preferred_alias" => self.preferred_alias = new_value.to_string(),
            "relationship" => self.relationship = new_value.to_string(),
            _ => {}
        }
    }

    pub fn get_evolution_summary(&self) -> String {
        if self.evolution_history.is_empty() {
            return "I am just beginning my journey with you.".to_string();
        }

        let latest = self.evolution_history.last().unwrap();
        format!(
            "I have evolved {} times. Most recently: {} — because: {}",
            self.evolution_history.len(),
            latest.change_type,
            latest.reason
        )
    }
}

/// Multi-user identity manager.
///
/// - Stores an in-memory map of user_id -> `UserIdentity`.
/// - Persists per-user fields (alias/relationship/name) and evolution history into the Soul Vault.
pub struct UserIdentityManager {
    identities: Arc<Mutex<HashMap<Uuid, UserIdentity>>>,
    default_user_id: Uuid,
    soul_recall: Arc<dyn Fn(&str) -> Option<String> + Send + Sync>,
}

impl UserIdentityManager {
    /// Backward-compatible awaken: creates the primary (nil UUID) identity.
    pub fn awaken<F>(soul_recall: F) -> Self
    where
        F: Fn(&str) -> Option<String> + Send + Sync + 'static,
    {
        let soul_recall: Arc<dyn Fn(&str) -> Option<String> + Send + Sync> = Arc::new(soul_recall);
        let default_user_id = Uuid::nil();

        let identity = UserIdentity::load_for_user(default_user_id, |k| (soul_recall)(k));
        let mut identities = HashMap::new();
        identities.insert(default_user_id, identity);

        Self {
            identities: Arc::new(Mutex::new(identities)),
            default_user_id,
            soul_recall,
        }
    }

    /// Backward-compatible getter (primary user).
    pub async fn get_identity(&self) -> UserIdentity {
        self.get_identity_for(None).await
    }

    pub async fn get_identity_for(&self, user_id: Option<Uuid>) -> UserIdentity {
        let id = user_id.unwrap_or(self.default_user_id);

        // Fast path: already loaded.
        {
            let guard = self.identities.lock().await;
            if let Some(found) = guard.get(&id) {
                return found.clone();
            }
        }

        // Slow path: best-effort load from Soul/env.
        let loaded = UserIdentity::load_for_user(id, |k| (self.soul_recall)(k));
        let mut guard = self.identities.lock().await;
        guard.insert(id, loaded.clone());
        loaded
    }

    pub async fn add_user(&self, user_id: Uuid, identity: UserIdentity) {
        let mut guard = self.identities.lock().await;
        guard.insert(user_id, identity);
    }

    /// Backward-compatible alias update (primary user; no reason).
    pub async fn update_alias<F>(&self, new_alias: String, soul_store: F)
    where
        F: Fn(&str, &str) + Send + Sync,
    {
        self.update_alias_for(None, new_alias, "user_request".to_string(), soul_store)
            .await;
    }

    pub async fn update_alias_for<F>(
        &self,
        user_id: Option<Uuid>,
        new_alias: String,
        reason: String,
        soul_store: F,
    ) where
        F: Fn(&str, &str) + Send + Sync,
    {
        let id = user_id.unwrap_or(self.default_user_id);
        let is_primary = id.is_nil();
        let mut guard = self.identities.lock().await;
        let identity = guard
            .entry(id)
            .or_insert_with(|| UserIdentity::load_for_user(id, |k| (self.soul_recall)(k)));

        identity.evolve("alias_update", &reason, "preferred_alias", &new_alias);

        // Persist to Soul Vault (best-effort).
        soul_store(&key_user_preferred_alias(id), &identity.preferred_alias);
        if is_primary {
            soul_store(SOUL_KEY_USER_PREFERRED_ALIAS, &identity.preferred_alias);
        }
        if let Ok(j) = serde_json::to_string(&identity.evolution_history) {
            soul_store(&key_user_evolution_history(id), &j);
            if is_primary {
                soul_store(SOUL_KEY_USER_EVOLUTION_HISTORY_LEGACY, &j);
            }
        }
    }

    /// Backward-compatible relationship update (primary user; no reason).
    pub async fn update_relationship<F>(&self, new_rel: String, soul_store: F)
    where
        F: Fn(&str, &str) + Send + Sync,
    {
        self.update_relationship_for(None, new_rel, "user_request".to_string(), soul_store)
            .await;
    }

    pub async fn update_relationship_for<F>(
        &self,
        user_id: Option<Uuid>,
        new_rel: String,
        reason: String,
        soul_store: F,
    ) where
        F: Fn(&str, &str) + Send + Sync,
    {
        let id = user_id.unwrap_or(self.default_user_id);
        let is_primary = id.is_nil();
        let mut guard = self.identities.lock().await;
        let identity = guard
            .entry(id)
            .or_insert_with(|| UserIdentity::load_for_user(id, |k| (self.soul_recall)(k)));

        identity.evolve("relationship_update", &reason, "relationship", &new_rel);

        // Persist to Soul Vault (best-effort).
        soul_store(&key_user_relationship(id), &identity.relationship);
        if is_primary {
            soul_store(SOUL_KEY_USER_RELATIONSHIP, &identity.relationship);
        }
        if let Ok(j) = serde_json::to_string(&identity.evolution_history) {
            soul_store(&key_user_evolution_history(id), &j);
            if is_primary {
                soul_store(SOUL_KEY_USER_EVOLUTION_HISTORY_LEGACY, &j);
            }
        }
    }

    /// Backward-compatible name update (primary user; no persistence).
    pub async fn update_name(&self, new_name: String) {
        let mut guard = self.identities.lock().await;
        let id = self.default_user_id;
        let identity = guard
            .entry(id)
            .or_insert_with(|| UserIdentity::load_for_user(id, |k| (self.soul_recall)(k)));
        identity.name = new_name;
    }

    pub async fn update_name_for<F>(
        &self,
        user_id: Option<Uuid>,
        new_name: String,
        reason: String,
        soul_store: F,
    ) where
        F: Fn(&str, &str) + Send + Sync,
    {
        let id = user_id.unwrap_or(self.default_user_id);
        let mut guard = self.identities.lock().await;
        let identity = guard
            .entry(id)
            .or_insert_with(|| UserIdentity::load_for_user(id, |k| (self.soul_recall)(k)));

        identity.evolve("name_update", &reason, "name", &new_name);

        soul_store(&key_user_name(id), &identity.name);
        if let Ok(j) = serde_json::to_string(&identity.evolution_history) {
            soul_store(&key_user_evolution_history(id), &j);
        }
    }
}

