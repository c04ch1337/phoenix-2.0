//! GitHub-centric evolution pipeline.
//!
//! Goals:
//! - Enforce a Create -> Push (branch) -> CI -> PR -> Merge flow
//! - Provide a shared place for playbook/telemetry evolution artifacts
//! - Keep GitHub operations auditable and reproducible

use git2::{Cred, IndexAddOption, PushOptions, RemoteCallbacks, Repository};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

pub mod github_api;
pub mod github_enforcement;
pub mod git_operations;

pub use github_enforcement::{CreationError, CreationKind, GitHubEnforcer};

pub const TEMPLATE_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolvingPlaybook {
    pub version: u32,
    #[serde(default)]
    pub updates: Vec<String>,
    #[serde(default)]
    pub telemetry: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
pub struct GitHubRepo {
    pub owner: String,
    pub name: String,
}

impl GitHubRepo {
    pub fn parse(owner_slash_name: &str) -> Result<Self, EvolutionPipelineError> {
        let s = owner_slash_name.trim();
        let mut it = s.split('/');
        let owner = it.next().unwrap_or("").trim();
        let name = it.next().unwrap_or("").trim();
        if owner.is_empty() || name.is_empty() || it.next().is_some() {
            return Err(EvolutionPipelineError::InvalidRepoSpec(s.to_string()));
        }
        Ok(Self {
            owner: owner.to_string(),
            name: name.to_string(),
        })
    }

    pub fn https_git_url(&self) -> String {
        format!("https://github.com/{}/{}.git", self.owner, self.name)
    }

    pub fn api_base(&self) -> String {
        format!("https://api.github.com/repos/{}/{}", self.owner, self.name)
    }
}

#[derive(Debug, Clone)]
pub struct EvolutionPipelineConfig {
    pub mandate_github_ci: bool,
    pub github_pat: String,
    pub user_agent: String,
    pub base_branch: String,
}

impl EvolutionPipelineConfig {
    pub fn from_env() -> Result<Self, EvolutionPipelineError> {
        let mandate_github_ci = env_bool("MANDATE_GITHUB_CI").unwrap_or(false);
        let github_pat = std::env::var("GITHUB_PAT")
            .or_else(|_| std::env::var("GITHUB_TOKEN"))
            .map_err(|_| EvolutionPipelineError::MissingEnv("GITHUB_PAT"))?;

        Ok(Self {
            mandate_github_ci,
            github_pat,
            user_agent: std::env::var("GITHUB_USER_AGENT")
                .unwrap_or_else(|_| "phoenix-2.0-evolution-pipeline".to_string()),
            base_branch: std::env::var("GITHUB_BASE_BRANCH")
                .unwrap_or_else(|_| "main".to_string()),
        })
    }
}

#[derive(Debug, Error)]
pub enum EvolutionPipelineError {
    #[error("missing env var: {0}")]
    MissingEnv(&'static str),

    #[error("invalid repo spec (expected 'owner/name'): {0}")]
    InvalidRepoSpec(String),

    #[error("git error: {0}")]
    Git(#[from] git2::Error),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

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

pub fn remote_callbacks_with_pat(pat: &str) -> RemoteCallbacks<'static> {
    let token = pat.to_string();
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |_, _, _| {
        // GitHub supports using PAT as the password with a fixed username.
        Cred::userpass_plaintext("x-access-token", &token)
    });
    callbacks
}

/// Clone an existing GitHub repo into `target_dir` using PAT.
pub fn clone_https_with_pat(
    https_git_url: &str,
    target_dir: &Path,
    github_pat: &str,
) -> Result<Repository, EvolutionPipelineError> {
    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(remote_callbacks_with_pat(github_pat));
    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fo);
    Ok(builder.clone(https_git_url, target_dir)?)
}

/// Stage all files, create a commit, and push `branch` to `origin`.
///
/// Returns the pushed branch name.
pub fn commit_all_and_push_branch(
    repo: &Repository,
    branch: &str,
    message: &str,
    github_pat: &str,
) -> Result<String, EvolutionPipelineError> {
    // Checkout/create branch.
    let head_commit = repo.head()?.peel_to_commit()?;
    repo.branch(branch, &head_commit, true)?;
    repo.set_head(&format!("refs/heads/{branch}"))?;
    repo.checkout_head(Some(
        git2::build::CheckoutBuilder::new()
            .force()
            .remove_untracked(true),
    ))?;

    // Stage everything.
    let mut index = repo.index()?;
    index.add_all(["*"], IndexAddOption::DEFAULT, None)?;
    index.write()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    // Commit.
    let sig = repo
        .signature()
        .or_else(|_| git2::Signature::now("Phoenix AGI (PAGI)", "phoenix@eternal.agi"))?;
    let parent = repo.head()?.peel_to_commit()?;
    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent])?;

    // Push.
    let mut remote = repo.find_remote("origin")?;
    let callbacks = remote_callbacks_with_pat(github_pat);
    let mut push_opts = PushOptions::new();
    push_opts.remote_callbacks(callbacks);
    remote.push(
        &[&format!(
            "refs/heads/{branch}:refs/heads/{branch}",
            branch = branch
        )],
        Some(&mut push_opts),
    )?;

    Ok(branch.to_string())
}

/// Create a pull request via GitHub REST API.
pub async fn open_pull_request(
    repo: &GitHubRepo,
    title: &str,
    head: &str,
    base: &str,
    body: Option<&str>,
    github_pat: &str,
    user_agent: &str,
) -> Result<String, EvolutionPipelineError> {
    let client = reqwest::Client::new();
    let url = format!("{}/pulls", repo.api_base());
    let resp = client
        .post(&url)
        .header(reqwest::header::USER_AGENT, user_agent)
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .bearer_auth(github_pat)
        .json(&serde_json::json!({
            "title": title,
            "head": head,
            "base": base,
            "body": body,
        }))
        .send()
        .await?;

    let status = resp.status();
    let txt = resp.text().await?;
    if !status.is_success() {
        return Err(EvolutionPipelineError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("open_pull_request failed ({status}): {txt}"),
        )));
    }
    // best-effort parse for html_url
    let v: serde_json::Value = serde_json::from_str(&txt)?;
    Ok(v
        .get("html_url")
        .and_then(|x| x.as_str())
        .unwrap_or("(created)")
        .to_string())
}


