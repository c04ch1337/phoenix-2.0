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
            evolution_history: vec![EvolutionEntry {
                ts_unix,
                change_type: "creation".to_string(),
                reason: "bootstrapped_from_template".to_string(),
            }],
            telemetry: HashMap::new(),
            playbook_version: 1,
        }
    }

    pub fn record_metric(&mut self, key: &str, value: f64) {
        self.telemetry.insert(key.to_string(), value);
    }
}

