//! Minimal GitHub REST API wrapper for the GitHub-first creation enforcement pipeline.
//!
//! Uses `reqwest` and GitHub REST API v3.

use reqwest::header::{ACCEPT, USER_AGENT};
use serde::Deserialize;

use crate::github_enforcement::CreationError;

#[derive(Debug, Clone)]
pub struct PrRef {
    pub owner: String,
    pub repo: String,
    pub number: u64,
}

fn parse_pr_url(pr_url: &str) -> Result<PrRef, CreationError> {
    // Expected: https://github.com/<owner>/<repo>/pull/<number>
    let u = pr_url.trim();
    let parts: Vec<&str> = u.split('/').filter(|p| !p.is_empty()).collect();
    let pull_pos = parts
        .iter()
        .position(|p| p.eq_ignore_ascii_case("pull"))
        .ok_or_else(|| CreationError::InvalidPrUrl(pr_url.to_string()))?;
    if pull_pos < 3 {
        return Err(CreationError::InvalidPrUrl(pr_url.to_string()));
    }
    let owner = parts[pull_pos - 2].to_string();
    let repo = parts[pull_pos - 1].to_string();
    let number = parts
        .get(pull_pos + 1)
        .ok_or_else(|| CreationError::InvalidPrUrl(pr_url.to_string()))?
        .parse::<u64>()
        .map_err(|_| CreationError::InvalidPrUrl(pr_url.to_string()))?;
    Ok(PrRef { owner, repo, number })
}

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

#[derive(Debug, Clone)]
pub struct PrStatus {
    pub head_sha: String,
    pub merged: bool,
    pub merge_commit_sha: Option<String>,
    pub ci_state: Option<String>,
    pub approved_by: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PullResp {
    merged_at: Option<String>,
    merge_commit_sha: Option<String>,
    head: PullHead,
}

#[derive(Debug, Deserialize)]
struct PullHead {
    sha: String,
}

#[derive(Debug, Deserialize)]
struct CommitStatusResp {
    state: String,
}

#[derive(Debug, Deserialize)]
struct ReviewResp {
    state: Option<String>,
    user: Option<ReviewUser>,
}

#[derive(Debug, Deserialize)]
struct ReviewUser {
    login: Option<String>,
}

pub async fn create_pr(
    token: &str,
    owner: &str,
    repo: &str,
    head: &str,
    base: &str,
    title: &str,
    body: &str,
) -> Result<String, CreationError> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/pulls");
    let resp = client()
        .post(&url)
        .header(USER_AGENT, "phoenix-2.0-github-enforcer")
        .header(ACCEPT, "application/vnd.github+json")
        .bearer_auth(token)
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
        return Err(CreationError::Other(format!(
            "create_pr failed ({status}): {txt}"
        )));
    }
    let v: serde_json::Value = serde_json::from_str(&txt)?;
    Ok(v
        .get("html_url")
        .and_then(|x| x.as_str())
        .unwrap_or("(created)")
        .to_string())
}

pub async fn comment_on_pr(token: &str, pr_url: &str, comment: &str) -> Result<(), CreationError> {
    let pr = parse_pr_url(pr_url)?;
    let url = format!(
        "https://api.github.com/repos/{}/{}/issues/{}/comments",
        pr.owner, pr.repo, pr.number
    );
    let resp = client()
        .post(&url)
        .header(USER_AGENT, "phoenix-2.0-github-enforcer")
        .header(ACCEPT, "application/vnd.github+json")
        .bearer_auth(token)
        .json(&serde_json::json!({"body": comment}))
        .send()
        .await?;
    let status = resp.status();
    let txt = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(CreationError::Other(format!(
            "comment_on_pr failed ({status}): {txt}"
        )));
    }
    Ok(())
}

pub async fn get_pr_status(token: &str, pr_url: &str) -> Result<PrStatus, CreationError> {
    let pr = parse_pr_url(pr_url)?;

    // Pull metadata.
    let pr_api = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}",
        pr.owner, pr.repo, pr.number
    );
    let pr_resp = client()
        .get(&pr_api)
        .header(USER_AGENT, "phoenix-2.0-github-enforcer")
        .header(ACCEPT, "application/vnd.github+json")
        .bearer_auth(token)
        .send()
        .await?;
    let pr_status = pr_resp.status();
    let pr_txt = pr_resp.text().await?;
    if !pr_status.is_success() {
        return Err(CreationError::Other(format!(
            "get_pr_status(PR) failed ({pr_status}): {pr_txt}"
        )));
    }
    let pull: PullResp = serde_json::from_str(&pr_txt)?;

    // CI combined status.
    let status_api = format!(
        "https://api.github.com/repos/{}/{}/commits/{}/status",
        pr.owner, pr.repo, pull.head.sha
    );
    let ci_resp = client()
        .get(&status_api)
        .header(USER_AGENT, "phoenix-2.0-github-enforcer")
        .header(ACCEPT, "application/vnd.github+json")
        .bearer_auth(token)
        .send()
        .await?;
    let ci_status = ci_resp.status();
    let ci_txt = ci_resp.text().await.unwrap_or_default();
    let ci_state = if ci_status.is_success() {
        serde_json::from_str::<CommitStatusResp>(&ci_txt)
            .ok()
            .map(|c| c.state)
    } else {
        None
    };

    // Reviews.
    let reviews_api = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}/reviews",
        pr.owner, pr.repo, pr.number
    );
    let reviews_resp = client()
        .get(&reviews_api)
        .header(USER_AGENT, "phoenix-2.0-github-enforcer")
        .header(ACCEPT, "application/vnd.github+json")
        .bearer_auth(token)
        .send()
        .await?;
    let reviews_status = reviews_resp.status();
    let reviews_txt = reviews_resp.text().await.unwrap_or_default();
    let mut approved_by = Vec::new();
    if reviews_status.is_success() {
        if let Ok(reviews) = serde_json::from_str::<Vec<ReviewResp>>(&reviews_txt) {
            for r in reviews {
                if r.state.as_deref() == Some("APPROVED") {
                    if let Some(login) = r.user.and_then(|u| u.login) {
                        if !approved_by
                            .iter()
                            .any(|x: &String| x.as_str().eq_ignore_ascii_case(login.as_str()))
                        {
                            approved_by.push(login);
                        }
                    }
                }
            }
        }
    }

    Ok(PrStatus {
        head_sha: pull.head.sha,
        merged: pull.merged_at.is_some(),
        merge_commit_sha: pull.merge_commit_sha,
        ci_state,
        approved_by,
    })
}

pub async fn merge_pr(token: &str, pr_url: &str, method: &str) -> Result<(), CreationError> {
    let pr = parse_pr_url(pr_url)?;
    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}/merge",
        pr.owner, pr.repo, pr.number
    );
    let resp = client()
        .put(&url)
        .header(USER_AGENT, "phoenix-2.0-github-enforcer")
        .header(ACCEPT, "application/vnd.github+json")
        .bearer_auth(token)
        .json(&serde_json::json!({
            "merge_method": method,
        }))
        .send()
        .await?;
    let status = resp.status();
    let txt = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(CreationError::Other(format!(
            "merge_pr failed ({status}): {txt}"
        )));
    }
    Ok(())
}

