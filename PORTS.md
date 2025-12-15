# Phoenix AGI (PAGI) Port Map

This document provides a comprehensive map of all ports used across Phoenix AGI (PAGI) services.

## Port Configuration

All ports can be overridden via environment variables. Defaults are listed below.

## Service Ports

| Service Name | Crate Path | Protocol | Bind Address | Default Port | Env Var Override | Notes |
|-------------|------------|----------|--------------|--------------|------------------|-------|
| **Phoenix Web UI** | `phoenix-web` | HTTP | `127.0.0.1` | `8888` | `PHOENIX_WEB_BIND` | Main web dashboard and API server |
| **Vital Pulse Collector** | `vital_pulse_collector` | HTTP | `127.0.0.1` | `5002` | `TELEMETRIST_BIND` | Telemetry ingestion service |
| **Synaptic Pulse Distributor** | `synaptic_pulse_distributor` | WebSocket | `127.0.0.1` | `5003` | `PULSE_DISTRIBUTOR_BIND` | Config update distribution service |
| **Frontend Dev Server** | `frontend` (Vite) | HTTP | `0.0.0.0` | `3000` | `VITE_PORT` | Development frontend server |
| **Chrome DevTools** | `browser_orch_ext` | WebSocket | `127.0.0.1` | `9222` | `CHROME_DEBUG_PORT` | Chrome DevTools Protocol (CDP) |
| **Selenium WebDriver** | `digital_twin` | HTTP | `localhost` | `4444` | `SELENIUM_HUB_URL` | Selenium Grid hub (external service) |

## Port Details

### Phoenix Web UI (Port 8888)
- **Purpose**: Main web dashboard and REST API
- **Protocol**: HTTP/HTTPS
- **Endpoints**: `/api/*`, `/health`, static file serving
- **CORS**: Allows `http://localhost:3000` and `http://127.0.0.1:3000`
- **Configuration**: Uses `common_types::ports::PhoenixWebPort::bind()` (env var: `PHOENIX_WEB_BIND`)
- **Status**: ✅ **CONFIGURABLE**

### Vital Pulse Collector (Port 5002)
- **Purpose**: Telemetry ingestion and analysis
- **Protocol**: HTTP
- **Endpoints**: `/health`, `/ingest`, `/analyze`, `/insights`
- **Configuration**: `TELEMETRIST_BIND` env var (default: `127.0.0.1:5002`)
- **Status**: ✅ **CONFIGURABLE**

### Synaptic Pulse Distributor (Port 5003)
- **Purpose**: WebSocket-based config update distribution
- **Protocol**: WebSocket (HTTP upgrade)
- **Endpoints**: `/health`, `/publish`, `/subscribe`
- **Configuration**: `PULSE_DISTRIBUTOR_BIND` env var (default: `127.0.0.1:5003`)
- **Status**: ✅ **CONFIGURABLE**

### Frontend Dev Server (Port 3000)
- **Purpose**: Vite development server for React frontend
- **Protocol**: HTTP
- **Configuration**: `VITE_PORT` env var (default: `3000`)
- **Status**: ✅ **CONFIGURABLE**

### Chrome DevTools Protocol (Port 9222)
- **Purpose**: Browser automation via Chrome DevTools Protocol
- **Protocol**: WebSocket (CDP)
- **Configuration**: Uses `CHROME_DEBUG_PORT` env var (default: `9222`)
- **Status**: ✅ **CONFIGURABLE**

### Selenium WebDriver (Port 4444)
- **Purpose**: Browser automation via Selenium Grid
- **Protocol**: HTTP
- **Configuration**: Uses `SELENIUM_HUB_URL` env var (default: `http://localhost:4444/wd/hub`)
- **Status**: ✅ **CONFIGURABLE**
- **Note**: External service, not part of Phoenix codebase

## Port Conflicts

**No conflicts detected** - all ports are unique:
- 3000: Frontend dev server
- 4444: Selenium (external)
- 5002: Vital Pulse Collector
- 5003: Synaptic Pulse Distributor
- 8888: Phoenix Web UI
- 9222: Chrome DevTools

## gRPC Services

**No gRPC services found** - Phoenix AGI (PAGI) uses:
- HTTP REST APIs (Actix-web)
- WebSocket connections (Actix-web)
- No gRPC/tonic/prost implementations detected

## Configuration Recommendations

### Current Status
1. ✅ `phoenix-web` port - Now uses `PHOENIX_WEB_BIND` env var
2. ✅ Chrome DevTools port (9222) - Now uses `CHROME_DEBUG_PORT` env var
3. ✅ Selenium URL - Now uses `SELENIUM_HUB_URL` env var

### Recommended Environment Variables

Add to `.env.example`:
```bash
# Phoenix Web UI
PHOENIX_WEB_BIND=127.0.0.1:8888

# Telemetry Services
TELEMETRIST_BIND=127.0.0.1:5002
PULSE_DISTRIBUTOR_BIND=127.0.0.1:5003

# Browser Automation
CHROME_DEBUG_PORT=9222
SELENIUM_HUB_URL=http://localhost:4444/wd/hub

# Frontend Dev Server
VITE_PORT=3000
```

## Service Dependencies

```
phoenix-web (8888)
  └─> Frontend dev server (3000) [dev mode only]
  └─> Vital Pulse Collector (5002) [optional]
  └─> Synaptic Pulse Distributor (5003) [optional]

browser_orch_ext
  └─> Chrome DevTools (9222) [required]

digital_twin
  └─> Selenium WebDriver (4444) [required, external]
```

## Verification Commands

```bash
# Check if ports are in use
netstat -an | findstr "3000 4444 5002 5003 8888 9222"

# Test Phoenix Web UI
curl http://127.0.0.1:8888/health

# Test Vital Pulse Collector
curl http://127.0.0.1:5002/health

# Test Synaptic Pulse Distributor
curl http://127.0.0.1:5003/health
```
