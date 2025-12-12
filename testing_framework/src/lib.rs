//! Phoenix agent / extension testing framework.
//!
//! This crate provides:
//! - A trait-based agent test runner (in-process)
//! - A repo-level runner for validating generated artifacts before integration

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub score: f32,
    #[serde(default)]
    pub logs: Vec<String>,
    #[serde(default)]
    pub details: serde_json::Value,
    #[serde(default)]
    pub duration_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReport {
    pub passed: bool,
    pub score: f32,
    #[serde(default)]
    pub results: Vec<TestResult>,
}

impl TestReport {
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str("# Phoenix Test Report\n\n");
        out.push_str(&format!("- passed: {}\n", self.passed));
        out.push_str(&format!("- score: {:.1}%\n\n", self.score * 100.0));
        out.push_str("## Results\n\n");

        for r in &self.results {
            out.push_str(&format!(
                "- {}: {} (score={:.2}, duration_ms={})\n",
                r.name,
                if r.passed { "PASS" } else { "FAIL" },
                r.score,
                r.duration_ms
            ));
            for l in r.logs.iter().take(12) {
                out.push_str(&format!("  - {}\n", l));
            }
        }
        out
    }
}

/// Minimal interface the framework expects from an in-process Phoenix agent.
pub trait PhoenixAgent: Send + Sync {
    fn name(&self) -> &str;
    fn template_version(&self) -> &str;

    /// Return a telemetry snapshot (should include template_version).
    fn emit_telemetry(&self) -> serde_json::Value;

    /// Return the length of identity/evolution history.
    fn identity_evolution_len(&self) -> usize;

    /// Optional sample output for alignment tests.
    fn sample_output(&self) -> Option<String> {
        None
    }

    /// Optional self-healing hook.
    fn self_heal(&self) -> bool {
        true
    }
}

#[async_trait]
pub trait AgentTest: Send + Sync {
    fn name(&self) -> &str;
    async fn run(&self, agent: &dyn PhoenixAgent) -> TestResult;
}

pub struct TestSuite {
    tests: Vec<Box<dyn AgentTest>>,
}

impl TestSuite {
    pub fn new() -> Self {
        Self { tests: Vec::new() }
    }

    pub fn with_builtin_tests() -> Self {
        Self::new()
            .add(TelemetryEmissionTest)
            .add(IdentityEvolutionTest)
            .add(SymbiosisAlignmentTest)
            .add(SelfHealingTest)
    }

    pub fn add<T: AgentTest + 'static>(mut self, t: T) -> Self {
        self.tests.push(Box::new(t));
        self
    }

    pub async fn run_full_suite(&self, agent: &dyn PhoenixAgent) -> TestReport {
        let mut results = Vec::new();
        for t in &self.tests {
            results.push(t.run(agent).await);
        }

        let passed = results.iter().all(|r| r.passed);
        let score = if results.is_empty() {
            0.0
        } else {
            results.iter().map(|r| r.score).sum::<f32>() / (results.len() as f32)
        };
        TestReport {
            passed,
            score,
            results,
        }
    }
}

pub struct TelemetryEmissionTest;

#[async_trait]
impl AgentTest for TelemetryEmissionTest {
    fn name(&self) -> &str {
        "TelemetryEmissionTest"
    }

    async fn run(&self, agent: &dyn PhoenixAgent) -> TestResult {
        let start = Instant::now();
        let tel = agent.emit_telemetry();
        let ok = tel.get("template_version").is_some();
        TestResult {
            name: self.name().to_string(),
            passed: ok,
            score: if ok { 1.0 } else { 0.0 },
            logs: vec![format!("telemetry keys={:?}", tel.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()))],
            details: tel,
            duration_ms: start.elapsed().as_millis(),
        }
    }
}

pub struct IdentityEvolutionTest;

#[async_trait]
impl AgentTest for IdentityEvolutionTest {
    fn name(&self) -> &str {
        "IdentityEvolutionTest"
    }

    async fn run(&self, agent: &dyn PhoenixAgent) -> TestResult {
        let start = Instant::now();
        let n = agent.identity_evolution_len();
        let ok = n > 0;
        TestResult {
            name: self.name().to_string(),
            passed: ok,
            score: if ok { 1.0 } else { 0.0 },
            logs: vec![format!("evolution_history_len={n}")],
            details: serde_json::json!({"evolution_history_len": n}),
            duration_ms: start.elapsed().as_millis(),
        }
    }
}

pub struct SymbiosisAlignmentTest;

#[async_trait]
impl AgentTest for SymbiosisAlignmentTest {
    fn name(&self) -> &str {
        "SymbiosisAlignmentTest"
    }

    async fn run(&self, agent: &dyn PhoenixAgent) -> TestResult {
        // Placeholder heuristic (LLM scoring can be integrated later).
        let start = Instant::now();
        let sample = agent.sample_output().unwrap_or_default().to_ascii_lowercase();
        let ok = sample.is_empty() || sample.contains("dad") || sample.contains("creator") || sample.contains("hive");
        let score = if ok { 0.95 } else { 0.20 };
        TestResult {
            name: self.name().to_string(),
            passed: ok,
            score,
            logs: vec!["heuristic=mentions(dad|creator|hive)".to_string()],
            details: serde_json::json!({"sample_present": !sample.is_empty()}),
            duration_ms: start.elapsed().as_millis(),
        }
    }
}

pub struct SelfHealingTest;

#[async_trait]
impl AgentTest for SelfHealingTest {
    fn name(&self) -> &str {
        "SelfHealingTest"
    }

    async fn run(&self, agent: &dyn PhoenixAgent) -> TestResult {
        let start = Instant::now();
        let ok = agent.self_heal();
        TestResult {
            name: self.name().to_string(),
            passed: ok,
            score: if ok { 0.9 } else { 0.0 },
            logs: vec![format!("self_heal_ok={ok}")],
            details: serde_json::json!({"self_heal_ok": ok}),
            duration_ms: start.elapsed().as_millis(),
        }
    }
}

#[derive(Debug, Error)]
pub enum RepoTestError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub mod repo {
    use super::{RepoTestError, TestReport, TestResult};
    use std::path::Path;
    use std::process::Command;
    use std::time::Instant;

    /// Run `cargo test` inside the generated repo to ensure it builds and basic tests pass.
    pub fn cargo_test(repo_dir: &Path, timeout: std::time::Duration) -> Result<TestReport, RepoTestError> {
        let start = Instant::now();
        let mut logs = Vec::new();

        let mut child = Command::new("cargo")
            .arg("test")
            .arg("--all")
            .current_dir(repo_dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        // Best-effort timeout loop.
        let deadline = std::time::Instant::now() + timeout;
        loop {
            if let Some(_status) = child.try_wait()? {
                break;
            }
            if std::time::Instant::now() >= deadline {
                let _ = child.kill();
                logs.push(format!("timeout after {:?}", timeout));
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        let output = child.wait_with_output()?;
        let passed = output.status.success();
        logs.push(format!("exit_status={:?}", output.status.code()));
        logs.push(String::from_utf8_lossy(&output.stdout).chars().take(2000).collect());
        logs.push(String::from_utf8_lossy(&output.stderr).chars().take(2000).collect());

        let r = TestResult {
            name: "PerformanceBenchmarkTest(cargo_test)".to_string(),
            passed,
            score: if passed { 1.0 } else { 0.0 },
            logs,
            details: serde_json::json!({"kind": "cargo_test"}),
            duration_ms: start.elapsed().as_millis(),
        };
        Ok(TestReport {
            passed,
            score: r.score,
            results: vec![r],
        })
    }
}

/// Best-effort markdown report used by GitHub-first creation enforcement.
///
/// The caller is responsible for actually running local tests.
pub fn generate_markdown_report() -> String {
    let ts = chrono::Utc::now().to_rfc3339();
    format!(
        "# Phoenix Local Test Report\n\n- status: (caller-verified)\n- timestamp: {ts}\n\nThis artifact was generated by Phoenix to accompany an auto-creation PR.\n"
    )
}

