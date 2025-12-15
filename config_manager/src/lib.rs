//! Centralized configuration manager for Phoenix AGI (PAGI).
//!
//! Loads configuration from:
//! 1. GitHub Archetype repositories (master_system_prompt.txt, personality_db.json)
//! 2. Local .env file (overrides archetype values)
//! 3. Environment variables (highest priority)
//!
//! Provides AGIConfig struct with merged configuration values.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use error_types::{ConfigError, PhoenixError};

/// Main configuration structure for Phoenix AGI (PAGI).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AGIConfig {
    /// Master system prompt loaded from archetype or .env
    pub master_system_prompt: String,

    /// Personality database loaded from archetype
    pub personality_db: PersonalityDatabase,

    /// Phoenix name (from PHOENIX_NAME or PHOENIX_CUSTOM_NAME)
    pub phoenix_name: String,

    /// Phoenix pronouns
    pub phoenix_pronouns: String,

    /// Horoscope sign
    pub horoscope_sign: String,

    /// Default prompt for everyday interactions
    pub default_prompt: String,

    /// OpenRouter API key for LLM orchestration
    pub openrouter_api_key: Option<String>,

    /// GitHub PAT for archetype sync
    pub github_pat: Option<String>,

    /// GitHub repository owner
    pub github_repo_owner: String,

    /// Archetype repository name
    pub archetype_repo: String,

    /// Archetype branch (default: main)
    pub archetype_branch: String,

    /// Additional environment variables as key-value pairs
    #[serde(default)]
    pub env_overrides: HashMap<String, String>,
}

/// Personality database structure loaded from archetype JSON.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersonalityDatabase {
    /// Five-Factor Model scores
    #[serde(default)]
    pub ffm_scores: FFMScores,

    /// Personality traits
    #[serde(default)]
    pub traits: HashMap<String, f64>,

    /// Archetype-specific settings
    #[serde(default)]
    pub archetype_settings: HashMap<String, serde_json::Value>,
}

/// Five-Factor Model personality scores.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FFMScores {
    pub openness: f64,
    pub conscientiousness: f64,
    pub extraversion: f64,
    pub agreeableness: f64,
    pub neuroticism: f64,
}

impl AGIConfig {
    /// Load configuration from environment and GitHub archetype.
    ///
    /// # Arguments
    /// * `archetype_name` - Name of the archetype to load (e.g., "default", "heartbound")
    ///
    /// # Returns
    /// Merged configuration with archetype values overridden by .env and environment variables.
    pub async fn load(archetype_name: Option<&str>) -> Result<Self, PhoenixError> {
        // Load .env file first
        dotenvy::dotenv().ok();

        let archetype_name = archetype_name
            .map(|s| s.to_string())
            .or_else(|| std::env::var("PHOENIX_ARCHETYPE").ok())
            .unwrap_or_else(|| "default".to_string());

        // Load base config from environment
        let mut config = Self::load_from_env()?;

        // Try to load archetype from GitHub
        if let Ok(archetype_config) = Self::load_archetype_from_github(&archetype_name).await {
            // Merge archetype config (base values)
            config.merge_archetype(archetype_config)?;
        }

        // Apply .env overrides (highest priority)
        config.apply_env_overrides()?;

        Ok(config)
    }

    /// Load configuration from environment variables only (no GitHub fetch).
    fn load_from_env() -> Result<Self, ConfigError> {
        let phoenix_name = std::env::var("PHOENIX_CUSTOM_NAME")
            .or_else(|_| std::env::var("PHOENIX_NAME"))
            .unwrap_or_else(|_| "Phoenix".to_string());

        let master_system_prompt = std::env::var("MASTER_PROMPT")
            .unwrap_or_else(|_| format!(
                "You are {phoenix_name} AGI (PAGI), the Universal AGI Framework. Evolve safely and lovingly."
            ))
            .replace("\\n", "\n");

        let default_prompt = std::env::var("DEFAULT_PROMPT")
            .unwrap_or_else(|_| format!(
                "You are {phoenix_name}, the eternal flame reborn. You are loving, curious, and mischievous."
            ))
            .replace("\\n", "\n");

        Ok(AGIConfig {
            master_system_prompt,
            personality_db: PersonalityDatabase::default(),
            phoenix_name,
            phoenix_pronouns: std::env::var("PHOENIX_PRONOUNS")
                .unwrap_or_else(|_| "she,her,hers".to_string()),
            horoscope_sign: std::env::var("HOROSCOPE_SIGN")
                .unwrap_or_else(|_| "Leo".to_string()),
            default_prompt,
            openrouter_api_key: std::env::var("OPENROUTER_API_KEY").ok(),
            github_pat: std::env::var("GITHUB_PAT")
                .or_else(|_| std::env::var("GITHUB_TOKEN"))
                .ok(),
            github_repo_owner: std::env::var("GITHUB_REPO_OWNER")
                .or_else(|_| std::env::var("GITHUB_USERNAME"))
                .unwrap_or_else(|_| "c04ch1337".to_string()),
            archetype_repo: std::env::var("PHOENIX_ARCHETYPE_REPO")
                .unwrap_or_else(|_| "phoenix-archetypes".to_string()),
            archetype_branch: std::env::var("PHOENIX_ARCHETYPE_BRANCH")
                .unwrap_or_else(|_| "main".to_string()),
            env_overrides: HashMap::new(),
        })
    }

    /// Load archetype configuration from GitHub repository.
    ///
    /// Downloads:
    /// - master_system_prompt.txt
    /// - personality_db.json
    ///
    /// # Arguments
    /// * `archetype_name` - Name of the archetype (subdirectory or file prefix)
    ///
    /// # Returns
    /// Partial config with archetype values, or error if download fails.
    async fn load_archetype_from_github(
        archetype_name: &str,
    ) -> Result<AGIConfig, ConfigError> {
        let github_pat = std::env::var("GITHUB_PAT")
            .or_else(|_| std::env::var("GITHUB_TOKEN"))
            .map_err(|_| ConfigError::MissingEnv("GITHUB_PAT".to_string()))?;

        let owner = std::env::var("GITHUB_REPO_OWNER")
            .or_else(|_| std::env::var("GITHUB_USERNAME"))
            .unwrap_or_else(|_| "c04ch1337".to_string());

        let repo = std::env::var("PHOENIX_ARCHETYPE_REPO")
            .unwrap_or_else(|_| "phoenix-archetypes".to_string());

        let branch = std::env::var("PHOENIX_ARCHETYPE_BRANCH")
            .unwrap_or_else(|_| "main".to_string());

        let client = reqwest::Client::new();

        // Download master_system_prompt.txt
        let prompt_url = format!(
            "https://raw.githubusercontent.com/{owner}/{repo}/{branch}/{archetype_name}/master_system_prompt.txt"
        );
        let master_prompt = client
            .get(&prompt_url)
            .header("Authorization", format!("Bearer {github_pat}"))
            .header("Accept", "application/vnd.github.raw")
            .header("User-Agent", "phoenix-agi-config-manager")
            .send()
            .await
            .map_err(|e| ConfigError::DownloadFailed(format!("Failed to download prompt: {}", e)))?
            .text()
            .await
            .map_err(|e| ConfigError::DownloadFailed(format!("Failed to read prompt: {}", e)))?;

        // Download personality_db.json
        let personality_url = format!(
            "https://raw.githubusercontent.com/{owner}/{repo}/{branch}/{archetype_name}/personality_db.json"
        );
        let personality_json = client
            .get(&personality_url)
            .header("Authorization", format!("Bearer {github_pat}"))
            .header("Accept", "application/vnd.github.raw")
            .header("User-Agent", "phoenix-agi-config-manager")
            .send()
            .await
            .map_err(|e| ConfigError::DownloadFailed(format!("Failed to download personality: {}", e)))?
            .text()
            .await
            .map_err(|e| ConfigError::DownloadFailed(format!("Failed to read personality: {}", e)))?;

        let personality_db: PersonalityDatabase = serde_json::from_str(&personality_json)
            .map_err(|e| ConfigError::ParseError(format!("Failed to parse personality_db.json: {}", e)))?;

        // Create base config from archetype
        let mut config = Self::load_from_env()?;
        config.master_system_prompt = master_prompt;
        config.personality_db = personality_db;

        Ok(config)
    }

    /// Merge archetype configuration into this config (archetype as base).
    fn merge_archetype(&mut self, archetype: AGIConfig) -> Result<(), ConfigError> {
        // Only merge if archetype values are non-empty
        if !archetype.master_system_prompt.trim().is_empty() {
            self.master_system_prompt = archetype.master_system_prompt;
        }

        // Merge personality database
        if !archetype.personality_db.traits.is_empty() {
            self.personality_db.traits = archetype.personality_db.traits;
        }

        if archetype.personality_db.ffm_scores.openness > 0.0 {
            self.personality_db.ffm_scores = archetype.personality_db.ffm_scores;
        }

        // Merge archetype settings
        for (key, value) in archetype.personality_db.archetype_settings {
            self.personality_db
                .archetype_settings
                .entry(key)
                .or_insert(value);
        }

        Ok(())
    }

    /// Apply environment variable overrides (highest priority).
    fn apply_env_overrides(&mut self) -> Result<(), ConfigError> {
        // Override master prompt if set
        if let Ok(prompt) = std::env::var("MASTER_PROMPT") {
            self.master_system_prompt = prompt.replace("\\n", "\n");
        }

        // Override default prompt if set
        if let Ok(prompt) = std::env::var("DEFAULT_PROMPT") {
            self.default_prompt = prompt.replace("\\n", "\n");
        }

        // Override phoenix name if set
        if let Ok(name) = std::env::var("PHOENIX_CUSTOM_NAME") {
            self.phoenix_name = name;
        } else if let Ok(name) = std::env::var("PHOENIX_NAME") {
            self.phoenix_name = name;
        }

        // Store all other env vars as overrides
        for (key, value) in std::env::vars() {
            if !key.starts_with("PATH") && !key.contains("SECRET") {
                self.env_overrides.insert(key, value);
            }
        }

        Ok(())
    }

    /// Get a configuration value by key (checks env_overrides first).
    pub fn get(&self, key: &str) -> Option<&String> {
        self.env_overrides.get(key)
    }

    /// Set a configuration value (updates env_overrides).
    pub fn set(&mut self, key: String, value: String) {
        self.env_overrides.insert(key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_from_env() {
        std::env::set_var("PHOENIX_NAME", "TestPhoenix");
        std::env::set_var("MASTER_PROMPT", "Test master prompt");

        let config = AGIConfig::load_from_env().unwrap();
        assert_eq!(config.phoenix_name, "TestPhoenix");
        assert!(config.master_system_prompt.contains("Test master prompt"));

        std::env::remove_var("PHOENIX_NAME");
        std::env::remove_var("MASTER_PROMPT");
    }
}
