use serde_json::Value;

/// A "pending creation" is represented as an open GitHub Pull Request.
///
/// In the current Phoenix workspace, creations are produced via GitHub-first flows
/// (PR + CI + human approval). The TUI provides a small approval queue over the
/// configured repos.
#[derive(Debug, Clone)]
pub struct PendingCreation {
    pub owner: String,
    pub repo: String,
    pub number: u64,
    pub title: String,
    pub html_url: String,
    // Stored for display/debugging (even if not currently rendered in the TUI).
    #[allow(dead_code)]
    pub head_ref: String,
}

#[derive(Debug, Clone)]
pub struct GitHubApprovalClient {
    token: String,
    owner: String,
    repos: Vec<String>,
    user_agent: String,
}

impl GitHubApprovalClient {
    pub fn from_env() -> Self {
        let token = std::env::var("GITHUB_PUSH_TOKEN")
            .or_else(|_| std::env::var("GITHUB_PAT"))
            .or_else(|_| std::env::var("GITHUB_TOKEN"))
            .unwrap_or_default();

        let owner = std::env::var("GITHUB_REPO_OWNER")
            .or_else(|_| std::env::var("GITHUB_USERNAME"))
            .unwrap_or_default();

        let tools_repo = std::env::var("GITHUB_TOOLS_REPO").unwrap_or_else(|_| "phoenix-tools".to_string());
        let agents_repo =
            std::env::var("GITHUB_AGENTS_REPO").unwrap_or_else(|_| "phoenix-agents".to_string());

        let user_agent = std::env::var("GITHUB_USER_AGENT")
            .unwrap_or_else(|_| "phoenix-tui-approval-queue".to_string());

        // Keep deterministic ordering.
        let mut repos = vec![tools_repo, agents_repo];
        repos.sort();
        repos.dedup();

        Self {
            token,
            owner,
            repos,
            user_agent,
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.token.trim().is_empty() && !self.owner.trim().is_empty()
    }

    pub async fn list_pending_creations(&self) -> Result<Vec<PendingCreation>, String> {
        if !self.is_configured() {
            return Err(
                "GitHub approval queue not configured (need GITHUB_PUSH_TOKEN/GITHUB_PAT + GITHUB_REPO_OWNER)."
                    .to_string(),
            );
        }

        let client = reqwest::Client::new();
        let mut out: Vec<PendingCreation> = Vec::new();

        for repo in &self.repos {
            let url = format!(
                "https://api.github.com/repos/{owner}/{repo}/pulls?state=open&per_page=50",
                owner = self.owner,
                repo = repo
            );
            let resp = client
                .get(&url)
                .header(reqwest::header::USER_AGENT, self.user_agent.clone())
                .header(reqwest::header::ACCEPT, "application/vnd.github+json")
                .bearer_auth(self.token.clone())
                .send()
                .await
                .map_err(|e| format!("GitHub list PRs failed: {e}"))?;

            let status = resp.status();
            let txt = resp.text().await.unwrap_or_default();
            if !status.is_success() {
                return Err(format!("GitHub list PRs failed ({status}): {txt}"));
            }

            let Ok(v) = serde_json::from_str::<Value>(&txt) else {
                continue;
            };
            let Some(arr) = v.as_array() else {
                continue;
            };

            for pr in arr {
                let title = pr.get("title").and_then(|x| x.as_str()).unwrap_or("").to_string();
                let html_url = pr
                    .get("html_url")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string();
                let number = pr.get("number").and_then(|x| x.as_u64()).unwrap_or(0);
                let head_ref = pr
                    .get("head")
                    .and_then(|h| h.get("ref"))
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string();

                if number == 0 || html_url.is_empty() {
                    continue;
                }

                // "Pending creations" heuristic:
                // - branch prefix is the strongest signal for GitHub-first creations
                // - title prefix covers legacy flows
                let looks_like_creation = head_ref.starts_with("phoenix-creation/")
                    || title.contains("[Phoenix Auto-Creation]")
                    || title.to_ascii_lowercase().contains("phoenix auto-creation");
                if !looks_like_creation {
                    continue;
                }

                out.push(PendingCreation {
                    owner: self.owner.clone(),
                    repo: repo.clone(),
                    number,
                    title,
                    html_url,
                    head_ref,
                });
            }
        }

        // Stable sort for deterministic key assignment.
        out.sort_by(|a, b| (a.repo.as_str(), a.number).cmp(&(b.repo.as_str(), b.number)));
        Ok(out)
    }

    pub async fn approve(&self, item: &PendingCreation) -> Result<(), String> {
        if !self.is_configured() {
            return Err(
                "GitHub approval queue not configured (need GITHUB_PUSH_TOKEN/GITHUB_PAT + GITHUB_REPO_OWNER)."
                    .to_string(),
            );
        }

        let client = reqwest::Client::new();
        let url = format!(
            "https://api.github.com/repos/{owner}/{repo}/pulls/{num}/reviews",
            owner = item.owner,
            repo = item.repo,
            num = item.number
        );

        let resp = client
            .post(&url)
            .header(reqwest::header::USER_AGENT, self.user_agent.clone())
            .header(reqwest::header::ACCEPT, "application/vnd.github+json")
            .bearer_auth(self.token.clone())
            .json(&serde_json::json!({
                "event": "APPROVE",
                "body": "Approved from Phoenix TUI (Dad)"
            }))
            .send()
            .await
            .map_err(|e| format!("GitHub approve failed: {e}"))?;

        let status = resp.status();
        let txt = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(format!("GitHub approve failed ({status}): {txt}"));
        }

        Ok(())
    }
}

