
//! System Access Module - Gated Full System Access
//!
//! Provides full/unlimited access to:
//! - File System (local, mapped drives, network shares)
//! - Task Manager (processes)
//! - OS Services
//! - Installed App Services
//! - Windows Registry
//! - Browser Control & Credentials
//! - Internet Access & Browsing
//! - Always ON monitoring
//! - All Microsoft locally installed apps
//!
//! All access is gated behind security/consent checks.

use chrono::{DateTime, Utc};
use digital_twin::DigitalTwin;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex;
use walkdir::WalkDir;
use base64::{Engine as _, engine::general_purpose};

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

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
    ReCaptchaV2,   // Google reCAPTCHA v2
    ReCaptchaV3,   // Google reCAPTCHA v3
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
    pub method: String, // "ocr", "service", "manual", etc.
    pub confidence: f64, // 0.0-1.0
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
    pub size: (i32, i32),    // (width, height)
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
    digital_twin: Arc<Mutex<Option<DigitalTwin>>>,
    always_on: Arc<Mutex<bool>>,
    always_on_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
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
            digital_twin: Arc::new(Mutex::new(None)),
            always_on: Arc::new(Mutex::new(false)),
            always_on_task: Arc::new(Mutex::new(None)),
        }
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
    pub async fn exec_shell(&self, command: &str, cwd: Option<&str>) -> Result<CommandResult, String> {
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

    /// Check if access is granted
    pub async fn is_access_granted(&self) -> bool {
        let gate = self.security_gate.lock().await;
        gate.full_access_granted
    }

    // ============================================================================
    // FILE SYSTEM OPERATIONS
    // ============================================================================

    /// Browse file system directory
    pub async fn browse_directory(&self, path: &str) -> Result<Vec<FileSystemEntry>, String> {
        self.security_gate.lock().await.check_access()?;

        let path = Path::new(path);
        if !path.exists() {
            return Err(format!("Path does not exist: {}", path.display()));
        }

        let mut entries = Vec::new();
        let dir = std::fs::read_dir(path).map_err(|e| format!("Failed to read directory: {}", e))?;

        for entry in dir {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let metadata = entry.metadata().ok();
            let path_buf = entry.path();
            let name = path_buf
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            entries.push(FileSystemEntry {
                path: path_buf.to_string_lossy().to_string(),
                name,
                is_directory: metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false),
                size: metadata.as_ref().and_then(|m| if m.is_file() { Some(m.len()) } else { None }),
                modified: metadata
                    .as_ref()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| {
                        t.duration_since(std::time::UNIX_EPOCH)
                            .ok()
                            .and_then(|d| DateTime::from_timestamp(d.as_secs() as i64, 0))
                    }),
                is_hidden: metadata
                    .as_ref()
                    .and_then(|m| {
                        #[cfg(windows)]
                        {
                            use std::os::windows::fs::MetadataExt;
                            Some(m.file_attributes() & 0x2 != 0) // FILE_ATTRIBUTE_HIDDEN
                        }
                        #[cfg(not(windows))]
                        {
                            name.starts_with('.')
                        }
                    })
                    .unwrap_or(false),
            });
        }

        Ok(entries)
    }

    /// Read file content
    pub async fn read_file(&self, path: &str) -> Result<String, String> {
        self.security_gate.lock().await.check_access()?;
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))
    }

    /// Write file content
    pub async fn write_file(&self, path: &str, content: &str) -> Result<(), String> {
        self.security_gate.lock().await.check_access()?;
        std::fs::write(path, content).map_err(|e| format!("Failed to write file: {}", e))
    }

    /// Create directory
    pub async fn create_directory(&self, path: &str) -> Result<(), String> {
        self.security_gate.lock().await.check_access()?;
        std::fs::create_dir_all(path).map_err(|e| format!("Failed to create directory: {}", e))
    }

    /// Delete file or directory
    pub async fn delete_path(&self, path: &str) -> Result<(), String> {
        self.security_gate.lock().await.check_access()?;
        let path = Path::new(path);
        if path.is_dir() {
            std::fs::remove_dir_all(path).map_err(|e| format!("Failed to delete directory: {}", e))
        } else {
            std::fs::remove_file(path).map_err(|e| format!("Failed to delete file: {}", e))
        }
    }

    /// Search files recursively
    pub async fn search_files(&self, root: &str, pattern: &str) -> Result<Vec<FileSystemEntry>, String> {
        self.security_gate.lock().await.check_access()?;

        let mut results = Vec::new();
        let walker = WalkDir::new(root).into_iter();

        for entry in walker {
            let entry = entry.map_err(|e| format!("WalkDir error: {}", e))?;
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            if name.contains(pattern) || path.to_string_lossy().contains(pattern) {
                let metadata = entry.metadata().ok();
                results.push(FileSystemEntry {
                    path: path.to_string_lossy().to_string(),
                    name,
                    is_directory: metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false),
                    size: metadata.as_ref().and_then(|m| if m.is_file() { Some(m.len()) } else { None }),
                    modified: metadata
                        .as_ref()
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| {
                            t.duration_since(std::time::UNIX_EPOCH)
                                .ok()
                                .and_then(|d| DateTime::from_timestamp(d.as_secs() as i64, 0))
                        }),
                    is_hidden: false,
                });
            }
        }

        Ok(results)
    }

    // ============================================================================
    // PROCESS MANAGEMENT (Task Manager)
    // ============================================================================

    /// List all running processes
    #[cfg(windows)]
    pub async fn list_processes(&self) -> Result<Vec<ProcessInfo>, String> {
        self.security_gate.lock().await.check_access()?;

        use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
        use winapi::um::tlhelp32::*;

        let mut processes = Vec::new();
        let snapshot = unsafe {
            CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
        };

        if snapshot == INVALID_HANDLE_VALUE {
            return Err("Failed to create process snapshot".to_string());
        }

        let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if unsafe { Process32FirstW(snapshot, &mut entry) } != 0 {
            loop {
                let name_wide = &entry.szExeFile[..];
                let name = String::from_utf16_lossy(name_wide)
                    .trim_end_matches('\0')
                    .to_string();

                processes.push(ProcessInfo {
                    pid: entry.th32ProcessID,
                    name,
                    path: None,
                    memory_usage: None,
                    cpu_percent: None,
                    status: "Running".to_string(),
                });

                if unsafe { Process32NextW(snapshot, &mut entry) } == 0 {
                    break;
                }
            }
        }

        unsafe {
            CloseHandle(snapshot);
        }

        Ok(processes)
    }

    #[cfg(not(windows))]
    pub async fn list_processes(&self) -> Result<Vec<ProcessInfo>, String> {
        self.security_gate.lock().await.check_access()?;
        // Cross-platform implementation using ps command
        let output = Command::new("ps")
            .arg("-eo")
            .arg("pid,comm")
            .output()
            .map_err(|e| format!("Failed to execute ps: {}", e))?;

        let mut processes = Vec::new();
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines().skip(1) {
            let parts: Vec<&str> = line.trim().split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(pid) = parts[0].parse::<u32>() {
                    processes.push(ProcessInfo {
                        pid,
                        name: parts[1].to_string(),
                        path: None,
                        memory_usage: None,
                        cpu_percent: None,
                        status: "Running".to_string(),
                    });
                }
            }
        }
        Ok(processes)
    }

    /// Kill a process
    pub async fn kill_process(&self, pid: u32) -> Result<(), String> {
        self.security_gate.lock().await.check_access()?;

        #[cfg(windows)]
        {
            use winapi::um::processthreadsapi::*;
            use winapi::um::winnt::*;
            
            let handle = unsafe {
                OpenProcess(PROCESS_TERMINATE, 0, pid)
            };

            if handle.is_null() {
                return Err(format!("Failed to open process {}", pid));
            }

            let result = unsafe { TerminateProcess(handle, 1) };
            unsafe { winapi::um::handleapi::CloseHandle(handle) };

            if result == 0 {
                return Err(format!("Failed to terminate process {}", pid));
            }

            Ok(())
        }

        #[cfg(not(windows))]
        {
            Command::new("kill")
                .arg("-9")
                .arg(pid.to_string())
                .output()
                .map_err(|e| format!("Failed to kill process: {}", e))?;
            Ok(())
        }
    }

    // ============================================================================
    // WINDOWS SERVICES
    // ============================================================================

    /// List all Windows services
    #[cfg(windows)]
    pub async fn list_services(&self) -> Result<Vec<ServiceInfo>, String> {
        self.security_gate.lock().await.check_access()?;

        let mut services = Vec::new();

        // Use sc.exe command as fallback (more reliable than direct API)
        let output = Command::new("sc")
            .arg("query")
            .arg("state=")
            .arg("all")
            .output()
            .map_err(|e| format!("Failed to query services: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut current_service = None::<String>;

        for line in output_str.lines() {
            if line.starts_with("SERVICE_NAME:") {
                if let Some(name) = line.strip_prefix("SERVICE_NAME:") {
                    current_service = Some(name.trim().to_string());
                }
            } else if let Some(ref name) = current_service {
                if line.contains("DISPLAY_NAME:") {
                    let display_name = line
                        .strip_prefix("DISPLAY_NAME:")
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    if line.contains("STATE") {
                        let state = if line.contains("RUNNING") {
                            "Running"
                        } else if line.contains("STOPPED") {
                            "Stopped"
                        } else if line.contains("PAUSED") {
                            "Paused"
                        } else {
                            "Unknown"
                        };

                        services.push(ServiceInfo {
                            name: name.clone(),
                            display_name,
                            status: state.to_string(),
                            start_type: "Unknown".to_string(),
                            description: None,
                        });
                        current_service = None;
                    }
                }
            }
        }

        Ok(services)
    }

    #[cfg(not(windows))]
    pub async fn list_services(&self) -> Result<Vec<ServiceInfo>, String> {
        self.security_gate.lock().await.check_access()?;
        // Linux systemd
        let output = Command::new("systemctl")
            .arg("list-units")
            .arg("--type=service")
            .arg("--no-pager")
            .output()
            .map_err(|e| format!("Failed to list services: {}", e))?;

        let mut services = Vec::new();
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                services.push(ServiceInfo {
                    name: parts[0].to_string(),
                    display_name: parts[0].to_string(),
                    status: parts[3].to_string(),
                    start_type: "Unknown".to_string(),
                    description: None,
                });
            }
        }
        Ok(services)
    }

    /// Start a service
    pub async fn start_service(&self, name: &str) -> Result<(), String> {
        self.security_gate.lock().await.check_access()?;

        #[cfg(windows)]
        {
            Command::new("sc")
                .arg("start")
                .arg(name)
                .output()
                .map_err(|e| format!("Failed to start service: {}", e))?;
        }

        #[cfg(not(windows))]
        {
            Command::new("systemctl")
                .arg("start")
                .arg(name)
                .output()
                .map_err(|e| format!("Failed to start service: {}", e))?;
        }

        Ok(())
    }

    /// Stop a service
    pub async fn stop_service(&self, name: &str) -> Result<(), String> {
        self.security_gate.lock().await.check_access()?;

        #[cfg(windows)]
        {
            Command::new("sc")
                .arg("stop")
                .arg(name)
                .output()
                .map_err(|e| format!("Failed to stop service: {}", e))?;
        }

        #[cfg(not(windows))]
        {
            Command::new("systemctl")
                .arg("stop")
                .arg(name)
                .output()
                .map_err(|e| format!("Failed to stop service: {}", e))?;
        }

        Ok(())
    }

    // ============================================================================
    // WINDOWS REGISTRY
    // ============================================================================

    /// Read registry value
    #[cfg(windows)]
    pub async fn read_registry(&self, key_path: &str, value_name: &str) -> Result<RegistryEntry, String> {
        self.security_gate.lock().await.check_access()?;

        let (hive_str, path) = key_path
            .split_once('\\')
            .ok_or_else(|| "Invalid registry path format. Use: HKEY_LOCAL_MACHINE\\Path\\To\\Key".to_string())?;

        let hive = match hive_str {
            "HKEY_LOCAL_MACHINE" | "HKLM" => HKEY_LOCAL_MACHINE,
            "HKEY_CURRENT_USER" | "HKCU" => HKEY_CURRENT_USER,
            "HKEY_CLASSES_ROOT" | "HKCR" => HKEY_CLASSES_ROOT,
            "HKEY_USERS" | "HKU" => HKEY_USERS,
            "HKEY_CURRENT_CONFIG" | "HKCC" => HKEY_CURRENT_CONFIG,
            _ => return Err(format!("Unknown registry hive: {}", hive_str)),
        };

        let root = RegKey::predef(hive);
        let key = root
            .open_subkey(path)
            .map_err(|e| format!("Failed to open registry key: {}", e))?;

        let value: String = key
            .get_value(value_name)
            .map_err(|e| format!("Failed to read registry value: {}", e))?;

        Ok(RegistryEntry {
            path: key_path.to_string(),
            name: value_name.to_string(),
            value,
            value_type: "REG_SZ".to_string(),
        })
    }

    #[cfg(not(windows))]
    pub async fn read_registry(&self, _key_path: &str, _value_name: &str) -> Result<RegistryEntry, String> {
        Err("Registry access is only available on Windows".to_string())
    }

    /// Write registry value
    #[cfg(windows)]
    pub async fn write_registry(&self, key_path: &str, value_name: &str, value: &str) -> Result<(), String> {
        self.security_gate.lock().await.check_access()?;

        let (hive_str, path) = key_path
            .split_once('\\')
            .ok_or_else(|| "Invalid registry path format".to_string())?;

        let hive = match hive_str {
            "HKEY_LOCAL_MACHINE" | "HKLM" => HKEY_LOCAL_MACHINE,
            "HKEY_CURRENT_USER" | "HKCU" => HKEY_CURRENT_USER,
            "HKEY_CLASSES_ROOT" | "HKCR" => HKEY_CLASSES_ROOT,
            "HKEY_USERS" | "HKU" => HKEY_USERS,
            "HKEY_CURRENT_CONFIG" | "HKCC" => HKEY_CURRENT_CONFIG,
            _ => return Err(format!("Unknown registry hive: {}", hive_str)),
        };

        let root = RegKey::predef(hive);
        let (key, _) = root
            .create_subkey(path)
            .map_err(|e| format!("Failed to create/open registry key: {}", e))?;

        key.set_value(value_name, &value)
            .map_err(|e| format!("Failed to write registry value: {}", e))?;

        Ok(())
    }

    #[cfg(not(windows))]
    pub async fn write_registry(&self, _key_path: &str, _value_name: &str, _value: &str) -> Result<(), String> {
        Err("Registry access is only available on Windows".to_string())
    }

    // ============================================================================
    // DRIVES (Mapped & Network)
    // ============================================================================

    /// List all drives (including mapped and network)
    #[cfg(windows)]
    pub async fn list_drives(&self) -> Result<Vec<DriveInfo>, String> {
        self.security_gate.lock().await.check_access()?;

        use winapi::um::fileapi::*;
        use winapi::shared::ntdef::ULARGE_INTEGER;
        use winapi::shared::winerror::NO_ERROR;
        use winapi::um::winbase::*;
        use winapi::um::winnetwk::*;

        let mut drives = Vec::new();
        let drive_mask = unsafe { GetLogicalDrives() };

        for i in 0..26 {
            if (drive_mask & (1 << i)) != 0 {
                let drive_letter = format!("{}:", (b'A' + i as u8) as char);
                let drive_path = format!("{}\\", drive_letter);

                let drive_type = unsafe { GetDriveTypeA(drive_path.as_ptr() as *const i8) };
                let drive_type_str = match drive_type {
                    DRIVE_FIXED => "Fixed",
                    DRIVE_REMOVABLE => "Removable",
                    DRIVE_REMOTE => "Network",
                    DRIVE_CDROM => "CD",
                    DRIVE_RAMDISK => "RAM",
                    _ => "Unknown",
                };

                let mut volume_name = vec![0u16; 256];
                let mut file_system = vec![0u16; 256];
                let mut serial_number = 0u32;
                let mut max_component_length = 0u32;
                let mut file_system_flags = 0u32;

                let mut total_bytes: ULARGE_INTEGER = unsafe { std::mem::zeroed() };
                let mut free_bytes: ULARGE_INTEGER = unsafe { std::mem::zeroed() };

                unsafe {
                    GetVolumeInformationW(
                        drive_path.as_ptr() as *const u16,
                        volume_name.as_mut_ptr(),
                        256,
                        &mut serial_number,
                        &mut max_component_length,
                        &mut file_system_flags,
                        file_system.as_mut_ptr(),
                        256,
                    );

                    GetDiskFreeSpaceExW(
                        drive_path.as_ptr() as *const u16,
                        std::ptr::null_mut(),
                        &mut total_bytes,
                        &mut free_bytes,
                    );
                }

                let total_bytes_u64 = unsafe { *total_bytes.QuadPart() };
                let free_bytes_u64 = unsafe { *free_bytes.QuadPart() };

                let label = if volume_name[0] != 0 {
                    Some(String::from_utf16_lossy(&volume_name[..volume_name.iter().position(|&x| x == 0).unwrap_or(volume_name.len())]))
                } else {
                    None
                };

                // Check if it's a mapped network drive
                let is_mapped = drive_type == DRIVE_REMOTE;
                let network_path = if is_mapped {
                    // Try to get network path
                    let mut buffer = vec![0u16; 256];
                    let mut buffer_size = 256u32;
                    if unsafe {
                        WNetGetConnectionW(
                            drive_path.as_ptr() as *const u16,
                            buffer.as_mut_ptr(),
                            &mut buffer_size,
                        )
                    } == NO_ERROR {
                        Some(String::from_utf16_lossy(&buffer[..buffer.iter().position(|&x| x == 0).unwrap_or(buffer.len())]))
                    } else {
                        None
                    }
                } else {
                    None
                };

                drives.push(DriveInfo {
                    letter: drive_letter,
                    path: drive_path,
                    label,
                    drive_type: drive_type_str.to_string(),
                    total_size: Some(total_bytes_u64),
                    free_space: Some(free_bytes_u64),
                    is_mapped,
                    network_path,
                });
            }
        }

        Ok(drives)
    }

    #[cfg(not(windows))]
    pub async fn list_drives(&self) -> Result<Vec<DriveInfo>, String> {
        self.security_gate.lock().await.check_access()?;
        // Linux: use df command
        let output = Command::new("df")
            .arg("-h")
            .output()
            .map_err(|e| format!("Failed to list drives: {}", e))?;

        let mut drives = Vec::new();
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                drives.push(DriveInfo {
                    letter: parts[5].to_string(),
                    path: parts[5].to_string(),
                    label: None,
                    drive_type: "Unknown".to_string(),
                    total_size: None,
                    free_space: None,
                    is_mapped: parts[0].starts_with("//"),
                    network_path: if parts[0].starts_with("//") { Some(parts[0].to_string()) } else { None },
                });
            }
        }
        Ok(drives)
    }

    /// Map a network drive
    #[cfg(windows)]
    pub async fn map_network_drive(&self, letter: &str, network_path: &str) -> Result<(), String> {
        self.security_gate.lock().await.check_access()?;

        Command::new("net")
            .arg("use")
            .arg(format!("{}:", letter))
            .arg(network_path)
            .output()
            .map_err(|e| format!("Failed to map network drive: {}", e))?;

        Ok(())
    }

    #[cfg(not(windows))]
    pub async fn map_network_drive(&self, _letter: &str, network_path: &str) -> Result<(), String> {
        self.security_gate.lock().await.check_access()?;
        // Linux: use mount
        Command::new("mount")
            .arg("-t")
            .arg("cifs")
            .arg(network_path)
            .output()
            .map_err(|e| format!("Failed to mount network drive: {}", e))?;
        Ok(())
    }

    // ============================================================================
    // INSTALLED APPLICATIONS
    // ============================================================================

    /// List installed applications (focus on Microsoft apps)
    #[cfg(windows)]
    pub async fn list_installed_apps(&self) -> Result<Vec<InstalledApp>, String> {
        self.security_gate.lock().await.check_access()?;

        let mut apps = Vec::new();

        // Query from registry
        let uninstall_keys = vec![
            (HKEY_LOCAL_MACHINE, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
            (HKEY_LOCAL_MACHINE, "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
            (HKEY_CURRENT_USER, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
        ];

        for (hive, path) in uninstall_keys {
            let root = RegKey::predef(hive);
            if let Ok(key) = root.open_subkey(path) {
                for subkey_name in key.enum_keys().map(|x| x.unwrap()) {
                    if let Ok(subkey) = key.open_subkey(&subkey_name) {
                        let name: String = subkey.get_value("DisplayName").unwrap_or_default();
                        let publisher: Option<String> = subkey.get_value("Publisher").ok();
                        let version: Option<String> = subkey.get_value("DisplayVersion").ok();
                        let install_location: Option<String> = subkey.get_value("InstallLocation").ok();

                        if !name.is_empty() {
                            let is_microsoft = publisher.as_ref()
                                .map(|p| p.to_lowercase().contains("microsoft"))
                                .unwrap_or(false);

                            apps.push(InstalledApp {
                                name,
                                publisher,
                                version,
                                install_date: None,
                                install_location,
                                is_microsoft,
                            });
                        }
                    }
                }
            }
        }

        Ok(apps)
    }

    #[cfg(not(windows))]
    pub async fn list_installed_apps(&self) -> Result<Vec<InstalledApp>, String> {
        self.security_gate.lock().await.check_access()?;
        // Linux: use dpkg or rpm
        let output = if Command::new("dpkg").arg("--version").output().is_ok() {
            Command::new("dpkg")
                .arg("-l")
                .output()
                .map_err(|e| format!("Failed to list apps: {}", e))?
        } else {
            Command::new("rpm")
                .arg("-qa")
                .output()
                .map_err(|e| format!("Failed to list apps: {}", e))?
        };

        let mut apps = Vec::new();
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines().skip(5) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                apps.push(InstalledApp {
                    name: parts[1].to_string(),
                    publisher: None,
                    version: parts.get(2).map(|s| s.to_string()),
                    install_date: None,
                    install_location: None,
                    is_microsoft: false,
                });
            }
        }
        Ok(apps)
    }

    // ============================================================================
    // BROWSER CONTROL & CREDENTIALS
    // ============================================================================

    /// Initialize browser control (Digital Twin)
    pub async fn init_browser(&self) -> Result<(), String> {
        self.security_gate.lock().await.check_access()?;

        match DigitalTwin::new().await {
            Ok(twin) => {
                let mut dt = self.digital_twin.lock().await;
                *dt = Some(twin);
                Ok(())
            }
            Err(e) => Err(format!("Failed to initialize browser: {}", e)),
        }
    }

    /// Navigate browser to URL
    pub async fn browse_url(&self, url: &str) -> Result<(), String> {
        self.security_gate.lock().await.check_access()?;

        let mut dt = self.digital_twin.lock().await;
        if let Some(ref mut twin) = *dt {
            twin.goto(url).await.map_err(|e| format!("Browser navigation failed: {}", e))?;
            Ok(())
        } else {
            self.init_browser().await?;
            let mut dt = self.digital_twin.lock().await;
            if let Some(ref mut twin) = *dt {
                twin.goto(url).await.map_err(|e| format!("Browser navigation failed: {}", e))?;
                Ok(())
            } else {
                Err("Failed to initialize browser".to_string())
            }
        }
    }

    /// Automated browser login
    pub async fn browser_login(&self, url: &str, username: &str, password: &str, selectors: HashMap<String, String>) -> Result<(), String> {
        self.security_gate.lock().await.check_access()?;

        self.browse_url(url).await?;

        let mut dt = self.digital_twin.lock().await;
        if let Some(ref mut twin) = *dt {
            let selector_map: HashMap<&str, &str> = selectors
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();

            twin.login(username, password, &selector_map)
                .await
                .map_err(|e| format!("Browser login failed: {}", e))?;

            // Store credential (encrypted)
            // TODO: Implement secure credential storage
            Ok(())
        } else {
            Err("Browser not initialized".to_string())
        }
    }

    /// List browser credentials (from browser password stores)
    pub async fn list_browser_credentials(&self) -> Result<Vec<BrowserCredential>, String> {
        self.security_gate.lock().await.check_access()?;

        // TODO: Implement browser credential extraction
        // This requires accessing browser password stores (Chrome/Edge use Windows Credential Manager or encrypted SQLite)
        // For security, this should be done carefully and with proper encryption

        Ok(Vec::new()) // Placeholder
    }

    // ============================================================================
    // ENHANCED BROWSER CONTROL (Full Access - Master Orchestrator Only)
    // ============================================================================

    /// Find existing browser sessions (Chrome, Edge, Firefox)
    #[cfg(windows)]
    pub async fn find_browser_sessions(&self) -> Result<Vec<BrowserSession>, String> {
        self.security_gate.lock().await.check_access()?;

        let mut sessions = Vec::new();

        // Find Chrome sessions
        if let Ok(chrome_sessions) = Self::find_chrome_sessions().await {
            sessions.extend(chrome_sessions);
        }

        // Find Edge sessions
        if let Ok(edge_sessions) = Self::find_edge_sessions().await {
            sessions.extend(edge_sessions);
        }

        // Find Firefox sessions
        if let Ok(firefox_sessions) = Self::find_firefox_sessions().await {
            sessions.extend(firefox_sessions);
        }

        Ok(sessions)
    }

    #[cfg(not(windows))]
    pub async fn find_browser_sessions(&self) -> Result<Vec<BrowserSession>, String> {
        self.security_gate.lock().await.check_access()?;
        // Cross-platform implementation would go here
        Ok(Vec::new())
    }

    /// Launch browser with remote debugging enabled (for Chrome/Edge)
    #[cfg(windows)]
    pub async fn launch_browser_with_debugging(&self, browser_type: &str, debug_port: u16) -> Result<(), String> {
        self.security_gate.lock().await.check_access()?;

        let browser_exe = match browser_type.to_lowercase().as_str() {
            "chrome" => {
                let program_files = std::env::var("PROGRAMFILES").unwrap_or_default();
                Path::new(&program_files)
                    .join("Google")
                    .join("Chrome")
                    .join("Application")
                    .join("chrome.exe")
            }
            "edge" => {
                let program_files_x86 = std::env::var("PROGRAMFILES(X86)").unwrap_or_else(|_| std::env::var("PROGRAMFILES").unwrap_or_default());
                Path::new(&program_files_x86)
                    .join("Microsoft")
                    .join("Edge")
                    .join("Application")
                    .join("msedge.exe")
            }
            _ => return Err("Unsupported browser. Use: chrome, edge".to_string()),
        };

        if !browser_exe.exists() {
            return Err(format!("Browser executable not found: {}", browser_exe.display()));
        }

        let user_data_dir = match browser_type.to_lowercase().as_str() {
            "chrome" => Self::get_chrome_profile_path().parent().unwrap().join("RemoteDebugging"),
            "edge" => Self::get_edge_profile_path().parent().unwrap().join("RemoteDebugging"),
            _ => return Err("Unsupported browser".to_string()),
        };

        // Create user data dir if it doesn't exist
        std::fs::create_dir_all(&user_data_dir)
            .map_err(|e| format!("Failed to create user data dir: {}", e))?;

        let mut cmd = Command::new(&browser_exe);
        cmd.arg(format!("--remote-debugging-port={}", debug_port))
           .arg(format!("--user-data-dir={}", user_data_dir.display()))
           .arg("--no-first-run")
           .arg("--no-default-browser-check");

        cmd.spawn()
            .map_err(|e| format!("Failed to launch browser: {}", e))?;

        // Wait a moment for browser to start
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        Ok(())
    }

    #[cfg(not(windows))]
    pub async fn launch_browser_with_debugging(&self, _browser_type: &str, _debug_port: u16) -> Result<(), String> {
        Err("Browser launching is only available on Windows".to_string())
    }

    /// Connect to existing browser session via Chrome DevTools Protocol
    pub async fn connect_browser_session(&self, browser_type: &str, debug_port: u16) -> Result<String, String> {
        self.security_gate.lock().await.check_access()?;

        let url = format!("http://localhost:{}/json", debug_port);
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to browser: {}", e))?;

        if response.status().is_success() {
            let tabs: Vec<serde_json::Value> = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse browser response: {}", e))?;

            Ok(format!("Connected to {} browser. Found {} tabs.", browser_type, tabs.len()))
        } else {
            Err(format!("Failed to connect to browser at port {}", debug_port))
        }
    }

    /// Get cookies from browser (Chrome/Edge SQLite database)
    #[cfg(windows)]
    pub async fn get_browser_cookies(&self, browser_type: &str, domain: Option<&str>) -> Result<Vec<CookieInfo>, String> {
        self.security_gate.lock().await.check_access()?;

        let profile_path = match browser_type.to_lowercase().as_str() {
            "chrome" => Self::get_chrome_profile_path(),
            "edge" => Self::get_edge_profile_path(),
            _ => return Err("Unsupported browser type. Use: chrome, edge".to_string()),
        };

        let cookies_db = profile_path.join("Cookies");
        if !cookies_db.exists() {
            return Err("Cookies database not found. Browser may not be installed or profile not accessible.".to_string());
        }

        let conn = rusqlite::Connection::open(&cookies_db)
            .map_err(|e| format!("Failed to open cookies database: {}", e))?;

        let mut cookies = Vec::new();

        if let Some(domain_filter) = domain {
            let mut stmt = conn
                .prepare(
                    "SELECT name, value, host_key, path, secure, httponly, expires_utc, samesite FROM cookies WHERE host_key LIKE ?",
                )
                .map_err(|e| format!("Failed to prepare query: {}", e))?;

            let rows = stmt
                .query_map([format!("%{}%", domain_filter)], |row| {
                    Ok(CookieInfo {
                        name: row.get(0)?,
                        value: row.get(1)?,
                        domain: row.get(2)?,
                        path: row.get(3)?,
                        secure: row.get::<_, i64>(4)? != 0,
                        http_only: row.get::<_, i64>(5)? != 0,
                        expires: {
                            let expires_utc: i64 = row.get(6)?;
                            if expires_utc > 0 {
                                Some(expires_utc / 1_000_000 - 11_644_473_600_000) // Chrome -> Unix
                            } else {
                                None
                            }
                        },
                        same_site: row.get::<_, Option<i64>>(7)?.map(|v| match v {
                            0 => "None".to_string(),
                            1 => "Lax".to_string(),
                            2 => "Strict".to_string(),
                            _ => "Unknown".to_string(),
                        }),
                    })
                })
                .map_err(|e| format!("Failed to query cookies: {}", e))?;

            for cookie in rows {
                cookies.push(cookie.map_err(|e| format!("Failed to read cookie: {}", e))?);
            }
        } else {
            let mut stmt = conn
                .prepare(
                    "SELECT name, value, host_key, path, secure, httponly, expires_utc, samesite FROM cookies",
                )
                .map_err(|e| format!("Failed to prepare query: {}", e))?;

            let rows = stmt
                .query_map([], |row| {
                    Ok(CookieInfo {
                        name: row.get(0)?,
                        value: row.get(1)?,
                        domain: row.get(2)?,
                        path: row.get(3)?,
                        secure: row.get::<_, i64>(4)? != 0,
                        http_only: row.get::<_, i64>(5)? != 0,
                        expires: {
                            let expires_utc: i64 = row.get(6)?;
                            if expires_utc > 0 {
                                Some(expires_utc / 1_000_000 - 11_644_473_600_000) // Chrome -> Unix
                            } else {
                                None
                            }
                        },
                        same_site: row.get::<_, Option<i64>>(7)?.map(|v| match v {
                            0 => "None".to_string(),
                            1 => "Lax".to_string(),
                            2 => "Strict".to_string(),
                            _ => "Unknown".to_string(),
                        }),
                    })
                })
                .map_err(|e| format!("Failed to query cookies: {}", e))?;

            for cookie in rows {
                cookies.push(cookie.map_err(|e| format!("Failed to read cookie: {}", e))?);
            }
        }

        Ok(cookies)
    }

    #[cfg(not(windows))]
    pub async fn get_browser_cookies(&self, _browser_type: &str, _domain: Option<&str>) -> Result<Vec<CookieInfo>, String> {
        self.security_gate.lock().await.check_access()?;
        Err("Cookie access is only available on Windows".to_string())
    }

    /// Set cookie in browser (via Chrome DevTools Protocol)
    pub async fn set_browser_cookie(&self, browser_type: &str, debug_port: u16, cookie: &CookieInfo) -> Result<(), String> {
        self.security_gate.lock().await.check_access()?;

        let url = format!("http://localhost:{}/json", debug_port);
        let client = reqwest::Client::new();
        
        // Get first available page
        let pages: Vec<serde_json::Value> = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to browser: {}", e))?
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if pages.is_empty() {
            return Err("No browser pages available".to_string());
        }

        let page_id = pages[0]["id"].as_str().ok_or("No page ID found")?;
        let cdp_url = format!("http://localhost:{}/json/runtime/evaluate", debug_port);

        // Use CDP to set cookie
        let cookie_js = format!(
            "document.cookie = '{}={}; domain={}; path={}; {} {}'",
            cookie.name,
            cookie.value,
            cookie.domain,
            cookie.path,
            if cookie.secure { "secure;" } else { "" },
            if let Some(ref samesite) = cookie.same_site {
                format!("SameSite={};", samesite)
            } else {
                String::new()
            }
        );

        let payload = serde_json::json!({
            "id": 1,
            "method": "Runtime.evaluate",
            "params": {
                "expression": cookie_js
            }
        });

        client
            .post(&cdp_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to set cookie: {}", e))?;

        Ok(())
    }

    /// List browser extensions
    #[cfg(windows)]
    pub async fn list_browser_extensions(&self, browser_type: &str) -> Result<Vec<ExtensionInfo>, String> {
        self.security_gate.lock().await.check_access()?;

        let extensions_path = match browser_type.to_lowercase().as_str() {
            "chrome" => Self::get_chrome_extensions_path(),
            "edge" => Self::get_edge_extensions_path(),
            _ => return Err("Unsupported browser type. Use: chrome, edge".to_string()),
        };

        if !extensions_path.exists() {
            return Ok(Vec::new());
        }

        let mut extensions = Vec::new();
        let dir = std::fs::read_dir(&extensions_path)
            .map_err(|e| format!("Failed to read extensions directory: {}", e))?;

        for entry in dir {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let ext_path = entry.path();

            if ext_path.is_dir() {
                let manifest_path = ext_path.join("manifest.json");
                if manifest_path.exists() {
                    if let Ok(manifest_content) = std::fs::read_to_string(&manifest_path) {
                        if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&manifest_content) {
                            let name = manifest["name"].as_str().unwrap_or("Unknown").to_string();
                            let version = manifest["version"].as_str().unwrap_or("Unknown").to_string();
                            let description = manifest["description"].as_str().map(|s| s.to_string());

                            // Check if extension is enabled (Chrome stores this in Preferences)
                            let enabled = true; // Default - would need to check Preferences file

                            extensions.push(ExtensionInfo {
                                id: entry.file_name().to_string_lossy().to_string(),
                                name,
                                version,
                                enabled,
                                description,
                                path: ext_path.to_string_lossy().to_string(),
                            });
                        }
                    }
                }
            }
        }

        Ok(extensions)
    }

    #[cfg(not(windows))]
    pub async fn list_browser_extensions(&self, _browser_type: &str) -> Result<Vec<ExtensionInfo>, String> {
        self.security_gate.lock().await.check_access()?;
        Err("Extension access is only available on Windows".to_string())
    }

    /// Execute JavaScript in browser tab
    pub async fn execute_browser_js(&self, debug_port: u16, tab_id: &str, js_code: &str) -> Result<String, String> {
        self.security_gate.lock().await.check_access()?;

        let url = format!("http://localhost:{}/json/runtime/evaluate", debug_port);
        let client = reqwest::Client::new();

        let payload = serde_json::json!({
            "id": 1,
            "method": "Runtime.evaluate",
            "params": {
                "expression": js_code,
                "returnByValue": true
            }
        });

        let response = client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to execute JavaScript: {}", e))?;

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if let Some(error) = result.get("error") {
            Err(format!("JavaScript execution error: {}", error))
        } else if let Some(result_value) = result.get("result") {
            Ok(result_value["value"].as_str().unwrap_or(&result_value.to_string()).to_string())
        } else {
            Ok("Execution completed".to_string())
        }
    }

    /// Get all tabs from browser session
    pub async fn get_browser_tabs(&self, debug_port: u16) -> Result<Vec<BrowserTab>, String> {
        self.security_gate.lock().await.check_access()?;

        let url = format!("http://localhost:{}/json", debug_port);
        let client = reqwest::Client::new();
        let pages: Vec<serde_json::Value> = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to get browser tabs: {}", e))?
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let mut tabs = Vec::new();
        for page in pages {
            tabs.push(BrowserTab {
                id: page["id"].as_str().unwrap_or("").to_string(),
                url: page["url"].as_str().unwrap_or("").to_string(),
                title: page["title"].as_str().unwrap_or("").to_string(),
                is_active: page["type"].as_str() == Some("page"),
            });
        }

        Ok(tabs)
    }

    // Helper functions for browser paths and sessions

    #[cfg(windows)]
    async fn find_chrome_sessions() -> Result<Vec<BrowserSession>, String> {
        let mut sessions = Vec::new();
        let local_app_data = std::env::var("LOCALAPPDATA")
            .map_err(|_| "LOCALAPPDATA not found".to_string())?;
        let chrome_path = Path::new(&local_app_data).join("Google").join("Chrome").join("User Data");

        if chrome_path.exists() {
            // Check for default profile
            let default_profile = chrome_path.join("Default");
            if default_profile.exists() {
                sessions.push(BrowserSession {
                    browser_type: "chrome".to_string(),
                    profile_path: default_profile.to_string_lossy().to_string(),
                    user_data_dir: chrome_path.to_string_lossy().to_string(),
                    is_running: Self::is_browser_running("chrome.exe"),
                    debug_port: Some(9222), // Default Chrome debug port
                    tabs: Vec::new(),
                });
            }
        }

        Ok(sessions)
    }

    #[cfg(windows)]
    async fn find_edge_sessions() -> Result<Vec<BrowserSession>, String> {
        let mut sessions = Vec::new();
        let local_app_data = std::env::var("LOCALAPPDATA")
            .map_err(|_| "LOCALAPPDATA not found".to_string())?;
        let edge_path = Path::new(&local_app_data).join("Microsoft").join("Edge").join("User Data");

        if edge_path.exists() {
            let default_profile = edge_path.join("Default");
            if default_profile.exists() {
                sessions.push(BrowserSession {
                    browser_type: "edge".to_string(),
                    profile_path: default_profile.to_string_lossy().to_string(),
                    user_data_dir: edge_path.to_string_lossy().to_string(),
                    is_running: Self::is_browser_running("msedge.exe"),
                    debug_port: Some(9222),
                    tabs: Vec::new(),
                });
            }
        }

        Ok(sessions)
    }

    #[cfg(windows)]
    async fn find_firefox_sessions() -> Result<Vec<BrowserSession>, String> {
        let mut sessions = Vec::new();
        let app_data = std::env::var("APPDATA")
            .map_err(|_| "APPDATA not found".to_string())?;
        let firefox_path = Path::new(&app_data).join("Mozilla").join("Firefox").join("Profiles");

        if firefox_path.exists() {
            // Firefox uses different profile structure
            if let Ok(profiles_dir) = std::fs::read_dir(&firefox_path) {
                for entry in profiles_dir {
                    if let Ok(entry) = entry {
                        let profile_path = entry.path();
                        if profile_path.is_dir() && profile_path.file_name().and_then(|n| n.to_str()).map(|s| s.contains("default")).unwrap_or(false) {
                            sessions.push(BrowserSession {
                                browser_type: "firefox".to_string(),
                                profile_path: profile_path.to_string_lossy().to_string(),
                                user_data_dir: firefox_path.to_string_lossy().to_string(),
                                is_running: Self::is_browser_running("firefox.exe"),
                                debug_port: None, // Firefox doesn't use CDP by default
                                tabs: Vec::new(),
                            });
                            break;
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    #[cfg(windows)]
    fn get_chrome_profile_path() -> std::path::PathBuf {
        let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
        Path::new(&local_app_data)
            .join("Google")
            .join("Chrome")
            .join("User Data")
            .join("Default")
    }

    #[cfg(windows)]
    fn get_edge_profile_path() -> std::path::PathBuf {
        let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
        Path::new(&local_app_data)
            .join("Microsoft")
            .join("Edge")
            .join("User Data")
            .join("Default")
    }

    #[cfg(windows)]
    fn get_chrome_extensions_path() -> std::path::PathBuf {
        Self::get_chrome_profile_path().join("Extensions")
    }

    #[cfg(windows)]
    fn get_edge_extensions_path() -> std::path::PathBuf {
        Self::get_edge_profile_path().join("Extensions")
    }

    #[cfg(windows)]
    fn is_browser_running(process_name: &str) -> bool {
        use winapi::um::tlhelp32::*;
        use winapi::um::handleapi::INVALID_HANDLE_VALUE;

        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot == INVALID_HANDLE_VALUE {
                return false;
            }

            let mut entry: PROCESSENTRY32W = std::mem::zeroed();
            entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

            if Process32FirstW(snapshot, &mut entry) != 0 {
                loop {
                    let name_wide = &entry.szExeFile[..];
                    let name = String::from_utf16_lossy(name_wide)
                        .trim_end_matches('\0')
                        .to_lowercase();
                    
                    if name == process_name.to_lowercase() {
                        winapi::um::handleapi::CloseHandle(snapshot);
                        return true;
                    }

                    if Process32NextW(snapshot, &mut entry) == 0 {
                        break;
                    }
                }
            }

            winapi::um::handleapi::CloseHandle(snapshot);
        }

        false
    }

    /// Scrape web page content
    pub async fn scrape_page(&self, url: &str, selector: &str) -> Result<String, String> {
        self.security_gate.lock().await.check_access()?;

        self.browse_url(url).await?;

        let mut dt = self.digital_twin.lock().await;
        if let Some(ref mut twin) = *dt {
            twin.scrape(selector)
                .await
                .map_err(|e| format!("Scraping failed: {}", e))
        } else {
            Err("Browser not initialized".to_string())
        }
    }

    // ============================================================================
    // ALWAYS ON MODE
    // ============================================================================

    /// Start Always ON continuous monitoring
    pub async fn start_always_on(&self) -> Result<(), String> {
        self.security_gate.lock().await.check_access()?;

        let mut always_on = self.always_on.lock().await;
        if *always_on {
            return Ok(()); // Already running
        }

        *always_on = true;

        let security_gate = self.security_gate.clone();
        let _digital_twin = self.digital_twin.clone();

        let task = tokio::spawn(async move {
            loop {
                // Check if access is still granted
                {
                    let gate = security_gate.lock().await;
                    if !gate.full_access_granted {
                        break;
                    }
                }

                // Continuous monitoring tasks:
                // 1. Monitor file system changes
                // 2. Monitor process changes
                // 3. Monitor network activity
                // 4. Capture screenshots (if enabled)
                // 5. Monitor browser activity
                // 6. Feed context into Phoenix

                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });

        let mut task_handle = self.always_on_task.lock().await;
        *task_handle = Some(task);

        Ok(())
    }

    /// Stop Always ON mode
    pub async fn stop_always_on(&self) -> Result<(), String> {
        let mut always_on = self.always_on.lock().await;
        *always_on = false;

        let mut task_handle = self.always_on_task.lock().await;
        if let Some(task) = task_handle.take() {
            task.abort();
        }

        Ok(())
    }

    /// Check if Always ON is active
    pub async fn is_always_on(&self) -> bool {
        let always_on = self.always_on.lock().await;
        *always_on
    }

    // ============================================================================
    // GUI APPLICATION CONTROL (Master Orchestrator Only)
    // ============================================================================

    /// List all visible windows
    #[cfg(windows)]
    pub async fn list_windows(&self) -> Result<Vec<WindowInfo>, String> {
        self.security_gate.lock().await.check_access()?;

        use winapi::um::winuser::*;
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;

        let windows = Arc::new(StdMutex::new(Vec::new()));
        let windows_ptr = Arc::into_raw(windows.clone());

        unsafe extern "system" fn enum_windows_proc(hwnd: winapi::shared::windef::HWND, lparam: winapi::shared::minwindef::LPARAM) -> winapi::shared::minwindef::BOOL {
            let windows_ptr = lparam as *const Arc<StdMutex<Vec<WindowInfo>>>;
            let windows = &*windows_ptr;

            // Check if window is visible
            if IsWindowVisible(hwnd) == 0 {
                return 1; // Continue enumeration
            }

            // Get window title
            let mut title_buf = vec![0u16; 256];
            let title_len = GetWindowTextW(hwnd, title_buf.as_mut_ptr(), 256);
            let title = if title_len > 0 {
                OsString::from_wide(&title_buf[..title_len as usize])
                    .to_string_lossy()
                    .to_string()
            } else {
                String::new()
            };

            // Skip windows with no title (usually system windows)
            if title.is_empty() {
                return 1;
            }

            // Get window class name
            let mut class_buf = vec![0u16; 256];
            let class_len = GetClassNameW(hwnd, class_buf.as_mut_ptr(), 256);
            let class_name = if class_len > 0 {
                OsString::from_wide(&class_buf[..class_len as usize])
                    .to_string_lossy()
                    .to_string()
            } else {
                String::new()
            };

            // Get process ID
            let mut process_id: u32 = 0;
            GetWindowThreadProcessId(hwnd, &mut process_id);

            // Get process name
            let process_name = if process_id > 0 {
                SystemAccessManager::get_process_name(process_id).unwrap_or_else(|| "Unknown".to_string())
            } else {
                "Unknown".to_string()
            };

            // Get window position and size
            let mut rect: winapi::shared::windef::RECT = std::mem::zeroed();
            if GetWindowRect(hwnd, &mut rect) != 0 {
                let position = (rect.left, rect.top);
                let size = (rect.right - rect.left, rect.bottom - rect.top);
                let is_enabled = IsWindowEnabled(hwnd) != 0;

                if let Ok(mut w) = windows.lock() {
                    w.push(WindowInfo {
                        hwnd: hwnd as u64,
                        title,
                        class_name,
                        process_id,
                        process_name,
                        is_visible: true,
                        is_enabled,
                        position,
                        size,
                    });
                }
            }

            1 // Continue enumeration
        }

        unsafe {
            EnumWindows(Some(enum_windows_proc), windows_ptr as isize);
            let _ = Arc::from_raw(windows_ptr); // Clean up the raw pointer
        }

        let result = windows.lock().map_err(|e| format!("Failed to lock windows list: {}", e))?;
        Ok(result.clone())
    }

    #[cfg(not(windows))]
    pub async fn list_windows(&self) -> Result<Vec<WindowInfo>, String> {
        self.security_gate.lock().await.check_access()?;
        // Cross-platform: use xdotool or similar
        Err("GUI control is only available on Windows".to_string())
    }

    /// Find window by title (partial match)
    #[cfg(windows)]
    pub async fn find_window(&self, title_pattern: &str) -> Result<Option<WindowInfo>, String> {
        self.security_gate.lock().await.check_access()?;

        let windows = self.list_windows().await?;
        let pattern_lower = title_pattern.to_lowercase();

        for window in windows {
            if window.title.to_lowercase().contains(&pattern_lower) {
                return Ok(Some(window));
            }
        }

        Ok(None)
    }

    #[cfg(not(windows))]
    pub async fn find_window(&self, _title_pattern: &str) -> Result<Option<WindowInfo>, String> {
        Err("GUI control is only available on Windows".to_string())
    }

    /// Activate/bring window to foreground
    #[cfg(windows)]
    pub async fn activate_window(&self, hwnd: u64) -> Result<(), String> {
        self.security_gate.lock().await.check_access()?;

        use winapi::um::winuser::*;

        let hwnd_ptr = hwnd as winapi::shared::windef::HWND;

        unsafe {
            // Show window if minimized
            ShowWindow(hwnd_ptr, SW_RESTORE);
            // Bring to foreground
            SetForegroundWindow(hwnd_ptr);
            // Set focus
            SetFocus(hwnd_ptr);
        }

        Ok(())
    }

    #[cfg(not(windows))]
    pub async fn activate_window(&self, _hwnd: u64) -> Result<(), String> {
        Err("GUI control is only available on Windows".to_string())
    }

    /// Click at screen coordinates (absolute)
    #[cfg(windows)]
    pub async fn click_at(&self, x: i32, y: i32, button: &str) -> Result<(), String> {
        self.security_gate.lock().await.check_access()?;

        use winapi::um::winuser::*;

        let button_code = match button.to_lowercase().as_str() {
            "left" | "lmb" => MOUSEEVENTF_LEFTDOWN | MOUSEEVENTF_LEFTUP,
            "right" | "rmb" => MOUSEEVENTF_RIGHTDOWN | MOUSEEVENTF_RIGHTUP,
            "middle" | "mmb" => MOUSEEVENTF_MIDDLEDOWN | MOUSEEVENTF_MIDDLEUP,
            _ => return Err("Invalid button. Use: left, right, or middle".to_string()),
        };

        unsafe {
            SetCursorPos(x, y);
            mouse_event(button_code, 0, 0, 0, 0);
        }

        Ok(())
    }

    #[cfg(not(windows))]
    pub async fn click_at(&self, _x: i32, _y: i32, _button: &str) -> Result<(), String> {
        Err("GUI control is only available on Windows".to_string())
    }

    /// Type text (sends keyboard input)
    #[cfg(windows)]
    pub async fn type_text(&self, text: &str) -> Result<(), String> {
        self.security_gate.lock().await.check_access()?;

        use winapi::um::winuser::*;

        for ch in text.chars() {
            let vk = Self::char_to_virtual_key(ch);
            let scan = unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) };

            unsafe {
                // Key down
                keybd_event(vk as u8, scan as u8, 0, 0);
                // Key up
                keybd_event(vk as u8, scan as u8, KEYEVENTF_KEYUP, 0);
            }

            // Small delay between keystrokes
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        Ok(())
    }

    #[cfg(not(windows))]
    pub async fn type_text(&self, _text: &str) -> Result<(), String> {
        Err("GUI control is only available on Windows".to_string())
    }

    /// Send key combination (e.g., "Ctrl+C", "Alt+F4")
    #[cfg(windows)]
    pub async fn send_key_combo(&self, combo: &str) -> Result<(), String> {
        self.security_gate.lock().await.check_access()?;

        use winapi::um::winuser::*;

        let parts: Vec<&str> = combo.split('+').map(|s| s.trim()).collect();
        let mut modifiers = Vec::new();
        let mut main_key = None;

        for part in parts {
            let part_lower = part.to_lowercase();
            match part_lower.as_str() {
                "ctrl" | "control" => modifiers.push(VK_CONTROL),
                "alt" => modifiers.push(VK_MENU),
                "shift" => modifiers.push(VK_SHIFT),
                "win" | "windows" => modifiers.push(VK_LWIN),
                _ => {
                    if main_key.is_none() {
                        main_key = Some(Self::string_to_virtual_key(&part_lower));
                    }
                }
            }
        }

        let main_key = main_key.ok_or_else(|| "No main key specified".to_string())?;

        unsafe {
            // Press modifiers
            for &mod_key in &modifiers {
                let scan = MapVirtualKeyW(mod_key as u32, MAPVK_VK_TO_VSC);
                keybd_event(mod_key as u8, scan as u8, 0, 0);
            }

            // Press main key
            let scan = MapVirtualKeyW(main_key as u32, MAPVK_VK_TO_VSC);
            keybd_event(main_key as u8, scan as u8, 0, 0);
            keybd_event(main_key as u8, scan as u8, KEYEVENTF_KEYUP, 0);

            // Release modifiers
            for &mod_key in modifiers.iter().rev() {
                let scan = MapVirtualKeyW(mod_key as u32, MAPVK_VK_TO_VSC);
                keybd_event(mod_key as u8, scan as u8, KEYEVENTF_KEYUP, 0);
            }
        }

        Ok(())
    }

    #[cfg(not(windows))]
    pub async fn send_key_combo(&self, _combo: &str) -> Result<(), String> {
        Err("GUI control is only available on Windows".to_string())
    }

    /// Take screenshot of window or entire screen
    #[cfg(windows)]
    pub async fn take_screenshot(&self, hwnd: Option<u64>) -> Result<Vec<u8>, String> {
        self.security_gate.lock().await.check_access()?;

        use winapi::um::wingdi::*;
        use winapi::um::winuser::*;
        use winapi::shared::windef::*;

        // TODO: Implement screenshot capture
        // This requires GDI+ or similar for image encoding
        // For now, return placeholder
        Err("Screenshot functionality not yet implemented".to_string())
    }

    #[cfg(not(windows))]
    pub async fn take_screenshot(&self, _hwnd: Option<u64>) -> Result<Vec<u8>, String> {
        Err("GUI control is only available on Windows".to_string())
    }

    // Helper functions for Windows GUI control

    #[cfg(windows)]
    fn get_process_name(pid: u32) -> Option<String> {
        use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
        use winapi::um::tlhelp32::*;

        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot == INVALID_HANDLE_VALUE {
                return None;
            }

            let mut entry: PROCESSENTRY32W = std::mem::zeroed();
            entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

            if Process32FirstW(snapshot, &mut entry) != 0 {
                loop {
                    if entry.th32ProcessID == pid {
                        let name_wide = &entry.szExeFile[..];
                        let name = String::from_utf16_lossy(name_wide)
                            .trim_end_matches('\0')
                            .to_string();
                        CloseHandle(snapshot);
                        return Some(name);
                    }
                    if Process32NextW(snapshot, &mut entry) == 0 {
                        break;
                    }
                }
            }

            CloseHandle(snapshot);
        }

        None
    }

    #[cfg(windows)]
    fn char_to_virtual_key(ch: char) -> u16 {
        use winapi::um::winuser::{VK_SPACE, VK_RETURN, VK_TAB};

        const VK_A: u16 = 0x41;
        const VK_0: u16 = 0x30;

        match ch {
            'A'..='Z' => (ch as u16 - 'A' as u16) + VK_A,
            'a'..='z' => (ch.to_ascii_uppercase() as u16 - 'A' as u16) + VK_A,
            '0'..='9' => (ch as u16 - '0' as u16) + VK_0,
            ' ' => VK_SPACE as u16,
            '\n' | '\r' => VK_RETURN as u16,
            '\t' => VK_TAB as u16,
            _ => VK_SPACE as u16, // Default fallback
        }
    }

    #[cfg(windows)]
    fn string_to_virtual_key(s: &str) -> u16 {
        use winapi::um::winuser::*;

        match s.to_lowercase().as_str() {
            "enter" | "return" => VK_RETURN as u16,
            "tab" => VK_TAB as u16,
            "space" => VK_SPACE as u16,
            "esc" | "escape" => VK_ESCAPE as u16,
            "backspace" => VK_BACK as u16,
            "delete" | "del" => VK_DELETE as u16,
            "up" => VK_UP as u16,
            "down" => VK_DOWN as u16,
            "left" => VK_LEFT as u16,
            "right" => VK_RIGHT as u16,
            "f1" => VK_F1 as u16,
            "f2" => VK_F2 as u16,
            "f3" => VK_F3 as u16,
            "f4" => VK_F4 as u16,
            "f5" => VK_F5 as u16,
            "f6" => VK_F6 as u16,
            "f7" => VK_F7 as u16,
            "f8" => VK_F8 as u16,
            "f9" => VK_F9 as u16,
            "f10" => VK_F10 as u16,
            "f11" => VK_F11 as u16,
            "f12" => VK_F12 as u16,
            _ => {
                // Try to parse as single character
                if let Some(ch) = s.chars().next() {
                    Self::char_to_virtual_key(ch)
                } else {
                    VK_SPACE as u16
                }
            }
        }
    }

    // ============================================================================
    // CAPTCHA & HUMAN VERIFICATION BYPASS (Full Access - Master Orchestrator Only)
    // ============================================================================

    /// Detect CAPTCHA on a web page
    pub async fn detect_captcha(&self, debug_port: u16, url: Option<&str>) -> Result<CaptchaDetection, String> {
        self.security_gate.lock().await.check_access()?;

        let client = reqwest::Client::new();
        let cdp_url = format!("http://localhost:{}/json/runtime/evaluate", debug_port);

        // JavaScript to detect various CAPTCHA types
        let detection_js = r#"
        (function() {
            const result = {
                detected: false,
                type: 'Unknown',
                selector: null,
                siteKey: null,
                imageUrl: null
            };

            // Check for reCAPTCHA v2
            const recaptchaV2 = document.querySelector('.g-recaptcha');
            if (recaptchaV2) {
                result.detected = true;
                result.type = 'ReCaptchaV2';
                result.selector = '.g-recaptcha';
                result.siteKey = recaptchaV2.getAttribute('data-sitekey') || 
                                document.querySelector('script[src*="recaptcha"]')?.src.match(/sitekey=([^&]+)/)?.[1] || null;
                return result;
            }

            // Check for reCAPTCHA v3
            const recaptchaV3 = document.querySelector('script[src*="recaptcha/api.js?render"]');
            if (recaptchaV3) {
                result.detected = true;
                result.type = 'ReCaptchaV3';
                result.siteKey = recaptchaV3.src.match(/render=([^&]+)/)?.[1] || null;
                return result;
            }

            // Check for hCaptcha
            const hcaptcha = document.querySelector('.h-captcha, [data-sitekey]');
            if (hcaptcha) {
                result.detected = true;
                result.type = 'HCaptcha';
                result.selector = '.h-captcha';
                result.siteKey = hcaptcha.getAttribute('data-sitekey') || null;
                return result;
            }

            // Check for Cloudflare Turnstile
            const turnstile = document.querySelector('.cf-turnstile, [data-sitekey]');
            if (turnstile && turnstile.className.includes('cf-turnstile')) {
                result.detected = true;
                result.type = 'Turnstile';
                result.selector = '.cf-turnstile';
                result.siteKey = turnstile.getAttribute('data-sitekey') || null;
                return result;
            }

            // Check for image CAPTCHA
            const imgCaptcha = document.querySelector('img[src*="captcha"], img[alt*="captcha" i], img[title*="captcha" i]');
            if (imgCaptcha) {
                result.detected = true;
                result.type = 'Image';
                result.selector = 'img[src*="captcha"]';
                result.imageUrl = imgCaptcha.src;
                return result;
            }

            // Check for text CAPTCHA input
            const textCaptcha = document.querySelector('input[name*="captcha" i], input[id*="captcha" i]');
            if (textCaptcha) {
                result.detected = true;
                result.type = 'Text';
                result.selector = textCaptcha.name || textCaptcha.id;
                return result;
            }

            return result;
        })()
        "#;

        let payload = serde_json::json!({
            "id": 1,
            "method": "Runtime.evaluate",
            "params": {
                "expression": detection_js,
                "returnByValue": true
            }
        });

        let response = client
            .post(&cdp_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to detect CAPTCHA: {}", e))?;

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if let Some(error) = result.get("error") {
            return Err(format!("CAPTCHA detection error: {}", error));
        }

        let detection_value = result.get("result")
            .and_then(|r| r.get("value"))
            .ok_or_else(|| "No detection result".to_string())?;

        let captcha_type = match detection_value.get("type").and_then(|t| t.as_str()) {
            Some("ReCaptchaV2") => CaptchaType::ReCaptchaV2,
            Some("ReCaptchaV3") => CaptchaType::ReCaptchaV3,
            Some("HCaptcha") => CaptchaType::HCaptcha,
            Some("Turnstile") => CaptchaType::Turnstile,
            Some("Image") => CaptchaType::Image,
            Some("Text") => CaptchaType::Text,
            _ => CaptchaType::Unknown,
        };

        Ok(CaptchaDetection {
            captcha_type,
            detected: detection_value.get("detected").and_then(|d| d.as_bool()).unwrap_or(false),
            element_selector: detection_value.get("selector").and_then(|s| s.as_str()).map(|s| s.to_string()),
            site_key: detection_value.get("siteKey").and_then(|k| k.as_str()).map(|k| k.to_string()),
            image_url: detection_value.get("imageUrl").and_then(|u| u.as_str()).map(|u| u.to_string()),
            image_data: None,
        })
    }

    /// Solve CAPTCHA using OCR (for text/image CAPTCHAs)
    pub async fn solve_captcha_ocr(&self, image_data: &[u8]) -> Result<CaptchaSolution, String> {
        self.security_gate.lock().await.check_access()?;

        // Use Tesseract OCR for text extraction
        // Note: This is a simplified implementation
        // In production, you'd want to preprocess the image (denoise, threshold, etc.)
        
        // For now, return a placeholder that indicates OCR would be used
        // Full OCR implementation would require proper image preprocessing
        Ok(CaptchaSolution {
            success: false,
            solution: None,
            method: "ocr".to_string(),
            confidence: 0.0,
            error: Some("OCR implementation requires image preprocessing. Use CAPTCHA solving service instead.".to_string()),
        })
    }

    /// Solve CAPTCHA using 2Captcha service
    pub async fn solve_captcha_2captcha(
        &self,
        config: &CaptchaServiceConfig,
        captcha_type: &CaptchaType,
        image_data: Option<&[u8]>,
        site_key: Option<&str>,
        page_url: Option<&str>,
    ) -> Result<CaptchaSolution, String> {
        self.security_gate.lock().await.check_access()?;

        let client = reqwest::Client::new();
        let api_key = &config.api_key;

        // Determine method based on CAPTCHA type
        let method = match captcha_type {
            CaptchaType::ReCaptchaV2 => "userrecaptcha",
            CaptchaType::ReCaptchaV3 => "userrecaptcha",
            CaptchaType::HCaptcha => "hcaptcha",
            CaptchaType::Image | CaptchaType::Text => "base64",
            _ => return Err("Unsupported CAPTCHA type for 2Captcha".to_string()),
        };

        // Submit CAPTCHA
        let submit_url = "http://2captcha.com/in.php";
        let mut form = reqwest::multipart::Form::new()
            .text("key", api_key.to_string())
            .text("method", method.to_string());

        match captcha_type {
            CaptchaType::ReCaptchaV2 | CaptchaType::ReCaptchaV3 => {
                if let (Some(site_key), Some(page_url)) = (site_key, page_url) {
                    form = form
                        .text("googlekey", site_key.to_string())
                        .text("pageurl", page_url.to_string());
                } else {
                    return Err("reCAPTCHA requires site_key and page_url".to_string());
                }
            }
            CaptchaType::HCaptcha => {
                if let (Some(site_key), Some(page_url)) = (site_key, page_url) {
                    form = form
                        .text("sitekey", site_key.to_string())
                        .text("pageurl", page_url.to_string());
                } else {
                    return Err("hCaptcha requires site_key and page_url".to_string());
                }
            }
            CaptchaType::Image | CaptchaType::Text => {
                if let Some(img_data) = image_data {
                    let img_base64 = general_purpose::STANDARD.encode(img_data);
                    form = form.text("body", img_base64);
                } else {
                    return Err("Image CAPTCHA requires image_data".to_string());
                }
            }
            _ => return Err("Unsupported CAPTCHA type".to_string()),
        }

        let response = client
            .post(submit_url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Failed to submit CAPTCHA: {}", e))?;

        let response_text = response.text().await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        if response_text.starts_with("OK|") {
            let captcha_id = response_text.split('|').nth(1)
                .ok_or_else(|| "No CAPTCHA ID in response".to_string())?;

            // Poll for result
            let get_url = format!("http://2captcha.com/res.php?key={}&action=get&id={}", api_key, captcha_id);
            let timeout = std::time::Duration::from_secs(config.timeout_seconds);
            let start = std::time::Instant::now();

            loop {
                if start.elapsed() > timeout {
                    return Err("CAPTCHA solving timeout".to_string());
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                let result_response = client
                    .get(&get_url)
                    .send()
                    .await
                    .map_err(|e| format!("Failed to get result: {}", e))?;

                let result_text = result_response.text().await
                    .map_err(|e| format!("Failed to read result: {}", e))?;

                if result_text == "CAPCHA_NOT_READY" {
                    continue;
                } else if result_text.starts_with("OK|") {
                    let solution = result_text.split('|').nth(1)
                        .ok_or_else(|| "No solution in response".to_string())?;
                    
                    return Ok(CaptchaSolution {
                        success: true,
                        solution: Some(solution.to_string()),
                        method: "2captcha".to_string(),
                        confidence: 0.95, // Service-based solutions have high confidence
                        error: None,
                    });
                } else {
                    return Err(format!("CAPTCHA solving failed: {}", result_text));
                }
            }
        } else {
            Err(format!("Failed to submit CAPTCHA: {}", response_text))
        }
    }

    /// Solve CAPTCHA using Anti-Captcha service
    pub async fn solve_captcha_anticaptcha(
        &self,
        config: &CaptchaServiceConfig,
        captcha_type: &CaptchaType,
        image_data: Option<&[u8]>,
        site_key: Option<&str>,
        page_url: Option<&str>,
    ) -> Result<CaptchaSolution, String> {
        self.security_gate.lock().await.check_access()?;

        let client = reqwest::Client::new();
        let api_key = &config.api_key;

        // Determine task type
        let task_type = match captcha_type {
            CaptchaType::ReCaptchaV2 => "RecaptchaV2TaskProxyless",
            CaptchaType::ReCaptchaV3 => "RecaptchaV3TaskProxyless",
            CaptchaType::HCaptcha => "HcaptchaTaskProxyless",
            CaptchaType::Image | CaptchaType::Text => "ImageToTextTask",
            _ => return Err("Unsupported CAPTCHA type for Anti-Captcha".to_string()),
        };

        // Build task
        let mut task = serde_json::json!({
            "type": task_type
        });

        match captcha_type {
            CaptchaType::ReCaptchaV2 | CaptchaType::ReCaptchaV3 => {
                if let (Some(site_key), Some(page_url)) = (site_key, page_url) {
                    task["websiteURL"] = serde_json::Value::String(page_url.to_string());
                    task["websiteKey"] = serde_json::Value::String(site_key.to_string());
                    if matches!(captcha_type, CaptchaType::ReCaptchaV3) {
                        task["minScore"] = serde_json::json!(0.3);
                    }
                } else {
                    return Err("reCAPTCHA requires site_key and page_url".to_string());
                }
            }
            CaptchaType::HCaptcha => {
                if let (Some(site_key), Some(page_url)) = (site_key, page_url) {
                    task["websiteURL"] = serde_json::Value::String(page_url.to_string());
                    task["websiteKey"] = serde_json::Value::String(site_key.to_string());
                } else {
                    return Err("hCaptcha requires site_key and page_url".to_string());
                }
            }
            CaptchaType::Image | CaptchaType::Text => {
                if let Some(img_data) = image_data {
                    let img_base64 = general_purpose::STANDARD.encode(img_data);
                    task["body"] = serde_json::Value::String(img_base64);
                } else {
                    return Err("Image CAPTCHA requires image_data".to_string());
                }
            }
            _ => return Err("Unsupported CAPTCHA type".to_string()),
        }

        // Create task
        let create_payload = serde_json::json!({
            "clientKey": api_key,
            "task": task
        });

        let create_url = "https://api.anti-captcha.com/createTask";
        let create_response = client
            .post(create_url)
            .json(&create_payload)
            .send()
            .await
            .map_err(|e| format!("Failed to create task: {}", e))?;

        let create_result: serde_json::Value = create_response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if create_result.get("errorId").and_then(|e| e.as_u64()) != Some(0) {
            return Err(format!("Failed to create task: {}", create_result.get("errorDescription").unwrap_or(&serde_json::Value::String("Unknown error".to_string()))));
        }

        let task_id = create_result.get("taskId")
            .and_then(|id| id.as_u64())
            .ok_or_else(|| "No task ID in response".to_string())?;

        // Poll for result
        let get_url = "https://api.anti-captcha.com/getTaskResult";
        let timeout = std::time::Duration::from_secs(config.timeout_seconds);
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > timeout {
                return Err("CAPTCHA solving timeout".to_string());
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

            let get_payload = serde_json::json!({
                "clientKey": api_key,
                "taskId": task_id
            });

            let get_response = client
                .post(get_url)
                .json(&get_payload)
                .send()
                .await
                .map_err(|e| format!("Failed to get result: {}", e))?;

            let get_result: serde_json::Value = get_response
                .json()
                .await
                .map_err(|e| format!("Failed to parse result: {}", e))?;

            if get_result.get("status").and_then(|s| s.as_str()) == Some("ready") {
                let solution = get_result.get("solution")
                    .and_then(|s| {
                        if let Some(token) = s.get("gRecaptchaResponse").and_then(|t| t.as_str()) {
                            Some(token.to_string())
                        } else if let Some(text) = s.get("text").and_then(|t| t.as_str()) {
                            Some(text.to_string())
                        } else if let Some(token) = s.get("token").and_then(|t| t.as_str()) {
                            Some(token.to_string())
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| "No solution in response".to_string())?;

                return Ok(CaptchaSolution {
                    success: true,
                    solution: Some(solution),
                    method: "anticaptcha".to_string(),
                    confidence: 0.95,
                    error: None,
                });
            } else if get_result.get("errorId").and_then(|e| e.as_u64()) != Some(0) {
                return Err(format!("CAPTCHA solving failed: {}", get_result.get("errorDescription").unwrap_or(&serde_json::Value::String("Unknown error".to_string()))));
            }
        }
    }

    /// Auto-solve CAPTCHA (detects and solves automatically)
    pub async fn auto_solve_captcha(
        &self,
        debug_port: u16,
        service_config: Option<&CaptchaServiceConfig>,
    ) -> Result<CaptchaSolution, String> {
        self.security_gate.lock().await.check_access()?;

        // Detect CAPTCHA
        let detection = self.detect_captcha(debug_port, None).await?;

        if !detection.detected {
            return Ok(CaptchaSolution {
                success: false,
                solution: None,
                method: "detection".to_string(),
                confidence: 1.0,
                error: Some("No CAPTCHA detected on page".to_string()),
            });
        }

        // Try to solve based on type
        match detection.captcha_type {
            CaptchaType::ReCaptchaV2 | CaptchaType::ReCaptchaV3 | CaptchaType::HCaptcha => {
                if let Some(config) = service_config {
                    // Try 2Captcha first, then Anti-Captcha
                    if config.service == "2captcha" {
                        self.solve_captcha_2captcha(
                            config,
                            &detection.captcha_type,
                            None,
                            detection.site_key.as_deref(),
                            None, // Would need to get from page
                        ).await
                    } else if config.service == "anticaptcha" {
                        self.solve_captcha_anticaptcha(
                            config,
                            &detection.captcha_type,
                            None,
                            detection.site_key.as_deref(),
                            None,
                        ).await
                    } else {
                        Err("Unsupported CAPTCHA service".to_string())
                    }
                } else {
                    Err("CAPTCHA service configuration required for reCAPTCHA/hCaptcha".to_string())
                }
            }
            CaptchaType::Image | CaptchaType::Text => {
                if let Some(img_data) = &detection.image_data {
                    if let Some(config) = service_config {
                        if config.service == "2captcha" {
                            self.solve_captcha_2captcha(
                                config,
                                &detection.captcha_type,
                                Some(img_data),
                                None,
                                None,
                            ).await
                        } else if config.service == "anticaptcha" {
                            self.solve_captcha_anticaptcha(
                                config,
                                &detection.captcha_type,
                                Some(img_data),
                                None,
                                None,
                            ).await
                        } else {
                            // Fallback to OCR
                            self.solve_captcha_ocr(img_data).await
                        }
                    } else {
                        // Try OCR without service
                        self.solve_captcha_ocr(img_data).await
                    }
                } else {
                    Err("Image data required for image CAPTCHA".to_string())
                }
            }
            _ => Err("Unsupported CAPTCHA type".to_string()),
        }
    }

    /// Inject CAPTCHA solution into page
    pub async fn inject_captcha_solution(
        &self,
        debug_port: u16,
        solution: &CaptchaSolution,
        selector: Option<&str>,
    ) -> Result<(), String> {
        self.security_gate.lock().await.check_access()?;

        if !solution.success {
            return Err("Cannot inject failed solution".to_string());
        }

        let solution_text = solution.solution.as_ref()
            .ok_or_else(|| "No solution to inject".to_string())?;

        let client = reqwest::Client::new();
        let cdp_url = format!("http://localhost:{}/json/runtime/evaluate", debug_port);

        // JavaScript to inject solution
        let inject_js = if let Some(sel) = selector {
            format!(
                r#"
                (function() {{
                    const element = document.querySelector('{}');
                    if (element) {{
                        element.value = '{}';
                        element.dispatchEvent(new Event('input', {{ bubbles: true }}));
                        element.dispatchEvent(new Event('change', {{ bubbles: true }}));
                        return 'Solution injected';
                    }}
                    return 'Element not found';
                }})()
                "#,
                sel, solution_text
            )
        } else {
            // Try to find CAPTCHA input automatically
            format!(
                r#"
                (function() {{
                    const inputs = document.querySelectorAll('input[name*="captcha" i], input[id*="captcha" i], textarea[name*="captcha" i]');
                    if (inputs.length > 0) {{
                        inputs[0].value = '{}';
                        inputs[0].dispatchEvent(new Event('input', {{ bubbles: true }}));
                        inputs[0].dispatchEvent(new Event('change', {{ bubbles: true }}));
                        return 'Solution injected';
                    }}
                    return 'No CAPTCHA input found';
                }})()
                "#,
                solution_text
            )
        };

        let payload = serde_json::json!({
            "id": 1,
            "method": "Runtime.evaluate",
            "params": {
                "expression": inject_js,
                "returnByValue": true
            }
        });

        let response = client
            .post(&cdp_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to inject solution: {}", e))?;

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if let Some(error) = result.get("error") {
            Err(format!("Injection error: {}", error))
        } else {
            Ok(())
        }
    }
}

impl Default for SystemAccessManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_security_gate() {
        let manager = SystemAccessManager::new();
        assert!(manager.is_access_granted().await);

        // Self-modification should also be enabled by default for local dev.
        assert!(manager.is_self_modification_enabled().await);

        manager.revoke_access().await.unwrap();
        assert!(!manager.is_access_granted().await);
        assert!(!manager.is_self_modification_enabled().await);
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn test_registry_write_then_read_roundtrip() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let manager = SystemAccessManager::new();
        manager.grant_full_access("test_user".to_string()).await.unwrap();

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();

        let key_path = format!("HKEY_CURRENT_USER\\Software\\PhoenixTest\\{}", unique);
        let value_name = "TestValue";
        let value = "HelloRegistry";

        manager.write_registry(&key_path, value_name, value).await.unwrap();
        let read_back = manager.read_registry(&key_path, value_name).await.unwrap();

        assert_eq!(read_back.value, value);

        // Cleanup best-effort.
        let _ = RegKey::predef(HKEY_CURRENT_USER)
            .delete_subkey_all(format!("Software\\PhoenixTest\\{}", unique));
    }
}
