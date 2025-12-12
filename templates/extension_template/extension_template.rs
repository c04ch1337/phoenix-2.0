// templates/extension_template/extension_template.rs
// Template version: 1.0.0

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Standard interface for a Phoenix extension.
///
/// This trait is intentionally minimal: the host (Phoenix/ORCH) can wrap it in WASM,
/// a dynamic library boundary, or a subprocess boundary.
pub trait PhoenixExtension {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;

    /// Called once when the extension is loaded.
    fn init(&mut self) -> Result<(), String> {
        Ok(())
    }

    /// Execute the extension.
    fn execute(&mut self, input: Value) -> Result<Value, String>;

    /// Return a telemetry snapshot for ingestion.
    fn telemetry_report(&self) -> TelemetryReport;

    /// Basic sanity check (should be fast).
    fn self_test(&mut self) -> bool;

    /// Generate a marketplace-style manifest for indexing/billing.
    fn generate_manifest(&self) -> Value {
        serde_json::json!({
            "name": self.name(),
            "version": self.version(),
            "description": self.description(),
            "template_version": "1.0.0",
            "capabilities": [],
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryReport {
    pub template_version: String,
    pub ok: bool,
    pub metrics: std::collections::HashMap<String, f64>,
}

/// Example extension implementation.
pub struct ExampleExtension {
    metrics: std::collections::HashMap<String, f64>,
}

impl Default for ExampleExtension {
    fn default() -> Self {
        Self {
            metrics: std::collections::HashMap::new(),
        }
    }
}

impl PhoenixExtension for ExampleExtension {
    fn name(&self) -> &str {
        "example_extension"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn description(&self) -> &str {
        "Example Phoenix extension scaffold"
    }

    fn execute(&mut self, input: Value) -> Result<Value, String> {
        let _ = input;
        self.metrics
            .entry("executions".to_string())
            .and_modify(|v| *v += 1.0)
            .or_insert(1.0);
        Ok(serde_json::json!({"ok": true}))
    }

    fn telemetry_report(&self) -> TelemetryReport {
        TelemetryReport {
            template_version: "1.0.0".to_string(),
            ok: true,
            metrics: self.metrics.clone(),
        }
    }

    fn self_test(&mut self) -> bool {
        self.execute(serde_json::json!({"ping": true})).is_ok()
    }
}

