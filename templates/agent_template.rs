// templates/agent_template.rs
// Template version: 1.0.0
//
// This file is a *scaffold* intended to be copied into new agent repos.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionEntry {
    pub ts_unix: i64,
    pub change_type: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatedAgent {
    pub name: String,
    pub version: String,
    pub template_version: String,
    pub creator: String,


    /// Optional zodiac sign override for this agent.
    ///
    /// Inheritance rule:
    /// - `None` => inherit the queen/Phoenix base sign.
    /// - `Some(sign)` => use `sign` as the override.
    ///
    /// Representation:
    /// - Stored as a string to keep this template repo-agnostic.
    /// - Expected values: one of `Aries|Taurus|Gemini|Cancer|Leo|Virgo|Libra|Scorpio|Sagittarius|Capricorn|Aquarius|Pisces`
    ///   (case-insensitive; callers may canonicalize).
    ///
    /// Note on utility agents:
    /// If this agent is intended to be a “utility agent” (tooling/ops), callers should treat zodiac as
    /// *flavor only* (e.g., communication style bias) rather than a full personality copy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zodiac_sign: Option<String>,

    #[serde(default)]
    pub evolution_history: Vec<EvolutionEntry>,
    #[serde(default)]
    pub telemetry: HashMap<String, f64>,
    pub playbook_version: u32,
}

impl TemplatedAgent {
    pub fn new(name: &str, creator: &str) -> Self {
        let ts_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        Self {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            template_version: "1.0.0".to_string(),
            creator: creator.to_string(),
            zodiac_sign: None,
            evolution_history: vec![EvolutionEntry {
                ts_unix,
                change_type: "creation".to_string(),
                reason: "bootstrapped_from_template".to_string(),
            }],
            telemetry: HashMap::new(),
            playbook_version: 1,
        }
    }

    /// Resolve this agent's effective zodiac sign by applying inheritance.
    ///
    /// Rule: `self.zodiac_sign` overrides; otherwise inherit the provided `phoenix_base_sign`.
    pub fn effective_zodiac_sign(&self, phoenix_base_sign: &str) -> String {
        match self.zodiac_sign.as_deref() {
            Some(s) if !s.trim().is_empty() => s.trim().to_string(),
            _ => phoenix_base_sign.trim().to_string(),
        }
    }

    pub fn record_metric(&mut self, key: &str, value: f64) {
        self.telemetry.insert(key.to_string(), value);
    }
}

