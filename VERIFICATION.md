# Phoenix AGI (PAGI) Backend Audit - Verification Guide

## Verification Checklist

### 1. Compilation ✅ (Mostly Complete)

```bash
# Check all packages
cargo check --workspace

# Expected: Most packages compile successfully
# Known issues: system_access may have minor warnings (non-blocking)
```

**Status**: ✅ Port-related packages compile successfully
- ✅ `phoenix-web` - Uses unified port config
- ✅ `vital_pulse_collector` - Uses unified port config  
- ✅ `synaptic_pulse_distributor` - Uses unified port config
- ✅ `browser_orch_ext` - Fixed compilation errors
- ✅ `common_types` - Port module compiles and tests pass

### 2. Port Configuration Tests ✅

```bash
# Test port configuration module
cargo test --package common_types --lib ports

# Expected output:
# test ports::tests::test_default_ports ... ok
# test ports::tests::test_port_validation ... ok
```

**Status**: ✅ **PASSING** - All port tests pass

### 3. Service Startup Tests ⏳ (Manual)

#### Test Phoenix Web UI
```bash
# Default port
cargo run --bin phoenix-web
# Should log: "Phoenix UI server online at http://127.0.0.1:8888"

# Custom port
PHOENIX_WEB_BIND=127.0.0.1:9999 cargo run --bin phoenix-web
# Should log: "Phoenix UI server online at http://127.0.0.1:9999"
```

#### Test Vital Pulse Collector
```bash
# Default port
cargo run --bin vital_pulse_collector
# Should log: "Vital Pulse Collector online at http://127.0.0.1:5002"

# Custom port
TELEMETRIST_BIND=127.0.0.1:6002 cargo run --bin vital_pulse_collector
# Should log: "Vital Pulse Collector online at http://127.0.0.1:6002"
```

#### Test Synaptic Pulse Distributor
```bash
# Default port
cargo run --bin synaptic_pulse_distributor
# Should log: "Synaptic Pulse Distributor online at ws://127.0.0.1:5003/subscribe"

# Custom port
PULSE_DISTRIBUTOR_BIND=127.0.0.1:6003 cargo run --bin synaptic_pulse_distributor
# Should log: "Synaptic Pulse Distributor online at ws://127.0.0.1:6003/subscribe"
```

### 4. Port Conflict Detection ✅

```bash
# Windows
netstat -an | findstr "3000 4444 5002 5003 8888 9222"

# Linux/Mac
netstat -an | grep -E "3000|4444|5002|5003|8888|9222"

# Expected: No duplicate ports in use
```

**Status**: ✅ **NO CONFLICTS** - All ports are unique

### 5. Health Endpoint Tests ⏳ (Manual)

```bash
# Test Phoenix Web UI health
curl http://127.0.0.1:8888/health
# Expected: {"status":"ok"}

# Test Vital Pulse Collector health
curl http://127.0.0.1:5002/health
# Expected: {"status":"ok"}

# Test Synaptic Pulse Distributor health
curl http://127.0.0.1:5003/health
# Expected: {"status":"ok"}
```

### 6. Environment Variable Override Tests ⏳ (Manual)

```bash
# Test port override
export PHOENIX_WEB_BIND=127.0.0.1:9999
cargo run --bin phoenix-web
# Verify service binds to 9999, not 8888

# Test Chrome debug port override
export CHROME_DEBUG_PORT=9333
# Run browser automation code
# Verify it connects to port 9333

# Test Selenium URL override
export SELENIUM_HUB_URL=http://localhost:5555/wd/hub
# Run digital_twin code
# Verify it connects to port 5555
```

### 7. Port Validation Function Test ✅

```rust
// In any service's main() or test
use common_types::ports::validate_ports;

#[tokio::main]
async fn main() -> Result<()> {
    // Validate ports at startup
    validate_ports()?;
    // ... rest of service initialization
}
```

**Status**: ✅ **AVAILABLE** - `validate_ports()` function exists and works

## Summary

### ✅ Completed
1. Port audit complete - all ports identified
2. Port normalization implemented - unified config module
3. Port documentation created - `PORTS.md`
4. Port tests passing - `common_types::ports` module
5. Environment variable support added to all services
6. `.env.example` updated with port configurations

### ⏳ Pending Manual Verification
1. Service startup with default ports
2. Service startup with overridden ports
3. Health endpoint accessibility
4. Port conflict detection in runtime
5. Cross-service communication (if applicable)

### ⚠️ Known Issues
1. `system_access` may have minor compilation warnings (non-blocking)
2. `browser_orch_ext` has style warnings (non-blocking)

## Next Steps

1. **Run manual verification** - Test each service startup
2. **Deploy to staging** - Test in production-like environment
3. **Monitor port usage** - Ensure no conflicts in deployment
4. **Document runtime behavior** - Add startup logs showing actual bind addresses
