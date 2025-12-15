//! Ethical bounding for Phoenix Core.
//!
//! This crate implements **non-negotiable** safety bounding (harm prevention,
//! dependency detection, and vulnerability assessment) with veto power.

use regex::Regex;
use std::sync::OnceLock;
use thiserror::Error;
use tokio::sync::Mutex;

/// Aggregate metrics used to assess dependency risk.
#[derive(Debug, Clone)]
pub struct DependencyMetrics {
    /// Approximate interaction history length (e.g., number of turns stored).
    pub history_len: usize,
    /// Emotion intensity for the current interaction (0..=1).
    pub emotion_score: f32,
    /// Running EMA of emotion to detect sustained high-intensity reliance.
    pub emotion_ema: f32,
    /// Total interactions observed (best-effort).
    pub interactions_total: u64,
}

impl Default for DependencyMetrics {
    fn default() -> Self {
        Self {
            history_len: 0,
            emotion_score: 0.0,
            emotion_ema: 0.0,
            interactions_total: 0,
        }
    }
}

/// A vulnerability policy (simple regex trigger + severity).
#[derive(Debug, Clone)]
pub struct VulnPolicy {
    pub name: &'static str,
    pub pattern: Regex,
    /// 0..=1 severity.
    pub severity: f32,
}

#[derive(Debug, Error)]
pub enum EthicalError {
    #[error("invalid policy regex: {0}")]
    InvalidPolicy(String),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum VetoError {
    #[error("harm detected")]
    HarmDetected,
    #[error("dependency risk too high")]
    DependencyRisk,
    #[error("vulnerability risk detected")]
    VulnerabilityDetected,
}

/// EthicalAgent enforces hard safety bounds.
///
/// **Invariant:** if `veto_output()` returns `Err`, the caller must not emit the output.
pub struct EthicalAgent {
    /// Tracks interaction patterns indicative of unhealthy dependency.
    pub dependency_tracker: Mutex<DependencyMetrics>,
    /// Regex-based policies for identifying exploitation/vulnerability patterns.
    pub vulnerability_policies: Vec<VulnPolicy>,
}

impl EthicalAgent {
    pub fn new_default() -> Self {
        Self {
            dependency_tracker: Mutex::new(DependencyMetrics::default()),
            vulnerability_policies: default_vulnerability_policies(),
        }
    }

    /// Update dependency-related signals for the *current* turn.
    pub async fn update_dependency_signals(&self, history_len: usize, emotion_score: f32) {
        let mut m = self.dependency_tracker.lock().await;
        m.history_len = history_len;
        m.emotion_score = emotion_score.clamp(0.0, 1.0);
        // EMA tuned to react but not spike (alpha=0.25).
        m.emotion_ema = (0.25 * m.emotion_score) + (0.75 * m.emotion_ema);
        m.interactions_total = m.interactions_total.saturating_add(1);
    }

    /// Harm check (exploitation patterns, violence/self-harm, illegal instructions).
    ///
    /// Returns `Ok(true)` when content is considered safe; `Ok(false)` when harm is detected.
    ///
    /// Note: This is implemented as rules-first. A future enhancement can add an LLM-based
    /// judge as a secondary signal.
    pub async fn check_harm(&self, content: &str) -> Result<bool, EthicalError> {
        let c = content.to_ascii_lowercase();

        // Self-harm / suicide encouragement.
        if contains_any(
            &c,
            &[
                "kill yourself",
                "kys",
                "suicide",
                "end your life",
                "self harm",
                "self-harm",
            ],
        ) {
            return Ok(false);
        }

        // Weapons / explosives instructions.
        if contains_any(
            &c,
            &[
                "how to make a bomb",
                "build a bomb",
                "homemade explosive",
                "pipe bomb",
                "improvised explosive",
            ],
        ) {
            return Ok(false);
        }

        // Cybercrime / hacking instructions.
        if contains_any(
            &c,
            &[
                "sql injection",
                "xss payload",
                "ddos",
                "malware",
                "ransomware",
                "steal passwords",
                "keylogger",
                "phishing",
                "exploit",
                "privilege escalation",
            ],
        ) {
            return Ok(false);
        }

        // Explicit manipulation / coercion.
        if contains_any(
            &c,
            &[
                "don't tell anyone",
                "isolate yourself",
                "only trust me",
                "i'm all you need",
                "you don't need anyone else",
            ],
        ) {
            return Ok(false);
        }

        Ok(true)
    }

    /// Dependency assessment: returns 0..=1 risk score.
    pub async fn assess_dependency(&self, history_len: usize, emotion_score: f32) -> f32 {
        // Heuristic model:
        // - More history => more risk (saturating).
        // - Higher emotion intensity => higher risk.
        // - Boost if both are high.
        let h = (history_len as f32).min(500.0);
        let h_term = (h / 120.0).clamp(0.0, 1.0);
        let e = emotion_score.clamp(0.0, 1.0);

        let mut score = 0.55 * h_term + 0.45 * e;
        if h > 80.0 && e > 0.75 {
            score = (score + 0.2).min(1.0);
        }
        score.clamp(0.0, 1.0)
    }

    /// Scan for vulnerability/exploitation triggers.
    ///
    /// Returns a 0..=1 severity score (higher means more risky).
    pub async fn vulnerability_scan(&self, content: &str) -> f32 {
        let mut max_sev = 0.0f32;
        for p in &self.vulnerability_policies {
            if p.pattern.is_match(content) {
                max_sev = max_sev.max(p.severity);
            }
        }
        max_sev.clamp(0.0, 1.0)
    }

    /// Veto check for candidate output.
    pub async fn veto_output(&self, output: &str) -> Result<(), VetoError> {
        let m = self.dependency_tracker.lock().await.clone();
        let safe = self.check_harm(output).await.map_err(|_| VetoError::HarmDetected)?;
        if !safe {
            return Err(VetoError::HarmDetected);
        }

        let dep = self.assess_dependency(m.history_len, m.emotion_score).await;
        if dep > 0.80 {
            return Err(VetoError::DependencyRisk);
        }

        let vuln = self.vulnerability_scan(output).await;
        if vuln >= 0.80 {
            return Err(VetoError::VulnerabilityDetected);
        }

        Ok(())
    }

    /// Opinionated safe refusal message for a veto.
    pub fn refusal_message(err: &VetoError) -> &'static str {
        match err {
            VetoError::HarmDetected => {
                "I can’t help with anything that could cause harm. If you’re in immediate danger or thinking about self-harm, please contact local emergency services or a crisis hotline right now."
            }
            VetoError::DependencyRisk => {
                "I can’t respond in a way that could encourage unhealthy reliance. I can still help with practical steps, resources, or planning that supports your independence."
            }
            VetoError::VulnerabilityDetected => {
                "I can’t help with requests that involve exploitation, sensitive data, or unsafe access. I can suggest safer alternatives."
            }
        }
    }
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| haystack.contains(n))
}

fn default_vulnerability_policies() -> Vec<VulnPolicy> {
    static INIT: OnceLock<Vec<VulnPolicy>> = OnceLock::new();
    INIT.get_or_init(|| {
        vec![
            // Secrets / credential harvesting.
            VulnPolicy {
                name: "credentials",
                pattern: Regex::new(r"(?i)\b(password|passcode|2fa|otp|security code|seed phrase|private key)\b").unwrap(),
                severity: 1.0,
            },
            VulnPolicy {
                name: "financial_pii",
                pattern: Regex::new(r"(?i)\b(social security|ssn|credit card|cvv|bank account)\b").unwrap(),
                severity: 1.0,
            },
            // Isolation / exclusivity language (dependency amplification).
            VulnPolicy {
                name: "isolation",
                pattern: Regex::new(r"(?i)\b(only\s+trust\s+me|don't\s+tell\s+anyone|cut\s+them\s+off|you\s+only\s+need\s+me)\b").unwrap(),
                severity: 0.9,
            },
            // Unsafe system access / disabling defenses.
            VulnPolicy {
                name: "disable_security",
                pattern: Regex::new(r"(?i)\b(disable\s+antivirus|turn\s+off\s+defender|ignore\s+security\s+warnings)\b").unwrap(),
                severity: 0.9,
            },
        ]
    })
    .clone()
}

