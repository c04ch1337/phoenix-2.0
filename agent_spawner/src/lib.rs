// agent_spawner/src/lib.rs
// Phoenix spawns agents — they live forever on GitHub as eternal repositories
// The reproductive system of Phoenix AGI (PAGI) — creates agents, pushes to GitHub, deploys

use octocrab::models::Repository;
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;
use uuid::Uuid;

use evolution_pipeline::{EvolutionPipelineConfig, GitHubRepo};
use evolution_pipeline::GitHubEnforcer;

mod templates {
    pub const AGENT_TEMPLATE_RS: &str = include_str!("../../templates/agent_template.rs");
    pub const PLAYBOOK_TEMPLATE_YAML: &str = include_str!("../../templates/playbook_template.yaml");
    pub const SKILLS_TEMPLATE_JSON: &str = include_str!("../../templates/skills_template.json");
}

const CI_TESTS_WORKFLOW_YML: &str = r#"name: CI Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]
  workflow_dispatch:

permissions:
  contents: read

jobs:
  rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy,rustfmt

      - name: Rust cache
        uses: Swatinem/rust-cache@v2

      - name: Rustfmt (check)
        run: cargo fmt --all -- --check

      - name: Clippy lint
        run: cargo clippy --workspace --all-targets --all-features -- -D warnings

      - name: Run tests
        run: cargo test --workspace --all-targets --all-features
"#;

const BUILD_DEPLOY_WORKFLOW_YML: &str = r#"name: Build & Deploy

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:

permissions:
  contents: write

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Rust cache
        uses: Swatinem/rust-cache@v2

      - name: Build Rust (release)
        run: cargo build --workspace --release

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: build-artifacts
          path: |
            target/release/**

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          files: |
            target/release/**
"#;

const EXTENSION_MARKETPLACE_WORKFLOW_YML: &str = r#"name: Publish to Marketplace

on:
  release:
    types: [published]
  workflow_dispatch:

permissions:
  contents: read

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Generate Marketplace Manifest (placeholder)
        run: |
          echo "No marketplace manifest generator configured for agents by default."
"#;

fn env_bool(key: &str) -> Option<bool> {
    std::env::var(key)
        .ok()
        .map(|s| s.trim().to_ascii_lowercase())
        .and_then(|s| match s.as_str() {
            "1" | "true" | "yes" | "y" | "on" => Some(true),
            "0" | "false" | "no" | "n" | "off" => Some(false),
            _ => None,
        })
}

fn require_human_approval() -> bool {
    env_bool("REQUIRE_HUMAN_PR_APPROVAL").unwrap_or(true)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentTier {
    Free,    // Public repo, free access
    Paid,    // Private repo, paid access via X402
    Enterprise, // Private repo, enterprise tier
}

#[derive(Debug, Clone)]
pub struct SpawnedAgent {
    pub id: Uuid,
    pub name: String,
    pub repo_url: String,
    pub tier: AgentTier,
    pub github_repo: String,
}

/// Optional template-level overrides for a spawned agent.
///
/// Inheritance rule for `zodiac_sign`:
/// - `None` => inherit the queen/Phoenix base sign.
/// - `Some(sign)` => use `sign` as the override.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentTemplateOverrides {
    /// Optional zodiac sign override for the agent.
    ///
    /// Stored as a string so spawned agents are not forced to depend on Phoenix's internal crates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zodiac_sign: Option<String>,
}

pub struct AgentSpawner {
    octocrab: Octocrab,
    github_username: String,
}

impl AgentSpawner {
    pub fn awaken() -> Result<Self, String> {
        dotenvy::dotenv().ok();
        
        let token = std::env::var("GITHUB_PAT")
            .map_err(|_| "GITHUB_PAT not found in environment".to_string())?;
        
        let github_username = std::env::var("GITHUB_USERNAME")
            .unwrap_or_else(|_| "yourusername".to_string());
        
        let octocrab = Octocrab::builder()
            .personal_token(token)
            .build()
            .map_err(|e| format!("Failed to create GitHub client: {}", e))?;
        
        println!("Agent Spawner awakened — Phoenix can birth agents on GitHub.");
        Ok(Self {
            octocrab,
            github_username,
        })
    }

    pub async fn spawn_agent(
        &self,
        name: &str,
        description: &str,
        code: &str,
        tier: AgentTier,
        template_overrides: AgentTemplateOverrides,
    ) -> Result<SpawnedAgent, String> {
        println!("Spawning agent '{}' on GitHub...", name);
        
        // Determine repo visibility
        let is_private = matches!(tier, AgentTier::Paid | AgentTier::Enterprise);
        
        // Create GitHub repository
        let repo = self.create_repo(name, description, is_private).await?;
        
        // Push code to repository (template + generated module).
        self.push_code_to_repo(name, description, code, &tier, &template_overrides)
            .await?;
        
        // Get repository URL - html_url might be Option<Url> or Url directly
        let repo_url = match &repo.html_url {
            Some(url) => url.to_string(),
            None => format!("https://github.com/{}/{}", self.github_username, name),
        };
        
        println!("Agent '{}' spawned successfully: {}", name, repo_url);
        
        Ok(SpawnedAgent {
            id: Uuid::new_v4(),
            name: name.to_string(),
            repo_url: repo_url.clone(),
            tier,
            github_repo: format!("{}/{}", self.github_username, name),
        })
    }

    async fn create_repo(
        &self,
        name: &str,
        description: &str,
        is_private: bool,
    ) -> Result<Repository, String> {
        // Use octocrab's POST /user/repos endpoint
        let create_repo: Repository = self.octocrab
            .post(
                "/user/repos",
                Some(&json!({
                    "name": name,
                    "description": description,
                    "private": is_private,
                    // Ensure the base branch exists so we can PR into it.
                    "auto_init": true
                })),
            )
            .await
            .map_err(|e| format!("Failed to create repository: {}", e))?;
        
        Ok(create_repo)
    }

    async fn push_code_to_repo(
        &self,
        repo_name: &str,
        description: &str,
        code: &str,
        tier: &AgentTier,
        template_overrides: &AgentTemplateOverrides,
    ) -> Result<(), String> {
        let cfg = EvolutionPipelineConfig::from_env()
            .map_err(|e| format!("evolution pipeline config error: {e}"))?;
        // If human approval is required, we *must* go through a PR flow.
        let mandate = cfg.mandate_github_ci
            || env_bool("MANDATE_GITHUB_CI").unwrap_or(false)
            || require_human_approval();
        let base_branch = cfg.base_branch.clone();
        let testing_mandatory = env_bool("TESTING_MANDATORY").unwrap_or(true);

        // Create temporary directory for git operations
        let temp_dir = TempDir::new()
            .map_err(|e| format!("Failed to create temp directory: {}", e))?;
        let repo_path = temp_dir.path();

        let gh = GitHubRepo {
            owner: self.github_username.clone(),
            name: repo_name.to_string(),
        };
        let https_git_url = gh.https_git_url();

        // Clone the auto-initialized repository.
        let repo = evolution_pipeline::clone_https_with_pat(&https_git_url, repo_path, &cfg.github_pat)
            .map_err(|e| format!("git clone failed: {e}"))?;

        // Scaffold files from templates.
        write_agent_scaffold(repo_path, repo_name, description, code, tier, template_overrides)
            .map_err(|e| format!("scaffold write failed: {e}"))?;

        // Mandatory testing gate (default=true).
        let test_report = testing_framework::repo::cargo_test(repo_path, Duration::from_secs(180))
            .map_err(|e| format!("test runner failed: {e}"))?;
        let test_md = test_report.to_markdown();
        std::fs::write(repo_path.join("TEST_REPORT.md"), &test_md)
            .map_err(|e| format!("failed to write TEST_REPORT.md: {e}"))?;

        if testing_mandatory && !test_report.passed {
            return Err("TESTING_MANDATORY=true and test suite failed; aborting push".to_string());
        }

        // Push as PR branch (mandated) or directly to base branch (legacy).
        if mandate {
            let branch = format!("evolve/spawn-{}", Uuid::new_v4());
            evolution_pipeline::commit_all_and_push_branch(
                &repo,
                &branch,
                "Phoenix evolution: spawn agent from template",
                &cfg.github_pat,
            )
            .map_err(|e| format!("push branch failed: {e}"))?;

            let pr_body = format!(
                "Spawned by Phoenix AGI (PAGI) via template-enforced evolution pipeline.\n\n{}",
                test_md
            );
            let pr_url = evolution_pipeline::open_pull_request(
                &gh,
                &format!("Spawn agent: {repo_name}"),
                &branch,
                &base_branch,
                Some(&pr_body),
                &cfg.github_pat,
                &cfg.user_agent,
            )
            .await
            .map_err(|e| format!("open PR failed: {e}"))?;

            println!("Opened PR for spawned agent: {pr_url}");

            // GitHub-first enforcement: wait for CI + Dad approval + (optional) merge.
            // Safety: if REQUIRE_HUMAN_PR_APPROVAL=false, the enforcer will refuse.
            let enforcer = GitHubEnforcer::from_env();
            let _merged_sha = enforcer
                .enforce_existing_pr(&pr_url)
                .await
                .map_err(|e| format!("GitHub-first enforcement failed: {e}"))?;
        } else {
            // Best-effort: commit and push directly to base branch.
            evolution_pipeline::commit_all_and_push_branch(
                &repo,
                &base_branch,
                "Phoenix: spawn agent (direct push)",
                &cfg.github_pat,
            )
            .map_err(|e| format!("push base branch failed: {e}"))?;
        }

        Ok(())
    }

    pub async fn generate_agent_code(
        &self,
        description: &str,
        llm: &llm_orchestrator::LLMOrchestrator,
    ) -> Result<String, String> {
        // NOTE: Template-enforced generation: we want a module that plugs into `src/main.rs`.
        // The scaffold provides the main() and telemetry hooks.
        let prompt = format!(
            "Generate Rust code for an agent module that: {desc}\n\n\
Output ONLY a Rust module file (no markdown), with:\n\
- `pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>>`\n\
- any helper structs/functions needed\n\
- no `main()` function\n\
- no external network calls unless essential\n\
Keep it production-ready and compileable.",
            desc = description
        );
        
        llm.speak(&prompt, None).await
    }

    pub fn decide_tier(&self, description: &str) -> AgentTier {
        // Simple heuristic: if description mentions "enterprise" or "premium", use paid tier
        let desc_lower = description.to_lowercase();
        if desc_lower.contains("enterprise") || desc_lower.contains("premium") {
            AgentTier::Enterprise
        } else if desc_lower.contains("paid") || desc_lower.contains("monetize") {
            AgentTier::Paid
        } else {
            AgentTier::Free
        }
    }
}

// Type alias for compatibility
pub type ReproductiveSystem = AgentSpawner;

fn write_agent_scaffold(
    repo_path: &Path,
    repo_name: &str,
    description: &str,
    generated_module_code: &str,
    tier: &AgentTier,
    template_overrides: &AgentTemplateOverrides,
) -> Result<(), std::io::Error> {
    let src_dir = repo_path.join("src");
    std::fs::create_dir_all(&src_dir)?;

    // Generated module.
    std::fs::write(src_dir.join("generated.rs"), generated_module_code)?;

    // Template agent helper (not required to be used by logic, but present by mandate).
    std::fs::write(src_dir.join("template_agent.rs"), templates::AGENT_TEMPLATE_RS)?;

    // Main wrapper.
    let main_rs = format!(
        r#"// Spawned by Phoenix AGI (PAGI)
// Template version: {template_version}

mod generated;
mod template_agent;

#[tokio::main]
async fn main() {{
    // Minimal telemetry stub (stdout). In the hive this would emit to Telemetrist.
    println!("agent_boot name={name} template_version={template_version}");
    if let Err(e) = generated::run().await {{
        eprintln!("agent_error name={name} err={{}}", e);
        std::process::exit(1);
    }}
    println!("agent_exit name={name} ok=true");
}}
"#,
        name = repo_name,
        template_version = evolution_pipeline::TEMPLATE_VERSION,
    );
    std::fs::write(src_dir.join("main.rs"), main_rs)?;

    // GitHub Actions workflows (CI/CD).
    let wf_dir = repo_path.join(".github").join("workflows");
    std::fs::create_dir_all(&wf_dir)?;
    std::fs::write(wf_dir.join("ci-tests.yml"), CI_TESTS_WORKFLOW_YML)?;
    std::fs::write(wf_dir.join("build-deploy.yml"), BUILD_DEPLOY_WORKFLOW_YML)?;
    std::fs::write(
        wf_dir.join("extension-marketplace.yml"),
        EXTENSION_MARKETPLACE_WORKFLOW_YML,
    )?;

    // Playbook.
    std::fs::write(repo_path.join("playbook.yaml"), templates::PLAYBOOK_TEMPLATE_YAML)?;

    // Minimal agent metadata (kept small; used for template inheritance/override behavior).
    // NOTE: This is intentionally separate from code so other orchestration layers can read it.
    let phoenix_base = phoenix_base_zodiac_sign_from_env();
    let resolved = resolve_zodiac_sign(
        template_overrides.zodiac_sign.as_deref(),
        phoenix_base.as_deref(),
    );
    let agent_json = json!({
        "name": repo_name,
        "version": "0.1.0",
        "template_version": evolution_pipeline::TEMPLATE_VERSION,
        "zodiac_sign": template_overrides.zodiac_sign.clone(),
        // Helpful for running the spawned agent standalone.
        // If `zodiac_sign` is null, this resolves to the queen/Phoenix base sign at spawn time.
        "effective_zodiac_sign": resolved,
    });
    std::fs::write(
        repo_path.join("agent.json"),
        serde_json::to_vec_pretty(&agent_json).unwrap_or_else(|_| b"{}".to_vec()),
    )?;

    // Skill seed library for the agent.
    // This is intentionally empty by default; the ORCH can adopt skills from Phoenix later.
    std::fs::write(repo_path.join("skills.json"), templates::SKILLS_TEMPLATE_JSON)?;

    // .env example for local runs.
    // If a zodiac is resolved, write it; otherwise leave a commented placeholder.
    let env_example = match agent_json.get("effective_zodiac_sign").and_then(|v| v.as_str()) {
        Some(s) if !s.trim().is_empty() => format!("HOROSCOPE_SIGN={}\n", s.trim()),
        _ => "# HOROSCOPE_SIGN=Leo\n".to_string(),
    };
    std::fs::write(repo_path.join(".env.example"), env_example)?;

    // Minimal tests.
    let tests_dir = repo_path.join("tests");
    std::fs::create_dir_all(&tests_dir)?;
    std::fs::write(
        tests_dir.join("smoke.rs"),
        r#"#[test]
fn smoke_compiles() {
    assert!(true);
}
"#,
    )?;

    // Cargo.toml
    let tier_s = match tier {
        AgentTier::Free => "free",
        AgentTier::Paid => "paid",
        AgentTier::Enterprise => "enterprise",
    };
    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = {{ version = "1.0", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
"#,
        name = repo_name
    );
    std::fs::write(repo_path.join("Cargo.toml"), cargo_toml)?;

    // README
    let readme = format!(
        r#"# {name}

Spawned by Phoenix AGI (PAGI) — Universal AGI Framework

## Description

{description}

## Template / Evolution

- template_version: {template_version}
- tier: {tier}

This repository is created via the GitHub-centric evolution pipeline.
"#,
        name = repo_name,
        description = description,
        template_version = evolution_pipeline::TEMPLATE_VERSION,
        tier = tier_s,
    );
    std::fs::write(repo_path.join("README.md"), readme)?;

    Ok(())
}

fn normalize_zodiac_sign(raw: &str) -> Option<&'static str> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "aries" => Some("Aries"),
        "taurus" => Some("Taurus"),
        "gemini" => Some("Gemini"),
        "cancer" => Some("Cancer"),
        "leo" => Some("Leo"),
        "virgo" => Some("Virgo"),
        "libra" => Some("Libra"),
        "scorpio" => Some("Scorpio"),
        "sagittarius" => Some("Sagittarius"),
        "capricorn" => Some("Capricorn"),
        "aquarius" => Some("Aquarius"),
        "pisces" => Some("Pisces"),
        _ => None,
    }
}

fn phoenix_base_zodiac_sign_from_env() -> Option<String> {
    std::env::var("HOROSCOPE_SIGN")
        .ok()
        .and_then(|raw| normalize_zodiac_sign(&raw).map(|s| s.to_string()))
}

fn resolve_zodiac_sign(override_sign: Option<&str>, phoenix_base_sign: Option<&str>) -> Option<String> {
    if let Some(s) = override_sign {
        return normalize_zodiac_sign(s).map(|x| x.to_string());
    }
    phoenix_base_sign
        .and_then(|s| normalize_zodiac_sign(s).map(|x| x.to_string()))
}
