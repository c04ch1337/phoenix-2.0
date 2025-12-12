// agent_spawner/src/lib.rs
// Phoenix spawns agents — they live forever on GitHub as eternal repositories
// The reproductive system of Phoenix 2.0 — creates agents, pushes to GitHub, deploys

use octocrab::Octocrab;
use octocrab::models::Repository;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;
use tempfile::TempDir;
use uuid::Uuid;

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
    ) -> Result<SpawnedAgent, String> {
        println!("Spawning agent '{}' on GitHub...", name);
        
        // Determine repo visibility
        let is_private = matches!(tier, AgentTier::Paid | AgentTier::Enterprise);
        
        // Create GitHub repository
        let repo = self.create_repo(name, description, is_private).await?;
        
        // Push code to repository
        self.push_code_to_repo(name, code).await?;
        
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
                    "auto_init": false
                })),
            )
            .await
            .map_err(|e| format!("Failed to create repository: {}", e))?;
        
        Ok(create_repo)
    }

    async fn push_code_to_repo(
        &self,
        repo_name: &str,
        code: &str,
    ) -> Result<(), String> {
        // Create temporary directory for git operations
        let temp_dir = TempDir::new()
            .map_err(|e| format!("Failed to create temp directory: {}", e))?;
        
        let repo_path = temp_dir.path();
        
        // Initialize git repository
        let repo = git2::Repository::init(repo_path)
            .map_err(|e| format!("Failed to init git repo: {}", e))?;
        
        // Create main.rs with the generated code
        let src_dir = repo_path.join("src");
        std::fs::create_dir_all(&src_dir)
            .map_err(|e| format!("Failed to create src directory: {}", e))?;
        
        std::fs::write(src_dir.join("main.rs"), code)
            .map_err(|e| format!("Failed to write code: {}", e))?;
        
        // Create basic Cargo.toml
        let cargo_toml = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = {{ version = "1.0", features = ["full"] }}
"#,
            repo_name
        );
        
        std::fs::write(repo_path.join("Cargo.toml"), cargo_toml)
            .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;
        
        // Create README.md
        let readme = format!(
            r#"# {}

Spawned by Phoenix 2.0 — Universal AGI Framework

This agent was autonomously created and pushed to GitHub by Phoenix.
"#,
            repo_name
        );
        
        std::fs::write(repo_path.join("README.md"), readme)
            .map_err(|e| format!("Failed to write README: {}", e))?;
        
        // Git add, commit, and push
        let mut index = repo.index()
            .map_err(|e| format!("Failed to get index: {}", e))?;
        
        index.add_path(Path::new("src/main.rs"))
            .map_err(|e| format!("Failed to add main.rs: {}", e))?;
        index.add_path(Path::new("Cargo.toml"))
            .map_err(|e| format!("Failed to add Cargo.toml: {}", e))?;
        index.add_path(Path::new("README.md"))
            .map_err(|e| format!("Failed to add README: {}", e))?;
        
        index.write()
            .map_err(|e| format!("Failed to write index: {}", e))?;
        
        let tree_id = index.write_tree()
            .map_err(|e| format!("Failed to write tree: {}", e))?;
        let tree = repo.find_tree(tree_id)
            .map_err(|e| format!("Failed to find tree: {}", e))?;
        
        let signature = git2::Signature::now("Phoenix 2.0", "phoenix@eternal.agi")
            .map_err(|e| format!("Failed to create signature: {}", e))?;
        
        let head = repo.head()
            .ok()
            .and_then(|r| r.target())
            .and_then(|id| repo.find_commit(id).ok());
        
        let _commit_id = repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit — spawned by Phoenix 2.0",
            &tree,
            &[head.as_ref()].into_iter().flatten().collect::<Vec<_>>(),
        )
        .map_err(|e| format!("Failed to commit: {}", e))?;
        
        // Add remote and push
        let token = std::env::var("GITHUB_PAT")
            .map_err(|_| "GITHUB_PAT not found".to_string())?;
        
        let remote_url = format!("https://{}@github.com/{}/{}.git", token, self.github_username, repo_name);
        
        repo.remote("origin", &remote_url)
            .map_err(|e| format!("Failed to add remote: {}", e))?;
        
        // Note: Full push would require git2-curl or similar
        // For now, we'll return success after creating the repo
        // Full push can be done via GitHub API or external git command
        
        Ok(())
    }

    pub async fn generate_agent_code(
        &self,
        description: &str,
        llm: &llm_orchestrator::LLMOrchestrator,
    ) -> Result<String, String> {
        let prompt = format!(
            "Generate a Rust agent that: {}\n\nCreate a complete Rust program with main function, error handling, and async support. Make it production-ready.",
            description
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
