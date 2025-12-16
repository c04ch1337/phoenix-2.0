// ecosystem_manager/src/lib.rs
// Ecosystem Manager - Import, build, and orchestrate GitHub repositories

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoMetadata {
    pub id: String,
    pub name: String,
    pub owner: String,
    pub url: String,
    pub local_path: PathBuf,
    pub build_system: BuildSystem,
    pub build_status: BuildStatus,
    pub service_status: ServiceStatus,
    pub port: Option<u16>,
    pub commands: Vec<String>,
    pub created_at: i64,
    pub last_built: Option<i64>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildSystem {
    Cargo,
    Npm,
    Pip,
    Make,
    Docker,
    Maven,
    Gradle,
    Custom(String),
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildStatus {
    NotBuilt,
    Building,
    Built,
    BuildFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error(String),
}

pub struct EcosystemManager {
    repos: Arc<Mutex<HashMap<String, RepoMetadata>>>,
    base_path: PathBuf,
    processes: Arc<Mutex<HashMap<String, tokio::process::Child>>>,
}

impl EcosystemManager {
    pub fn new(base_path: impl AsRef<Path>) -> Result<Self> {
        let base = base_path.as_ref().to_path_buf();
        std::fs::create_dir_all(&base)
            .context("Failed to create ecosystem base directory")?;

        Ok(Self {
            repos: Arc::new(Mutex::new(HashMap::new())),
            base_path: base,
            processes: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Clone a GitHub repository
    pub async fn import_repo(
        &self,
        owner: &str,
        repo: &str,
        branch: Option<&str>,
    ) -> Result<RepoMetadata> {
        let url = format!("https://github.com/{}/{}", owner, repo);
        let repo_id = Uuid::new_v4().to_string();
        let local_path = self.base_path.join(&repo_id);

        // Clone the repository
        let mut git_cmd = Command::new("git");
        git_cmd.arg("clone");
        if let Some(branch) = branch {
            git_cmd.args(["-b", branch]);
        }
        git_cmd.args([&url, local_path.to_str().unwrap()]);

        let output = git_cmd.output().context("Failed to execute git clone")?;
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Git clone failed: {}", error));
        }

        // Detect build system
        let build_system = Self::detect_build_system(&local_path).await;

        // Discover available commands
        let commands = Self::discover_commands(&local_path, &build_system).await;

        let metadata = RepoMetadata {
            id: repo_id.clone(),
            name: repo.to_string(),
            owner: owner.to_string(),
            url,
            local_path,
            build_system,
            build_status: BuildStatus::NotBuilt,
            service_status: ServiceStatus::Stopped,
            port: None,
            commands,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            last_built: None,
            description: None,
        };

        let mut repos = self.repos.lock().await;
        repos.insert(repo_id, metadata.clone());

        Ok(metadata)
    }

    /// Detect build system from repository files
    async fn detect_build_system(path: &Path) -> BuildSystem {
        let checks = vec![
            ("Cargo.toml", BuildSystem::Cargo),
            ("package.json", BuildSystem::Npm),
            ("requirements.txt", BuildSystem::Pip),
            ("setup.py", BuildSystem::Pip),
            ("Makefile", BuildSystem::Make),
            ("Dockerfile", BuildSystem::Docker),
            ("pom.xml", BuildSystem::Maven),
            ("build.gradle", BuildSystem::Gradle),
        ];

        for (file, system) in checks {
            if path.join(file).exists() {
                return system;
            }
        }

        BuildSystem::Unknown
    }

    /// Discover available commands from build system
    async fn discover_commands(path: &Path, build_system: &BuildSystem) -> Vec<String> {
        let mut commands = vec!["build".to_string(), "start".to_string(), "stop".to_string()];

        match build_system {
            BuildSystem::Cargo => {
                // Check for binaries in Cargo.toml
                if let Ok(contents) = std::fs::read_to_string(path.join("Cargo.toml")) {
                    if contents.contains("[[bin]]") {
                        commands.push("run".to_string());
                    }
                }
            }
            BuildSystem::Npm => {
                // Read package.json scripts
                if let Ok(contents) = std::fs::read_to_string(path.join("package.json")) {
                    if let Ok(json) = serde_json::from_str::<JsonValue>(&contents) {
                        if let Some(scripts) = json.get("scripts").and_then(|s| s.as_object()) {
                            for key in scripts.keys() {
                                if !commands.contains(&key.clone()) {
                                    commands.push(key.clone());
                                }
                            }
                        }
                    }
                }
            }
            BuildSystem::Pip => {
                commands.push("install".to_string());
                commands.push("test".to_string());
            }
            _ => {}
        }

        commands
    }

    /// Build a repository
    pub async fn build_repo(&self, repo_id: &str) -> Result<String> {
        let mut repos = self.repos.lock().await;
        let repo = repos
            .get_mut(repo_id)
            .ok_or_else(|| anyhow::anyhow!("Repository not found: {}", repo_id))?;

        repo.build_status = BuildStatus::Building;
        let build_system = repo.build_system.clone();
        let path = repo.local_path.clone();
        drop(repos);

        let build_output = match build_system {
            BuildSystem::Cargo => {
                let output = Command::new("cargo")
                    .args(["build", "--release"])
                    .current_dir(&path)
                    .output()
                    .context("Failed to execute cargo build")?;
                String::from_utf8_lossy(&output.stdout).to_string()
            }
            BuildSystem::Npm => {
                let output = Command::new("npm")
                    .args(["install"])
                    .current_dir(&path)
                    .output()
                    .context("Failed to execute npm install")?;
                String::from_utf8_lossy(&output.stdout).to_string()
            }
            BuildSystem::Pip => {
                let output = Command::new("pip")
                    .args(["install", "-e", "."])
                    .current_dir(&path)
                    .output()
                    .context("Failed to execute pip install")?;
                String::from_utf8_lossy(&output.stdout).to_string()
            }
            BuildSystem::Make => {
                let output = Command::new("make")
                    .current_dir(&path)
                    .output()
                    .context("Failed to execute make")?;
                String::from_utf8_lossy(&output.stdout).to_string()
            }
            BuildSystem::Docker => {
                let output = Command::new("docker")
                    .args(["build", "-t", repo_id, "."])
                    .current_dir(&path)
                    .output()
                    .context("Failed to execute docker build")?;
                String::from_utf8_lossy(&output.stdout).to_string()
            }
            _ => return Err(anyhow::anyhow!("Unsupported build system")),
        };

        let mut repos = self.repos.lock().await;
        let repo = repos.get_mut(repo_id).unwrap();
        repo.build_status = BuildStatus::Built;
        repo.last_built = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        );

        Ok(build_output)
    }

    /// Start a service from a repository
    pub async fn start_service(&self, repo_id: &str, command: Option<&str>) -> Result<String> {
        let mut repos = self.repos.lock().await;
        let repo = repos
            .get_mut(repo_id)
            .ok_or_else(|| anyhow::anyhow!("Repository not found: {}", repo_id))?;

        if matches!(repo.service_status, ServiceStatus::Running) {
            return Err(anyhow::anyhow!("Service is already running"));
        }

        repo.service_status = ServiceStatus::Starting;
        let build_system = repo.build_system.clone();
        let path = repo.local_path.clone();
        drop(repos);

        let process = match build_system {
            BuildSystem::Cargo => {
                let mut cmd = tokio::process::Command::new("cargo");
                cmd.args(["run", "--release"]);
                cmd.current_dir(&path);
                cmd.spawn()
                    .context("Failed to start cargo run")?
            }
            BuildSystem::Npm => {
                let cmd_name = command.unwrap_or("start");
                let mut cmd = tokio::process::Command::new("npm");
                cmd.arg("run");
                cmd.arg(cmd_name);
                cmd.current_dir(&path);
                cmd.spawn()
                    .context("Failed to start npm run")?
            }
            BuildSystem::Pip => {
                // Try to find main.py or __main__.py
                let main_file = if path.join("main.py").exists() {
                    "main.py"
                } else if path.join("__main__.py").exists() {
                    "__main__.py"
                } else {
                    return Err(anyhow::anyhow!("No main entry point found"));
                };
                let mut cmd = tokio::process::Command::new("python");
                cmd.arg(main_file);
                cmd.current_dir(&path);
                cmd.spawn()
                    .context("Failed to start python service")?
            }
            _ => return Err(anyhow::anyhow!("Unsupported build system for service")),
        };

        let mut processes = self.processes.lock().await;
        processes.insert(repo_id.to_string(), process);

        let mut repos = self.repos.lock().await;
        let repo = repos.get_mut(repo_id).unwrap();
        repo.service_status = ServiceStatus::Running;

        Ok(format!("Service {} started", repo_id))
    }

    /// Stop a service
    pub async fn stop_service(&self, repo_id: &str) -> Result<String> {
        let mut processes = self.processes.lock().await;
        if let Some(mut process) = processes.remove(repo_id) {
            process.kill().await.ok();
        }

        let mut repos = self.repos.lock().await;
        if let Some(repo) = repos.get_mut(repo_id) {
            repo.service_status = ServiceStatus::Stopped;
        }

        Ok(format!("Service {} stopped", repo_id))
    }

    /// List all repositories
    pub async fn list_repos(&self) -> Vec<RepoMetadata> {
        let repos = self.repos.lock().await;
        repos.values().cloned().collect()
    }

    /// Get repository metadata
    pub async fn get_repo(&self, repo_id: &str) -> Option<RepoMetadata> {
        let repos = self.repos.lock().await;
        repos.get(repo_id).cloned()
    }

    /// Remove a repository
    pub async fn remove_repo(&self, repo_id: &str) -> Result<()> {
        // Stop service if running
        self.stop_service(repo_id).await.ok();

        let mut repos = self.repos.lock().await;
        if let Some(repo) = repos.remove(repo_id) {
            // Remove local directory
            std::fs::remove_dir_all(&repo.local_path).ok();
        }

        Ok(())
    }

    /// Execute a custom command on a repository
    pub async fn execute_command(
        &self,
        repo_id: &str,
        command: &str,
        args: Vec<String>,
    ) -> Result<String> {
        let repos = self.repos.lock().await;
        let repo = repos
            .get(repo_id)
            .ok_or_else(|| anyhow::anyhow!("Repository not found: {}", repo_id))?;
        let path = repo.local_path.clone();
        let build_system = repo.build_system.clone();
        drop(repos);

        let output = match build_system {
            BuildSystem::Cargo => {
                let mut cmd = Command::new("cargo");
                cmd.arg(command);
                cmd.args(&args);
                cmd.current_dir(&path);
                cmd.output()?
            }
            BuildSystem::Npm => {
                let mut cmd = Command::new("npm");
                cmd.arg("run");
                cmd.arg(command);
                cmd.args(&args);
                cmd.current_dir(&path);
                cmd.output()?
            }
            _ => return Err(anyhow::anyhow!("Command execution not supported for this build system")),
        };

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Command failed: {}", error));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
