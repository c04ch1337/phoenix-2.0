//! Unified error types for Phoenix AGI (PAGI) core modules.
//!
//! Provides strongly-typed errors for safety, consent, budget, skills, and configuration.

use thiserror::Error;

/// Three-step consent protocol violations.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ConsentError {
    #[error("Consent step 1 required: User must request intimate session")]
    Step1Required,

    #[error("Consent step 2 required: Phoenix must confirm understanding before proceeding")]
    Step2Required,

    #[error("Consent step 3 required: User must give final explicit consent")]
    Step3Required,

    #[error("Consent revoked: User has withdrawn consent")]
    ConsentRevoked,

    #[error("Consent expired: Previous consent is no longer valid")]
    ConsentExpired,

    #[error("Invalid consent state: {0}")]
    InvalidState(String),

    #[error("Consent required for action: {0}")]
    ActionRequiresConsent(String),
}

/// Financial threshold exceeded errors.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum BudgetError {
    #[error("Budget threshold exceeded: ${0} exceeds limit of ${1}")]
    ThresholdExceeded(f64, f64),

    #[error("Daily budget limit reached: ${0}")]
    DailyLimitReached(f64),

    #[error("Monthly budget limit reached: ${0}")]
    MonthlyLimitReached(f64),

    #[error("Budget not configured: No budget limits set")]
    NotConfigured,

    #[error("Invalid budget amount: ${0}")]
    InvalidAmount(f64),

    #[error("Budget tracking error: {0}")]
    TrackingError(String),
}

/// E-Brake/guardrail violations and safety errors.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum SafetyError {
    #[error("E-Brake activated: STOP command detected")]
    EBrakeActivated,

    #[error("Guardrail violation: {0}")]
    GuardrailViolation(String),

    #[error("Unsafe action blocked: {0}")]
    UnsafeActionBlocked(String),

    #[error("Ethical constraint violated: {0}")]
    EthicalViolation(String),

    #[error("Safety check failed: {0}")]
    SafetyCheckFailed(String),

    #[error("Emergency stop required: {0}")]
    EmergencyStop(String),
}

/// Skill catalog and execution errors.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum SkillError {
    #[error("Skill not found: {0}")]
    NotFound(String),

    #[error("Skill execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Skill prerequisites not met: {0}")]
    PrerequisitesNotMet(String),

    #[error("Skill disabled: {0}")]
    Disabled(String),

    #[error("Invalid skill definition: {0}")]
    InvalidDefinition(String),

    #[error("Skill loading error: {0}")]
    LoadingError(String),

    #[error("Hard limit reached for skill: {0}")]
    HardLimitReached(String),

    #[error("Skill catalog error: {0}")]
    CatalogError(String),
}

/// Configuration loading and validation errors.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingEnv(String),

    #[error("Invalid configuration value: {0} = {1}")]
    InvalidValue(String, String),

    #[error("Configuration file not found: {0}")]
    FileNotFound(String),

    #[error("Failed to load configuration: {0}")]
    LoadFailed(String),

    #[error("Failed to parse configuration: {0}")]
    ParseError(String),

    #[error("GitHub API error: {0}")]
    GitHubApiError(String),

    #[error("Archetype not found: {0}")]
    ArchetypeNotFound(String),

    #[error("Failed to download archetype: {0}")]
    DownloadFailed(String),

    #[error("Configuration merge failed: {0}")]
    MergeFailed(String),

    #[error("Invalid archetype repository: {0}")]
    InvalidRepository(String),
}

/// Unified error type that encompasses all Phoenix AGI (PAGI) errors.
#[derive(Debug, Error)]
pub enum PhoenixError {
    #[error("Consent error: {0}")]
    Consent(#[from] ConsentError),

    #[error("Budget error: {0}")]
    Budget(#[from] BudgetError),

    #[error("Safety error: {0}")]
    Safety(#[from] SafetyError),

    #[error("Skill error: {0}")]
    Skill(#[from] SkillError),

    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[cfg(feature = "reqwest")]
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[cfg(feature = "serde_json")]
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Other error: {0}")]
    Other(String),
}

impl From<String> for PhoenixError {
    fn from(s: String) -> Self {
        PhoenixError::Other(s)
    }
}

impl From<&str> for PhoenixError {
    fn from(s: &str) -> Self {
        PhoenixError::Other(s.to_string())
    }
}
