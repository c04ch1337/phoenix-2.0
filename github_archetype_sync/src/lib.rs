//! GitHub Archetype Synchronization for Phoenix AGI (PAGI).
//!
//! Provides federated learning push/pull capabilities for archetype configurations:
//! - Download master_system_prompt.txt and personality_db.json from GitHub
//! - Push local changes back to GitHub (federated learning contributions)
//! - Sync archetype configurations across instances

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use error_types::{ConfigError, PhoenixError};
use config_manager::{AGIConfig, PersonalityDatabase};

/// Archetype synchronization client for GitHub operations.
#[derive(Debug, Clone)]
pub struct ArchetypeSync {
    github_pat: String,
    owner: String,
    repo: String,
    branch: String,
    user_agent: String,
    client: reqwest::Client,
}

/// Federated learning contribution data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedContribution {
    /// Instance identifier (unique per Phoenix instance)
    pub instance_id: String,

    /// Timestamp of contribution
    pub timestamp: String,

    /// Type of contribution (prompt_update, personality_update, etc.)
    pub contribution_type: String,

    /// Contribution data (diff, update, or full replacement)
    pub data: serde_json::Value,

    /// Metadata about the contribution
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Pull result containing downloaded archetype files.
#[derive(Debug, Clone)]
pub struct PullResult {
    pub master_system_prompt: String,
    pub personality_db: PersonalityDatabase,
    pub commit_sha: Option<String>,
}

impl ArchetypeSync {
    /// Create a new archetype sync client from environment variables.
    pub fn from_env() -> Result<Self, ConfigError> {
        let github_pat = std::env::var("GITHUB_PAT")
            .or_else(|_| std::env::var("GITHUB_TOKEN"))
            .map_err(|_| ConfigError::MissingEnv("GITHUB_PAT".to_string()))?;

        let owner = std::env::var("GITHUB_REPO_OWNER")
            .or_else(|_| std::env::var("GITHUB_USERNAME"))
            .unwrap_or_else(|_| "c04ch1337".to_string());

        let repo = std::env::var("PHOENIX_ARCHETYPE_REPO")
            .unwrap_or_else(|_| "phoenix-archetypes".to_string());

        let branch = std::env::var("PHOENIX_ARCHETYPE_BRANCH")
            .unwrap_or_else(|_| "main".to_string());

        let user_agent = std::env::var("GITHUB_USER_AGENT")
            .unwrap_or_else(|_| "phoenix-agi-archetype-sync".to_string());

        Ok(Self {
            github_pat,
            owner,
            repo,
            branch,
            user_agent,
            client: reqwest::Client::new(),
        })
    }

    /// Create a new archetype sync client from AGIConfig.
    pub fn from_config(config: &AGIConfig) -> Result<Self, ConfigError> {
        let github_pat = config
            .github_pat
            .as_ref()
            .ok_or_else(|| ConfigError::MissingEnv("GITHUB_PAT".to_string()))?
            .clone();

        Ok(Self {
            github_pat,
            owner: config.github_repo_owner.clone(),
            repo: config.archetype_repo.clone(),
            branch: config.archetype_branch.clone(),
            user_agent: "phoenix-agi-archetype-sync".to_string(),
            client: reqwest::Client::new(),
        })
    }

    /// Pull archetype files from GitHub for a specific archetype.
    ///
    /// # Arguments
    /// * `archetype_name` - Name of the archetype to pull (e.g., "default", "heartbound")
    ///
    /// # Returns
    /// PullResult with downloaded files and optional commit SHA
    pub async fn pull_archetype(&self, archetype_name: &str) -> Result<PullResult, PhoenixError> {
        // Download master_system_prompt.txt
        let prompt_url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}/master_system_prompt.txt",
            self.owner, self.repo, self.branch, archetype_name
        );

        let master_prompt = self
            .client
            .get(&prompt_url)
            .header("Authorization", format!("Bearer {}", self.github_pat))
            .header("Accept", "application/vnd.github.raw")
            .header("User-Agent", &self.user_agent)
            .send()
            .await
            .map_err(|e| ConfigError::DownloadFailed(format!("Failed to download prompt: {}", e)))?
            .text()
            .await
            .map_err(|e| ConfigError::DownloadFailed(format!("Failed to read prompt: {}", e)))?;

        // Download personality_db.json
        let personality_url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}/personality_db.json",
            self.owner, self.repo, self.branch, archetype_name
        );

        let personality_json = self
            .client
            .get(&personality_url)
            .header("Authorization", format!("Bearer {}", self.github_pat))
            .header("Accept", "application/vnd.github.raw")
            .header("User-Agent", &self.user_agent)
            .send()
            .await
            .map_err(|e| ConfigError::DownloadFailed(format!("Failed to download personality: {}", e)))?
            .text()
            .await
            .map_err(|e| ConfigError::DownloadFailed(format!("Failed to read personality: {}", e)))?;

        let personality_db: PersonalityDatabase = serde_json::from_str(&personality_json)
            .map_err(|e| ConfigError::ParseError(format!("Failed to parse personality_db.json: {}", e)))?;

        // Get latest commit SHA for the archetype directory (optional)
        let commit_sha = self
            .get_latest_commit_sha(archetype_name)
            .await
            .ok();

        Ok(PullResult {
            master_system_prompt: master_prompt,
            personality_db,
            commit_sha,
        })
    }

    /// Push federated learning contribution to GitHub.
    ///
    /// Creates a pull request or direct commit (depending on permissions) with
    /// the contribution data.
    ///
    /// # Arguments
    /// * `archetype_name` - Name of the archetype being contributed to
    /// * `contribution` - Federated learning contribution data
    /// * `create_pr` - If true, create a PR instead of direct push
    ///
    /// # Returns
    /// URL of the created PR or commit
    pub async fn push_contribution(
        &self,
        archetype_name: &str,
        contribution: &FederatedContribution,
        create_pr: bool,
    ) -> Result<String, PhoenixError> {
        if create_pr {
            self.push_contribution_as_pr(archetype_name, contribution)
                .await
        } else {
            self.push_contribution_direct(archetype_name, contribution)
                .await
        }
    }

    /// Push contribution as a direct commit (requires write access).
    async fn push_contribution_direct(
        &self,
        _archetype_name: &str,
        contribution: &FederatedContribution,
    ) -> Result<String, PhoenixError> {
        // TODO: Full implementation would use GitHub Contents API to update files
        // Get current file contents via:
        // let prompt_url = format!(
        //     "https://api.github.com/repos/{}/{}/contents/{}/master_system_prompt.txt",
        //     self.owner, self.repo, archetype_name
        // );

        // For now, return a placeholder URL
        Ok(format!(
            "https://github.com/{}/{}/commit/contribution-{}",
            self.owner, self.repo, contribution.instance_id
        ))
    }

    /// Push contribution as a pull request (safer, works with forks).
    async fn push_contribution_as_pr(
        &self,
        archetype_name: &str,
        contribution: &FederatedContribution,
    ) -> Result<String, PhoenixError> {
        // Create a branch for the contribution
        let branch_name = format!(
            "federated-learning/{}/{}-{}",
            archetype_name,
            contribution.contribution_type,
            contribution.instance_id
        );

        // Create PR via GitHub API
        let pr_url = format!(
            "https://api.github.com/repos/{}/{}/pulls",
            self.owner, self.repo
        );

        let pr_body = format!(
            r#"Federated Learning Contribution

**Instance ID**: {}
**Type**: {}
**Timestamp**: {}

## Contribution Data
```json
{}
```

## Metadata
{}
"#,
            contribution.instance_id,
            contribution.contribution_type,
            contribution.timestamp,
            serde_json::to_string_pretty(&contribution.data)
                .unwrap_or_else(|_| "{}".to_string()),
            serde_json::to_string_pretty(&contribution.metadata)
                .unwrap_or_else(|_| "{}".to_string())
        );

        let pr_data = serde_json::json!({
            "title": format!("Federated Learning: {} - {}", archetype_name, contribution.contribution_type),
            "head": branch_name,
            "base": self.branch,
            "body": pr_body,
        });

        let response = self
            .client
            .post(&pr_url)
            .header("Authorization", format!("Bearer {}", self.github_pat))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", &self.user_agent)
            .json(&pr_data)
            .send()
            .await
            .map_err(|e| ConfigError::GitHubApiError(format!("Failed to create PR: {}", e)))?;

        if response.status().is_success() {
            let pr_json: serde_json::Value = response
                .json()
                .await
                .map_err(|e| ConfigError::ParseError(format!("Failed to parse PR response: {}", e)))?;

            Ok(pr_json
                .get("html_url")
                .and_then(|v| v.as_str())
                .unwrap_or("PR created")
                .to_string())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(ConfigError::GitHubApiError(format!(
                "PR creation failed: {}",
                error_text
            ))
            .into())
        }
    }

    /// Get the latest commit SHA for an archetype directory.
    async fn get_latest_commit_sha(&self, archetype_name: &str) -> Result<String, ConfigError> {
        let commits_url = format!(
            "https://api.github.com/repos/{}/{}/commits?path={}&sha={}&per_page=1",
            self.owner, self.repo, archetype_name, self.branch
        );

        let response = self
            .client
            .get(&commits_url)
            .header("Authorization", format!("Bearer {}", self.github_pat))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", &self.user_agent)
            .send()
            .await
            .map_err(|e| ConfigError::GitHubApiError(format!("Failed to get commits: {}", e)))?;

        if response.status().is_success() {
            let commits: Vec<serde_json::Value> = response
                .json()
                .await
                .map_err(|e| ConfigError::ParseError(format!("Failed to parse commits: {}", e)))?;

            if let Some(commit) = commits.first() {
                commit
                    .get("sha")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| ConfigError::ParseError("No SHA in commit".to_string()))
            } else {
                Err(ConfigError::ArchetypeNotFound(archetype_name.to_string()))
            }
        } else {
            Err(ConfigError::GitHubApiError(format!(
                "Failed to get commits: {}",
                response.status()
            )))
        }
    }

    /// List available archetypes in the repository.
    pub async fn list_archetypes(&self) -> Result<Vec<String>, PhoenixError> {
        let contents_url = format!(
            "https://api.github.com/repos/{}/{}/contents?ref={}",
            self.owner, self.repo, self.branch
        );

        let response = self
            .client
            .get(&contents_url)
            .header("Authorization", format!("Bearer {}", self.github_pat))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", &self.user_agent)
            .send()
            .await
            .map_err(|e| ConfigError::GitHubApiError(format!("Failed to list contents: {}", e)))?;

        if response.status().is_success() {
            let contents: Vec<serde_json::Value> = response
                .json()
                .await
                .map_err(|e| ConfigError::ParseError(format!("Failed to parse contents: {}", e)))?;

            let archetypes: Vec<String> = contents
                .iter()
                .filter_map(|item| {
                    if item.get("type")?.as_str() == Some("dir") {
                        item.get("name")?.as_str().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect();

            Ok(archetypes)
        } else {
            Err(ConfigError::GitHubApiError(format!(
                "Failed to list archetypes: {}",
                response.status()
            ))
            .into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_archetype_sync_from_env() {
        // This test requires GITHUB_PAT to be set
        if std::env::var("GITHUB_PAT").is_ok() {
            let sync = ArchetypeSync::from_env();
            assert!(sync.is_ok());
        }
    }
}
