//! Unified port configuration for Phoenix AGI (PAGI) services.
//!
//! Provides consistent port configuration across all services with:
//! - Sensible defaults
//! - Environment variable overrides
//! - Validation
//! - Single source of truth

use std::env;

/// Port configuration for Phoenix Web UI
pub struct PhoenixWebPort;

impl PhoenixWebPort {
    /// Default bind address
    pub const DEFAULT_BIND: &'static str = "127.0.0.1:8888";
    
    /// Environment variable name
    pub const ENV_VAR: &'static str = "PHOENIX_WEB_BIND";
    
    /// Get bind address from env or default
    pub fn bind() -> String {
        env::var(Self::ENV_VAR).unwrap_or_else(|_| Self::DEFAULT_BIND.to_string())
    }
}

/// Port configuration for Vital Pulse Collector (Telemetrist)
pub struct VitalPulseCollectorPort;

impl VitalPulseCollectorPort {
    /// Default bind address
    pub const DEFAULT_BIND: &'static str = "127.0.0.1:5002";
    
    /// Environment variable name
    pub const ENV_VAR: &'static str = "TELEMETRIST_BIND";
    
    /// Get bind address from env or default
    pub fn bind() -> String {
        env::var(Self::ENV_VAR).unwrap_or_else(|_| Self::DEFAULT_BIND.to_string())
    }
}

/// Port configuration for Synaptic Pulse Distributor
pub struct SynapticPulseDistributorPort;

impl SynapticPulseDistributorPort {
    /// Default bind address
    pub const DEFAULT_BIND: &'static str = "127.0.0.1:5003";
    
    /// Environment variable name
    pub const ENV_VAR: &'static str = "PULSE_DISTRIBUTOR_BIND";
    
    /// Get bind address from env or default
    pub fn bind() -> String {
        env::var(Self::ENV_VAR).unwrap_or_else(|_| Self::DEFAULT_BIND.to_string())
    }
}

/// Port configuration for Chrome DevTools Protocol
pub struct ChromeDevToolsPort;

impl ChromeDevToolsPort {
    /// Default port
    pub const DEFAULT_PORT: u16 = 9222;
    
    /// Environment variable name
    pub const ENV_VAR: &'static str = "CHROME_DEBUG_PORT";
    
    /// Get port from env or default
    pub fn port() -> u16 {
        env::var(Self::ENV_VAR)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(Self::DEFAULT_PORT)
    }
    
    /// Get full CDP URL
    pub fn url() -> String {
        format!("http://127.0.0.1:{}", Self::port())
    }
}

/// Port configuration for Selenium WebDriver
pub struct SeleniumPort;

impl SeleniumPort {
    /// Default hub URL
    pub const DEFAULT_HUB_URL: &'static str = "http://localhost:4444/wd/hub";
    
    /// Environment variable name
    pub const ENV_VAR: &'static str = "SELENIUM_HUB_URL";
    
    /// Get hub URL from env or default
    pub fn hub_url() -> String {
        env::var(Self::ENV_VAR).unwrap_or_else(|_| Self::DEFAULT_HUB_URL.to_string())
    }
}

/// Port configuration for Frontend Dev Server (Vite)
pub struct FrontendDevPort;

impl FrontendDevPort {
    /// Default port
    pub const DEFAULT_PORT: u16 = 3000;
    
    /// Environment variable name (Vite uses VITE_PORT)
    pub const ENV_VAR: &'static str = "VITE_PORT";
    
    /// Get port from env or default
    pub fn port() -> u16 {
        env::var(Self::ENV_VAR)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(Self::DEFAULT_PORT)
    }
}

/// Validate that all configured ports are unique
pub fn validate_ports() -> Result<(), String> {
    let phoenix_web = PhoenixWebPort::bind();
    let vital_pulse = VitalPulseCollectorPort::bind();
    let pulse_dist = SynapticPulseDistributorPort::bind();
    
    // Extract ports from bind addresses
    let extract_port = |bind: &str| -> Option<u16> {
        bind.split(':').last()?.parse().ok()
    };
    
    let ports: Vec<Option<u16>> = vec![
        extract_port(&phoenix_web),
        extract_port(&vital_pulse),
        extract_port(&pulse_dist),
        Some(ChromeDevToolsPort::port()),
        Some(FrontendDevPort::port()),
    ];
    
    let mut seen = std::collections::HashSet::new();
    for port_opt in ports {
        if let Some(port) = port_opt {
            if !seen.insert(port) {
                return Err(format!("Port conflict detected: port {} is used by multiple services", port));
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_ports() {
        assert_eq!(PhoenixWebPort::DEFAULT_BIND, "127.0.0.1:8888");
        assert_eq!(VitalPulseCollectorPort::DEFAULT_BIND, "127.0.0.1:5002");
        assert_eq!(SynapticPulseDistributorPort::DEFAULT_BIND, "127.0.0.1:5003");
        assert_eq!(ChromeDevToolsPort::DEFAULT_PORT, 9222);
        assert_eq!(FrontendDevPort::DEFAULT_PORT, 3000);
    }
    
    #[test]
    fn test_port_validation() {
        // Default ports should be unique
        assert!(validate_ports().is_ok());
    }
}
