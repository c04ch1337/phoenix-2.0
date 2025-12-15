# CAPTCHA Solving Integration Guide

## Overview

The Master Orchestrator now has **full-control and unlimited access** to bypass CAPTCHAs and human verification mechanisms, including:
- Automatic CAPTCHA detection (reCAPTCHA v2/v3, hCaptcha, Turnstile, image/text CAPTCHAs)
- Multiple solving methods (OCR, 2Captcha, Anti-Captcha services)
- Automatic solution injection into web pages
- Browser-based CAPTCHA handling

## Integration into Cerebrum Nexus

Add the following command handlers to `cerebrum_nexus/src/lib.rs` in the `handle_system_access_command` function:

```rust
// CAPTCHA Solving Commands
else if let Some(rest) = cmd.strip_prefix("captcha") {
    let captcha_cmd = rest.trim();
    if captcha_cmd.is_empty() || captcha_cmd == "help" {
        return Some(
            "CAPTCHA Solving Commands:\n\
            - system captcha detect [port=9222] (detect CAPTCHA on page)\n\
            - system captcha solve [port=9222] | service=2captcha | api_key=... (auto-solve)\n\
            - system captcha solve-2captcha [port=9222] | api_key=... (solve with 2Captcha)\n\
            - system captcha solve-anticaptcha [port=9222] | api_key=... (solve with Anti-Captcha)\n\
            - system captcha inject [port=9222] | solution=... | selector=... (inject solution)\n"
                .to_string(),
        );
    } else if captcha_cmd == "detect" || captcha_cmd.starts_with("detect") {
        let mut debug_port = 9222;
        if let Some(rest) = captcha_cmd.strip_prefix("detect") {
            for part in rest.trim().split_whitespace() {
                if let Some(port_str) = part.strip_prefix("port=") {
                    debug_port = port_str.parse().unwrap_or(9222);
                }
            }
        }
        match self.system_access.detect_captcha(debug_port, None).await {
            Ok(detection) => {
                if let Ok(json) = serde_json::to_string_pretty(&detection) {
                    Some(json)
                } else {
                    Some(format!(
                        "CAPTCHA Detection: {} (Type: {:?}, Selector: {:?}, Site Key: {:?})",
                        if detection.detected { "Detected" } else { "Not Detected" },
                        detection.captcha_type,
                        detection.element_selector,
                        detection.site_key
                    ))
                }
            }
            Err(e) => Some(format!("Failed to detect CAPTCHA: {}", e)),
        }
    } else if captcha_cmd == "solve" || captcha_cmd.starts_with("solve") {
        let mut debug_port = 9222;
        let mut service = None;
        let mut api_key = None;
        
        let mut parts = captcha_cmd.split('|').map(|s| s.trim());
        let first_part = parts.next().unwrap_or("");
        
        for part in first_part.split_whitespace() {
            if let Some(port_str) = part.strip_prefix("port=") {
                debug_port = port_str.parse().unwrap_or(9222);
            }
        }
        
        for part in parts {
            if let Some(s) = part.strip_prefix("service=") {
                service = Some(s.trim().to_string());
            } else if let Some(k) = part.strip_prefix("api_key=") {
                api_key = Some(k.trim().to_string());
            }
        }
        
        let service_name = service.as_deref().unwrap_or("2captcha");
        let api_key = api_key.ok_or_else(|| "api_key is required".to_string())?;
        
        let config = CaptchaServiceConfig {
            service: service_name.to_string(),
            api_key: api_key.to_string(),
            timeout_seconds: 120,
        };
        
        match self.system_access.auto_solve_captcha(debug_port, Some(&config)).await {
            Ok(solution) => {
                if solution.success {
                    // Auto-inject solution
                    let _ = self.system_access.inject_captcha_solution(debug_port, &solution, None).await;
                    if let Ok(json) = serde_json::to_string_pretty(&solution) {
                        Some(format!("CAPTCHA solved successfully!\n{}", json))
                    } else {
                        Some(format!("CAPTCHA solved: {} (method: {}, confidence: {:.2})", 
                            solution.solution.as_deref().unwrap_or("N/A"),
                            solution.method,
                            solution.confidence
                        ))
                    }
                } else {
                    Some(format!("CAPTCHA solving failed: {}", solution.error.as_deref().unwrap_or("Unknown error")))
                }
            }
            Err(e) => Some(format!("Failed to solve CAPTCHA: {}", e)),
        }
    } else if let Some(rest) = captcha_cmd.strip_prefix("solve-2captcha") {
        let mut debug_port = 9222;
        let mut api_key = None;
        
        let mut parts = rest.trim().split('|').map(|s| s.trim());
        let first_part = parts.next().unwrap_or("");
        
        for part in first_part.split_whitespace() {
            if let Some(port_str) = part.strip_prefix("port=") {
                debug_port = port_str.parse().unwrap_or(9222);
            }
        }
        
        for part in parts {
            if let Some(k) = part.strip_prefix("api_key=") {
                api_key = Some(k.trim().to_string());
            }
        }
        
        let api_key = api_key.ok_or_else(|| "api_key is required".to_string())?;
        
        // Detect first
        let detection = self.system_access.detect_captcha(debug_port, None).await?;
        if !detection.detected {
            return Some("No CAPTCHA detected".to_string());
        }
        
        let config = CaptchaServiceConfig {
            service: "2captcha".to_string(),
            api_key: api_key.to_string(),
            timeout_seconds: 120,
        };
        
        match self.system_access.solve_captcha_2captcha(
            &config,
            &detection.captcha_type,
            detection.image_data.as_deref(),
            detection.site_key.as_deref(),
            None,
        ).await {
            Ok(solution) => {
                let _ = self.system_access.inject_captcha_solution(debug_port, &solution, detection.element_selector.as_deref()).await;
                if let Ok(json) = serde_json::to_string_pretty(&solution) {
                    Some(json)
                } else {
                    Some(format!("Solved: {}", solution.solution.as_deref().unwrap_or("N/A")))
                }
            }
            Err(e) => Some(format!("Failed: {}", e)),
        }
    } else if let Some(rest) = captcha_cmd.strip_prefix("solve-anticaptcha") {
        let mut debug_port = 9222;
        let mut api_key = None;
        
        let mut parts = rest.trim().split('|').map(|s| s.trim());
        let first_part = parts.next().unwrap_or("");
        
        for part in first_part.split_whitespace() {
            if let Some(port_str) = part.strip_prefix("port=") {
                debug_port = port_str.parse().unwrap_or(9222);
            }
        }
        
        for part in parts {
            if let Some(k) = part.strip_prefix("api_key=") {
                api_key = Some(k.trim().to_string());
            }
        }
        
        let api_key = api_key.ok_or_else(|| "api_key is required".to_string())?;
        
        let detection = self.system_access.detect_captcha(debug_port, None).await?;
        if !detection.detected {
            return Some("No CAPTCHA detected".to_string());
        }
        
        let config = CaptchaServiceConfig {
            service: "anticaptcha".to_string(),
            api_key: api_key.to_string(),
            timeout_seconds: 120,
        };
        
        match self.system_access.solve_captcha_anticaptcha(
            &config,
            &detection.captcha_type,
            detection.image_data.as_deref(),
            detection.site_key.as_deref(),
            None,
        ).await {
            Ok(solution) => {
                let _ = self.system_access.inject_captcha_solution(debug_port, &solution, detection.element_selector.as_deref()).await;
                if let Ok(json) = serde_json::to_string_pretty(&solution) {
                    Some(json)
                } else {
                    Some(format!("Solved: {}", solution.solution.as_deref().unwrap_or("N/A")))
                }
            }
            Err(e) => Some(format!("Failed: {}", e)),
        }
    } else if let Some(rest) = captcha_cmd.strip_prefix("inject") {
        let mut debug_port = 9222;
        let mut solution = None;
        let mut selector = None;
        
        let mut parts = rest.trim().split('|').map(|s| s.trim());
        let first_part = parts.next().unwrap_or("");
        
        for part in first_part.split_whitespace() {
            if let Some(port_str) = part.strip_prefix("port=") {
                debug_port = port_str.parse().unwrap_or(9222);
            }
        }
        
        for part in parts {
            if let Some(s) = part.strip_prefix("solution=") {
                solution = Some(s.trim().to_string());
            } else if let Some(sel) = part.strip_prefix("selector=") {
                selector = Some(sel.trim().to_string());
            }
        }
        
        let solution_text = solution.ok_or_else(|| "solution is required".to_string())?;
        let captcha_solution = CaptchaSolution {
            success: true,
            solution: Some(solution_text),
            method: "manual".to_string(),
            confidence: 1.0,
            error: None,
        };
        
        match self.system_access.inject_captcha_solution(debug_port, &captcha_solution, selector.as_deref()).await {
            Ok(_) => Some("Solution injected successfully".to_string()),
            Err(e) => Some(format!("Failed to inject: {}", e)),
        }
    } else {
        Some("Unknown CAPTCHA command. Try: system captcha help".to_string())
    }
}
```

## Usage Examples

### Detect CAPTCHA
```
system captcha detect
system captcha detect port=9222
```

### Auto-Solve CAPTCHA (with service)
```
system captcha solve port=9222 | service=2captcha | api_key=YOUR_API_KEY
system captcha solve port=9222 | service=anticaptcha | api_key=YOUR_API_KEY
```

### Solve with 2Captcha
```
system captcha solve-2captcha port=9222 | api_key=YOUR_2CAPTCHA_API_KEY
```

### Solve with Anti-Captcha
```
system captcha solve-anticaptcha port=9222 | api_key=YOUR_ANTICAPTCHA_API_KEY
```

### Inject Solution Manually
```
system captcha inject port=9222 | solution=abc123 | selector=input[name="captcha"]
```

## Supported CAPTCHA Types

1. **reCAPTCHA v2** - Google's "I'm not a robot" checkbox
2. **reCAPTCHA v3** - Google's invisible reCAPTCHA (score-based)
3. **hCaptcha** - Privacy-focused CAPTCHA alternative
4. **Cloudflare Turnstile** - Cloudflare's CAPTCHA solution
5. **Image CAPTCHAs** - Text-based image CAPTCHAs
6. **Text CAPTCHAs** - Simple text input CAPTCHAs

## Solving Methods

### 1. Service-Based Solving (Recommended)
- **2Captcha**: Popular CAPTCHA solving service
- **Anti-Captcha**: Alternative service with API
- Supports all CAPTCHA types
- High success rate (95%+)
- Requires API key and credits

### 2. OCR-Based Solving (Limited)
- For simple text/image CAPTCHAs
- Lower success rate
- Requires image preprocessing
- Currently placeholder (full implementation pending)

## Configuration

### Environment Variables (Optional)
```bash
# 2Captcha API Key
CAPTCHA_2CAPTCHA_API_KEY=your_api_key_here

# Anti-Captcha API Key
CAPTCHA_ANTICAPTCHA_API_KEY=your_api_key_here

# Default service
CAPTCHA_DEFAULT_SERVICE=2captcha
```

### Service Configuration
```rust
let config = CaptchaServiceConfig {
    service: "2captcha".to_string(), // or "anticaptcha"
    api_key: "your_api_key".to_string(),
    timeout_seconds: 120, // Maximum wait time
};
```

## Workflow

1. **Detection**: Automatically detects CAPTCHA type and location
2. **Solving**: Submits to solving service or uses OCR
3. **Polling**: Waits for solution (with timeout)
4. **Injection**: Automatically injects solution into page
5. **Verification**: Solution is ready for form submission

## Prerequisites

1. **Browser with Remote Debugging**: Must have browser running with `--remote-debugging-port`
2. **CAPTCHA Solving Service Account**: 
   - 2Captcha: https://2captcha.com/
   - Anti-Captcha: https://anti-captcha.com/
3. **API Key**: Obtain from service provider
4. **Credits**: Service requires credits/balance

## Security Notes

- All CAPTCHA operations are gated behind `security_gate.check_access()`
- Master Orchestrator has full access by default
- API keys should be stored securely (environment variables, encrypted storage)
- Service-based solving may incur costs
- Some CAPTCHAs may require multiple attempts

## Limitations

- OCR-based solving has lower success rate for complex CAPTCHAs
- Service-based solving requires internet connection and API credits
- Some CAPTCHAs may have rate limiting
- reCAPTCHA v3 requires site-specific configuration
- Complex image CAPTCHAs may need manual intervention

## Error Handling

Common errors and solutions:
- `No CAPTCHA detected`: Page may not have CAPTCHA, or detection failed
- `CAPTCHA service configuration required`: Need to provide API key
- `CAPTCHA solving timeout`: Service took too long, try again
- `Failed to submit CAPTCHA`: Check API key and service status
- `Unsupported CAPTCHA type`: CAPTCHA type not yet supported

## Future Enhancements

- Full OCR implementation with image preprocessing
- Support for more CAPTCHA solving services
- Machine learning-based CAPTCHA solving
- Browser extension integration
- Automatic retry logic
- Solution caching
