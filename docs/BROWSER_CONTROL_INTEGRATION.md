# Browser Control Integration Guide

## Overview

The Master Orchestrator now has **full-control and unlimited access** to web browsers, including:
- Access to existing browser sessions (Chrome, Edge, Firefox)
- Cookie management (read/write cookies)
- Extension management (list/enable/disable)
- Full browser automation via Chrome DevTools Protocol (CDP)
- JavaScript execution in browser tabs
- Tab management

## Integration into Cerebrum Nexus

Add the following command handlers to `cerebrum_nexus/src/lib.rs` in the `handle_system_access_command` function:

```rust
// Enhanced Browser Control Commands
else if let Some(rest) = cmd.strip_prefix("browser") {
    let browser_cmd = rest.trim();
    if browser_cmd.is_empty() || browser_cmd == "help" {
        return Some(
            "Enhanced Browser Control Commands:\n\
            - system browser sessions (list all browser sessions)\n\
            - system browser launch <chrome|edge> [port=9222] (launch with remote debugging)\n\
            - system browser connect <chrome|edge> [port=9222] (connect to existing session)\n\
            - system browser tabs [port=9222] (list all tabs)\n\
            - system browser cookies <chrome|edge> [domain=...] (get cookies)\n\
            - system browser set-cookie <chrome|edge> [port=9222] | name=... | value=... | domain=... | path=...\n\
            - system browser extensions <chrome|edge> (list extensions)\n\
            - system browser js <port> | code=... (execute JavaScript)\n\
            - system browser navigate <port> | url=... (navigate tab)\n"
                .to_string(),
        );
    } else if browser_cmd == "sessions" {
        match self.system_access.find_browser_sessions().await {
            Ok(sessions) => {
                if let Ok(json) = serde_json::to_string_pretty(&sessions) {
                    Some(json)
                } else {
                    let mut output = String::new();
                    output.push_str(&format!("Found {} browser sessions:\n", sessions.len()));
                    for s in sessions {
                        output.push_str(&format!(
                            "- {}: {} (running: {}, debug_port: {:?})\n",
                            s.browser_type, s.profile_path, s.is_running, s.debug_port
                        ));
                    }
                    Some(output)
                }
            }
            Err(e) => Some(format!("Failed to find browser sessions: {}", e)),
        }
    } else if let Some(rest) = browser_cmd.strip_prefix("launch") {
        let parts: Vec<&str> = rest.trim().split_whitespace().collect();
        if parts.is_empty() {
            return Some("system browser launch requires: system browser launch <chrome|edge> [port=9222]".to_string());
        }
        let browser_type = parts[0];
        let mut debug_port = 9222;
        for part in parts.iter().skip(1) {
            if let Some(port_str) = part.strip_prefix("port=") {
                debug_port = port_str.parse().unwrap_or(9222);
            }
        }
        match self.system_access.launch_browser_with_debugging(browser_type, debug_port).await {
            Ok(_) => Some(format!("Launched {} browser with remote debugging on port {}", browser_type, debug_port)),
            Err(e) => Some(format!("Failed to launch browser: {}", e)),
        }
    } else if let Some(rest) = browser_cmd.strip_prefix("connect") {
        let parts: Vec<&str> = rest.trim().split_whitespace().collect();
        if parts.is_empty() {
            return Some("system browser connect requires: system browser connect <chrome|edge> [port=9222]".to_string());
        }
        let browser_type = parts[0];
        let mut debug_port = 9222;
        for part in parts.iter().skip(1) {
            if let Some(port_str) = part.strip_prefix("port=") {
                debug_port = port_str.parse().unwrap_or(9222);
            }
        }
        match self.system_access.connect_browser_session(browser_type, debug_port).await {
            Ok(msg) => Some(msg),
            Err(e) => Some(format!("Failed to connect: {}", e)),
        }
    } else if browser_cmd == "tabs" || browser_cmd.starts_with("tabs") {
        let mut debug_port = 9222;
        if let Some(rest) = browser_cmd.strip_prefix("tabs") {
            for part in rest.trim().split_whitespace() {
                if let Some(port_str) = part.strip_prefix("port=") {
                    debug_port = port_str.parse().unwrap_or(9222);
                }
            }
        }
        match self.system_access.get_browser_tabs(debug_port).await {
            Ok(tabs) => {
                if let Ok(json) = serde_json::to_string_pretty(&tabs) {
                    Some(json)
                } else {
                    let mut output = String::new();
                    output.push_str(&format!("Found {} tabs:\n", tabs.len()));
                    for t in tabs {
                        output.push_str(&format!("- {}: {} ({})\n", t.id, t.title, t.url));
                    }
                    Some(output)
                }
            }
            Err(e) => Some(format!("Failed to get tabs: {}", e)),
        }
    } else if let Some(rest) = browser_cmd.strip_prefix("cookies") {
        let parts: Vec<&str> = rest.trim().split_whitespace().collect();
        if parts.is_empty() {
            return Some("system browser cookies requires: system browser cookies <chrome|edge> [domain=...]".to_string());
        }
        let browser_type = parts[0];
        let mut domain = None;
        for part in parts.iter().skip(1) {
            if let Some(d) = part.strip_prefix("domain=") {
                domain = Some(d);
            }
        }
        match self.system_access.get_browser_cookies(browser_type, domain).await {
            Ok(cookies) => {
                if let Ok(json) = serde_json::to_string_pretty(&cookies) {
                    Some(json)
                } else {
                    let mut output = String::new();
                    output.push_str(&format!("Found {} cookies:\n", cookies.len()));
                    for c in cookies.iter().take(20) {
                        output.push_str(&format!("- {}={} (domain: {}, path: {})\n", c.name, c.value, c.domain, c.path));
                    }
                    Some(output)
                }
            }
            Err(e) => Some(format!("Failed to get cookies: {}", e)),
        }
    } else if let Some(rest) = browser_cmd.strip_prefix("set-cookie") {
        // Parse: system browser set-cookie <chrome|edge> [port=9222] | name=... | value=... | domain=... | path=...
        let mut parts = rest.trim().split('|').map(|s| s.trim());
        let first_part = parts.next().unwrap_or("");
        let browser_and_port: Vec<&str> = first_part.split_whitespace().collect();
        let browser_type = browser_and_port.get(0).unwrap_or(&"chrome");
        let mut debug_port = 9222;
        for part in browser_and_port.iter().skip(1) {
            if let Some(port_str) = part.strip_prefix("port=") {
                debug_port = port_str.parse().unwrap_or(9222);
            }
        }
        
        let mut name = None;
        let mut value = None;
        let mut domain = None;
        let mut path = Some("/".to_string());
        
        for part in parts {
            if let Some(v) = part.strip_prefix("name=") {
                name = Some(v.trim().to_string());
            } else if let Some(v) = part.strip_prefix("value=") {
                value = Some(v.trim().to_string());
            } else if let Some(v) = part.strip_prefix("domain=") {
                domain = Some(v.trim().to_string());
            } else if let Some(v) = part.strip_prefix("path=") {
                path = Some(v.trim().to_string());
            }
        }
        
        let name = name.ok_or_else(|| "name is required".to_string())?;
        let value = value.ok_or_else(|| "value is required".to_string())?;
        let domain = domain.ok_or_else(|| "domain is required".to_string())?;
        let path = path.unwrap_or_else(|| "/".to_string());
        
        let cookie = CookieInfo {
            name,
            value,
            domain,
            path,
            secure: false,
            http_only: false,
            same_site: None,
            expires: None,
        };
        
        match self.system_access.set_browser_cookie(browser_type, debug_port, &cookie).await {
            Ok(_) => Some("Cookie set successfully".to_string()),
            Err(e) => Some(format!("Failed to set cookie: {}", e)),
        }
    } else if let Some(rest) = browser_cmd.strip_prefix("extensions") {
        let browser_type = rest.trim();
        if browser_type.is_empty() {
            return Some("system browser extensions requires: system browser extensions <chrome|edge>".to_string());
        }
        match self.system_access.list_browser_extensions(browser_type).await {
            Ok(extensions) => {
                if let Ok(json) = serde_json::to_string_pretty(&extensions) {
                    Some(json)
                } else {
                    let mut output = String::new();
                    output.push_str(&format!("Found {} extensions:\n", extensions.len()));
                    for e in extensions {
                        output.push_str(&format!("- {}: {} v{} (enabled: {})\n", e.id, e.name, e.version, e.enabled));
                    }
                    Some(output)
                }
            }
            Err(e) => Some(format!("Failed to list extensions: {}", e)),
        }
    } else if let Some(rest) = browser_cmd.strip_prefix("js") {
        // Parse: system browser js <port> | code=...
        let mut parts = rest.trim().split('|').map(|s| s.trim());
        let port_str = parts.next().unwrap_or("9222");
        let debug_port = port_str.parse::<u16>().unwrap_or(9222);
        let mut js_code = None;
        for part in parts {
            if let Some(code) = part.strip_prefix("code=") {
                js_code = Some(code.trim().to_string());
            }
        }
        let js_code = js_code.ok_or_else(|| "code is required".to_string())?;
        match self.system_access.execute_browser_js(debug_port, "", &js_code).await {
            Ok(result) => Some(result),
            Err(e) => Some(format!("Failed to execute JavaScript: {}", e)),
        }
    } else if let Some(rest) = browser_cmd.strip_prefix("navigate") {
        // Parse: system browser navigate <port> | url=...
        let mut parts = rest.trim().split('|').map(|s| s.trim());
        let port_str = parts.next().unwrap_or("9222");
        let debug_port = port_str.parse::<u16>().unwrap_or(9222);
        let mut url = None;
        for part in parts {
            if let Some(u) = part.strip_prefix("url=") {
                url = Some(u.trim().to_string());
            }
        }
        let url = url.ok_or_else(|| "url is required".to_string())?;
        // Use CDP to navigate
        let navigate_js = format!("window.location.href = '{}'", url);
        match self.system_access.execute_browser_js(debug_port, "", &navigate_js).await {
            Ok(_) => Some(format!("Navigated to {}", url)),
            Err(e) => Some(format!("Failed to navigate: {}", e)),
        }
    } else {
        Some("Unknown browser command. Try: system browser help".to_string())
    }
}
```

## Usage Examples

### List Browser Sessions
```
system browser sessions
```

### Launch Chrome with Remote Debugging
```
system browser launch chrome port=9222
```

### Connect to Existing Browser
```
system browser connect chrome port=9222
```

### List All Tabs
```
system browser tabs port=9222
```

### Get Cookies
```
system browser cookies chrome
system browser cookies chrome domain=example.com
```

### Set Cookie
```
system browser set-cookie chrome port=9222 | name=session | value=abc123 | domain=.example.com | path=/
```

### List Extensions
```
system browser extensions chrome
```

### Execute JavaScript
```
system browser js 9222 | code=document.title
```

### Navigate Tab
```
system browser navigate 9222 | url=https://example.com
```

## Prerequisites

1. **Chrome/Edge Remote Debugging**: Browsers must be launched with `--remote-debugging-port` flag
2. **Cookie Access**: Requires access to browser profile directories (Windows: `%LOCALAPPDATA%\Google\Chrome\User Data\Default\Cookies`)
3. **Extension Access**: Requires access to browser extensions directory

## Security Notes

- All browser operations are gated behind `security_gate.check_access()`
- Master Orchestrator has full access by default
- Cookie access requires direct file system access to browser profile
- Remote debugging ports should be protected (localhost only recommended)
