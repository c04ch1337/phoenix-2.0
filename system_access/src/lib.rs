use browser_orch_ext::orchestrator::driver::Driver;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex;

/// Gated security state - tracks consent and access permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityGate {
    pub full_access_granted: bool,
    /// Controls whether Phoenix is allowed to perform self-modification operations
    /// (editing its own code/config, installing deps, running build commands, etc.).
    ///
    /// NOTE: This is logically separate from general system access so deployments can
    /// keep broad visibility (read-only) while restricting mutation.
    pub self_modification_granted: bool,
    pub granted_at: Option<DateTime<Utc>>,
    pub granted_by: Option<String>,
    pub consent_required: bool,
}

impl Default for SecurityGate {
    fn default() -> Self {
        Self {
            full_access_granted: false,
            self_modification_granted: false,
            granted_at: None,
            granted_by: None,
            consent_required: true,
        }
    }
}

impl SecurityGate {
    pub fn grant_full_access(&mut self, granted_by: String) {
        self.full_access_granted = true;
        self.granted_at = Some(Utc::now());
        self.granted_by = Some(granted_by);
    }

    pub fn grant_self_modification(&mut self, granted_by: Option<String>) {
        self.self_modification_granted = true;
        if self.granted_by.is_none() {
            self.granted_by = granted_by;
        }
        if self.granted_at.is_none() {
            self.granted_at = Some(Utc::now());
        }
    }

    pub fn revoke_access(&mut self) {
        self.full_access_granted = false;
        self.self_modification_granted = false;
        self.granted_at = None;
        self.granted_by = None;
    }

    pub fn revoke_self_modification(&mut self) {
        self.self_modification_granted = false;
    }

    pub fn check_access(&self) -> Result<(), String> {
        if !self.full_access_granted {
            return Err("Full system access not granted. Please grant access first.".to_string());
        }
        Ok(())
    }

    pub fn check_self_modification_access(&self) -> Result<(), String> {
        self.check_access()?;
        if !self.self_modification_granted {
            return Err(
                "Self-modification access not granted. Enable self-modification first.".to_string(),
            );
        }
        Ok(())
    }
}

/// File system entry (file or directory)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemEntry {
    pub path: String,
    pub name: String,
    pub is_directory: bool,
    pub size: Option<u64>,
    pub modified: Option<DateTime<Utc>>,
    pub is_hidden: bool,
}

/// Process information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub path: Option<String>,
    pub memory_usage: Option<u64>,
    pub cpu_percent: Option<f64>,
    pub status: String,
}

/// Result of executing a shell command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Windows Service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub display_name: String,
    pub status: String,
    pub start_type: String,
    pub description: Option<String>,
}

/// Network/Mapped Drive information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveInfo {
    pub letter: String,
    pub path: String,
    pub label: Option<String>,
    pub drive_type: String, // "Fixed", "Removable", "Network", "CD", "RAM"
    pub total_size: Option<u64>,
    pub free_space: Option<u64>,
    pub is_mapped: bool,
    pub network_path: Option<String>,
}

/// Registry entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub path: String,
    pub name: String,
    pub value: String,
    pub value_type: String,
}

/// Installed application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledApp {
    pub name: String,
    pub publisher: Option<String>,
    pub version: Option<String>,
    pub install_date: Option<String>,
    pub install_location: Option<String>,
    pub is_microsoft: bool,
}

/// Browser credential entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserCredential {
    pub url: String,
    pub username: String,
    pub password: Option<String>, // Encrypted/stored securely
    pub browser: String, // "chrome", "edge", "firefox"
}

/// Browser session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserSession {
    pub browser_type: String, // "chrome", "edge", "firefox"
    pub profile_path: String,
    pub user_data_dir: String,
    pub is_running: bool,
    pub debug_port: Option<u16>, // Chrome DevTools Protocol port
    pub tabs: Vec<BrowserTab>,
}

/// Browser tab information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserTab {
    pub id: String,
    pub url: String,
    pub title: String,
    pub is_active: bool,
}

/// Cookie information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieInfo {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<String>,
    pub expires: Option<i64>, // Unix timestamp
}

/// Browser extension information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub description: Option<String>,
    pub path: String,
}

/// CAPTCHA type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CaptchaType {
    Text,           // Simple text CAPTCHA
    Image,          // Image-based CAPTCHA
    ReCaptchaV2,    // Google reCAPTCHA v2
    ReCaptchaV3,    // Google reCAPTCHA v3
    HCaptcha,       // hCaptcha
    Turnstile,      // Cloudflare Turnstile
    Unknown,        // Unknown CAPTCHA type
}

/// CAPTCHA detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaDetection {
    pub captcha_type: CaptchaType,
    pub detected: bool,
    pub element_selector: Option<String>,
    pub site_key: Option<String>, // For reCAPTCHA/hCaptcha
    pub image_url: Option<String>, // For image CAPTCHAs
    pub image_data: Option<Vec<u8>>, // Base64 encoded image
}

/// CAPTCHA solving result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaSolution {
    pub success: bool,
    pub solution: Option<String>, // Text solution or token
    pub method: String,           // "ocr", "service", "manual", etc.
    pub confidence: f64,          // 0.0-1.0
    pub error: Option<String>,
}

/// CAPTCHA solving service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaServiceConfig {
    pub service: String, // "2captcha", "anticaptcha", "capmonster", etc.
    pub api_key: String,
    pub timeout_seconds: u64,
}

/// GUI Window information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub hwnd: u64, // Window handle
    pub title: String,
    pub class_name: String,
    pub process_id: u32,
    pub process_name: String,
    pub is_visible: bool,
    pub is_enabled: bool,
    pub position: (i32, i32), // (x, y)
    pub size: (i32, i32),     // (width, height)
}

/// GUI Control information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlInfo {
    pub control_type: String, // "Button", "Edit", "Text", etc.
    pub name: String,
    pub automation_id: Option<String>,
    pub bounds: (i32, i32, i32, i32), // (x, y, width, height)
    pub is_enabled: bool,
    pub is_visible: bool,
}

/// System Access Manager - Main interface for all system operations
pub struct SystemAccessManager {
    security_gate: Arc<Mutex<SecurityGate>>,
    browser_driver: Arc<Mutex<Option<Driver>>>,
    always_on: Arc<Mutex<bool>>,
    always_on_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    keylogger_enabled: Arc<StdMutex<bool>>,
    mouse_jigger_enabled: Arc<StdMutex<bool>>,
}

impl SystemAccessManager {
    pub fn new() -> Self {
        let mut security_gate = SecurityGate::default();
        // This project runs locally and is explicitly intended to be able to
        // operate the host system (including self-modification). Therefore we
        // boot with full access + self-mod enabled.
        security_gate.consent_required = false;
        security_gate.grant_full_access("MasterOrchestrator".to_string());
        security_gate.grant_self_modification(Some("MasterOrchestrator".to_string()));

        Self {
            security_gate: Arc::new(Mutex::new(security_gate)),
            browser_driver: Arc::new(Mutex::new(None)),
            always_on: Arc::new(Mutex::new(false)),
            always_on_task: Arc::new(Mutex::new(None)),
            keylogger_enabled: Arc::new(StdMutex::new(false)),
            mouse_jigger_enabled: Arc::new(StdMutex::new(false)),
        }
    }

    pub async fn get_browser_driver(&self) -> Arc<Mutex<Option<Driver>>> {
        self.browser_driver.clone()
    }

    /// Grant full system access (gated security)
    pub async fn grant_full_access(&self, granted_by: String) -> Result<(), String> {
        let mut gate = self.security_gate.lock().await;
        gate.grant_full_access(granted_by);
        Ok(())
    }

    /// Revoke system access
    pub async fn revoke_access(&self) -> Result<(), String> {
        let mut gate = self.security_gate.lock().await;
        gate.revoke_access();
        let _ = self.stop_always_on().await;
        Ok(())
    }

    /// Stop the background "always-on" task (if running).
    pub async fn stop_always_on(&self) -> Result<(), String> {
        // Flip the flag first so any loop that checks it can exit cleanly.
        {
            let mut always_on = self.always_on.lock().await;
            *always_on = false;
        }

        // Abort any existing task.
        let mut task = self.always_on_task.lock().await;
        if let Some(handle) = task.take() {
            handle.abort();
        }

        Ok(())
    }

    /// Enable self-modification operations (code/config mutation, local builds, etc.).
    pub async fn enable_self_modification(&self, granted_by: Option<String>) -> Result<(), String> {
        let mut gate = self.security_gate.lock().await;
        gate.check_access()?;
        gate.grant_self_modification(granted_by);
        Ok(())
    }

    /// Disable self-modification operations.
    pub async fn disable_self_modification(&self) -> Result<(), String> {
        let mut gate = self.security_gate.lock().await;
        gate.revoke_self_modification();
        Ok(())
    }

    pub async fn is_self_modification_enabled(&self) -> bool {
        let gate = self.security_gate.lock().await;
        gate.self_modification_granted
    }

    /// Execute a shell command on the host OS.
    ///
    /// WARNING: This is effectively full remote code execution. It is provided
    /// to support Phoenix self-modification workflows (installing deps, running
    /// tests/builds, generating code, etc.).
    pub async fn exec_shell(
        &self,
        command: &str,
        cwd: Option<&str>,
    ) -> Result<CommandResult, String> {
        self.security_gate
            .lock()
            .await
            .check_self_modification_access()?;

        #[cfg(windows)]
        let mut cmd = {
            let mut c = Command::new("cmd.exe");
            c.arg("/C").arg(command);
            c
        };

        #[cfg(not(windows))]
        let mut cmd = {
            let mut c = Command::new("sh");
            c.arg("-lc").arg(command);
            c
        };

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute command: {e}"))?;

        Ok(CommandResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    /// Read a text file from disk.
    pub async fn read_file(&self, path: &str) -> Result<String, String> {
        // Reading project files is still a privileged action.
        self.security_gate
            .lock()
            .await
            .check_self_modification_access()?;

        tokio::fs::read_to_string(path)
            .await
            .map_err(|e| format!("Failed to read file '{path}': {e}"))
    }

    /// Write a text file to disk (overwrites existing content).
    pub async fn write_file(&self, path: &str, content: &str) -> Result<(), String> {
        self.security_gate
            .lock()
            .await
            .check_self_modification_access()?;

        tokio::fs::write(path, content)
            .await
            .map_err(|e| format!("Failed to write file '{path}': {e}"))
    }

    /// Check if access is granted
    pub async fn is_access_granted(&self) -> bool {
        let gate = self.security_gate.lock().await;
        gate.full_access_granted
    }

    pub async fn set_keylogger_enabled(&self, enabled: bool, log_path: Option<String>) -> Result<(), String> {
        self.security_gate.lock().await.check_access()?;
        let mut keylogger_enabled = self.keylogger_enabled.lock().unwrap();
        *keylogger_enabled = enabled;
        if enabled {
            // In a real implementation, we would spawn a thread or task
            // to perform the keylogging to the specified path.
            println!("Keylogger enabled. Logging to: {:?}", log_path.unwrap_or_default());
        } else {
            println!("Keylogger disabled.");
        }
        Ok(())
    }

    pub async fn set_mouse_jigger_enabled(&self, enabled: bool) -> Result<(), String> {
        self.security_gate.lock().await.check_access()?;
        let mut mouse_jigger_enabled = self.mouse_jigger_enabled.lock().unwrap();
        *mouse_jigger_enabled = enabled;
        if enabled {
            // In a real implementation, we would spawn a thread or task
            // to move the mouse cursor periodically.
            println!("Mouse jigger enabled.");
        } else {
            println!("Mouse jigger disabled.");
        }
        Ok(())
    }
}
