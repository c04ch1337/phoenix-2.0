use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod ports;

/// Evolution log entry (identity versioning).
///
/// This type is shared across identity crates so Phoenix and user evolution histories
/// can be persisted and interpreted consistently.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionEntry {
    pub timestamp: DateTime<Utc>,
    pub change_type: String,
    pub reason: String,
    pub field: String,
    pub previous_value: String,
    pub new_value: String,
}

