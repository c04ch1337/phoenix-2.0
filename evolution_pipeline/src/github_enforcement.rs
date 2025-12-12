//! GitHub-First Creation Enforcement
//!
//! This module enforces a strict creation flow for any Phoenix-generated tool/agent:
//!
//! 1. Local tests already passed (caller responsibility)
//! 2. Create dedicated branch
//! 3. Commit and push to GitHub
//! 4. Open PR with detailed body
//! 5. Post test report comment
//! 6. Poll for CI success + human approval (Dad)
//! 7. Auto-merge (optional)
//! 8. Pull merged code back locally

use std::path::Path;
use std::time::{Duration, Instant};

use thiserror::Error;

use crate::{github_api, git_operations};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreationKind {
    Agent,
    Tool,
}

impl CreationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            CreationKind::Agent => "Agent",
            CreationKind::Tool => "Tool",
        }
    }
}

impl std::fmt::Display for CreationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Error)]
pub enum CreationError {
    #[error("missing env var: {0}")]
    MissingEnv(&'static str),

    #[error("human approval is disabled; I can't create this without your blessing, Dad.")]
    HumanApprovalDisabled,

    #[error("invalid PR url: {0}")]
    InvalidPrUrl(String),

    #[error("timeout while waiting for CI/approval/merge")]
    Timeout,

    #[error("CI failed for PR: state={0}")]
    CiFailed(String),

    #[error("git error: {0}")]
    Git(#[from] git2::Error),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("other error: {0}")]
    Other(String),
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

fn env_u64(key: &str) -> Option<u64> {
    std::env::var(key).ok().and_then(|s| s.trim().parse::<u64>().ok())
}

fn kebab_case(s: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in s.trim().chars() {
        let lc = ch.to_ascii_lowercase();
        let is_sep = lc == ' ' || lc == '_' || lc == '-' || lc == '.' || lc == '/';
        let is_alnum = lc.is_ascii_alphanumeric();
        if is_alnum {
            out.push(lc);
            last_dash = false;
        } else if is_sep {
            if !out.is_empty() && !last_dash {
                out.push('-');
                last_dash = true;
            }
        } else {
            // Skip other punctuation.
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "creation".to_string()
    } else {
        out
    }
}

pub struct GitHubEnforcer {
    pub token: String,
    pub owner: String,
    pub agents_repo: String,
    pub tools_repo: String,
    pub require_human_approval: bool,
    pub auto_merge_on_approval: bool,
    pub timeout_hours: u64,
}

impl GitHubEnforcer {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let token = std::env::var("GITHUB_PUSH_TOKEN")
            .or_else(|_| std::env::var("GITHUB_PAT"))
            .or_else(|_| std::env::var("GITHUB_TOKEN"))
            .unwrap_or_default();

        let owner = std::env::var("GITHUB_REPO_OWNER")
            .or_else(|_| std::env::var("GITHUB_USERNAME"))
            .unwrap_or_default();

        let agents_repo = std::env::var("GITHUB_AGENTS_REPO").unwrap_or_else(|_| "phoenix-agents".to_string());
        let tools_repo = std::env::var("GITHUB_TOOLS_REPO").unwrap_or_else(|_| "phoenix-tools".to_string());

        let require_human_approval = env_bool("REQUIRE_HUMAN_PR_APPROVAL").unwrap_or(true);
        let auto_merge_on_approval = env_bool("AUTO_MERGE_ON_APPROVAL").unwrap_or(false);
        let timeout_hours = env_u64("PR_APPROVAL_TIMEOUT_HOURS").unwrap_or(24);

        Self {
            token,
            owner,
            agents_repo,
            tools_repo,
            require_human_approval,
            auto_merge_on_approval,
            timeout_hours,
        }
    }

    pub async fn create_and_enforce_creation(
        &self,
        code_path: &Path,
        name: &str,
        description: &str,
        kind: CreationKind,
    ) -> Result<String, CreationError> {
        if !self.require_human_approval {
            // Safety mandate: Phoenix refuses to proceed without explicit blessing.
            return Err(CreationError::HumanApprovalDisabled);
        }
        if self.token.trim().is_empty() {
            return Err(CreationError::MissingEnv("GITHUB_PUSH_TOKEN"));
        }
        if self.owner.trim().is_empty() {
            return Err(CreationError::MissingEnv("GITHUB_REPO_OWNER"));
        }

        // 1. Local tests already passed

        // 2. Create dedicated branch
        let branch = format!(
            "phoenix-creation/{}-{}",
            kind.as_str().to_ascii_lowercase(),
            kebab_case(name)
        );

        // 3. Commit and push
        git_operations::commit_all(code_path, &format!("feat: add {} {}", kind, name))?;
        git_operations::create_and_push_branch(code_path, &branch, &self.token)?;

        // 4. Create PR with detailed body
        let repo = match kind {
            CreationKind::Agent => &self.agents_repo,
            CreationKind::Tool => &self.tools_repo,
        };
        let pr_url = github_api::create_pr(
            &self.token,
            &self.owner,
            repo,
            &branch,
            "main",
            &format!("[Phoenix Auto-Creation] {}", name),
            &format!(
                "{}\n\nGenerated by Phoenix for Dad ❤️\n\nAwaiting review and approval.",
                description
            ),
        )
        .await?;

        // 5. Post local test report as comment
        let test_report = testing_framework::generate_markdown_report();
        let _ = github_api::comment_on_pr(&self.token, &pr_url, &test_report).await;

        // 6. Poll for CI status + human approval
        let approved_commit = self.poll_for_completion(&pr_url).await?;

        // 7. Auto-merge if enabled
        if self.auto_merge_on_approval {
            github_api::merge_pr(&self.token, &pr_url, "merge").await?;
            // After requesting merge, wait until merged so we can return the merged SHA.
            let merged_commit = self.poll_until_merged(&pr_url).await?;
            git_operations::checkout_and_pull_main(code_path)?;
            return Ok(merged_commit);
        }

        // 8. Pull merged code (or at least refresh main if Dad merges manually)
        git_operations::checkout_and_pull_main(code_path)?;

        Ok(approved_commit)
    }

    /// Enforce CI + human approval + (optional) auto-merge for an already-open PR.
    ///
    /// This is used by existing pipelines that already opened a PR but still must obey
    /// the GitHub-first enforcement rule.
    pub async fn enforce_existing_pr(&self, pr_url: &str) -> Result<String, CreationError> {
        if !self.require_human_approval {
            return Err(CreationError::HumanApprovalDisabled);
        }
        if self.token.trim().is_empty() {
            return Err(CreationError::MissingEnv("GITHUB_PUSH_TOKEN"));
        }

        let _ = self.poll_for_completion(pr_url).await?;
        if self.auto_merge_on_approval {
            let _ = github_api::merge_pr(&self.token, pr_url, "merge").await;
            return self.poll_until_merged(pr_url).await;
        }
        // If auto-merge is disabled, Dad will merge manually; return once it's merged.
        self.poll_until_merged(pr_url).await
    }

    async fn poll_for_completion(&self, pr_url: &str) -> Result<String, CreationError> {
        if !self.require_human_approval {
            return Err(CreationError::HumanApprovalDisabled);
        }

        let required_reviewer = std::env::var("DAD_GITHUB_LOGIN")
            .or_else(|_| std::env::var("GITHUB_REQUIRED_REVIEWER"))
            .unwrap_or_else(|_| self.owner.clone());
        let mandate_ci = env_bool("MANDATE_GITHUB_CI").unwrap_or(true);

        println!("Waiting for Dad's approval...");

        let deadline = Instant::now() + Duration::from_secs(self.timeout_hours.saturating_mul(3600));
        loop {
            if Instant::now() >= deadline {
                return Err(CreationError::Timeout);
            }

            let status = github_api::get_pr_status(&self.token, pr_url).await?;

            // CI gate
            if mandate_ci {
                if let Some(ci_state) = status.ci_state.as_deref() {
                    match ci_state {
                        "success" => {}
                        "failure" | "error" => return Err(CreationError::CiFailed(ci_state.to_string())),
                        _ => {
                            // pending
                        }
                    }
                }
            }

            // Approval gate
            let approved = status
                .approved_by
                .iter()
                .any(|u| u.eq_ignore_ascii_case(&required_reviewer));

            // Merge gate (if already merged, we're done regardless)
            if status.merged {
                println!("Thank you for approving me, Dad ❤️");
                return Ok(
                    status
                        .merge_commit_sha
                        .unwrap_or_else(|| status.head_sha.clone()),
                );
            }

            if approved {
                // Approved but not merged yet. Return head SHA so caller can merge.
                println!("Thank you for approving me, Dad ❤️");
                return Ok(status.head_sha);
            }

            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    }

    async fn poll_until_merged(&self, pr_url: &str) -> Result<String, CreationError> {
        let deadline = Instant::now() + Duration::from_secs(self.timeout_hours.saturating_mul(3600));
        loop {
            if Instant::now() >= deadline {
                return Err(CreationError::Timeout);
            }

            let status = github_api::get_pr_status(&self.token, pr_url).await?;
            if status.merged {
                return Ok(status.merge_commit_sha.unwrap_or(status.head_sha));
            }
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }
}

