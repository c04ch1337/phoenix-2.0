# EcoSystem Design & Implementation Plan

## Overview

The EcoSystem page allows users to import, manage, build, and orchestrate any GitHub repository within the Phoenix AGI framework. This enables dynamic extension of the system's capabilities through external frameworks and systems.

## Architecture

### Backend Components

1. **Ecosystem Manager Module** (`ecosystem_manager`)
   - Repo cloning and management
   - Build system detection and execution
   - Service orchestration
   - Metadata storage

2. **API Endpoints** (`/api/ecosystem/*`)
   - `POST /api/ecosystem/import` - Clone and import a repo
   - `GET /api/ecosystem/list` - List all managed repos
   - `GET /api/ecosystem/{id}/status` - Get repo status
   - `POST /api/ecosystem/{id}/build` - Build a repo
   - `POST /api/ecosystem/{id}/start` - Start a service
   - `POST /api/ecosystem/{id}/stop` - Stop a service
   - `DELETE /api/ecosystem/{id}` - Remove a repo
   - `POST /api/ecosystem/{id}/orchestrate` - Register with Master Orchestrator

### Frontend Components

1. **EcoSystemView Component**
   - Repo import form
   - Repo list with status cards
   - Build controls
   - Service management
   - Orchestration status

2. **Integration Points**
   - Sidebar navigation
   - Master Orchestrator command routing
   - Service discovery

## Data Model

```rust
pub struct RepoMetadata {
    pub id: String,                    // Unique identifier
    pub name: String,                   // Repo name
    pub owner: String,                  // GitHub owner
    pub url: String,                    // GitHub URL
    pub local_path: PathBuf,            // Local clone path
    pub build_system: BuildSystem,      // Detected build system
    pub build_status: BuildStatus,      // Current build status
    pub service_status: ServiceStatus,  // Service running status
    pub port: Option<u16>,              // Service port (if applicable)
    pub commands: Vec<String>,          // Available commands
    pub created_at: i64,                // Unix timestamp
    pub last_built: Option<i64>,        // Last build timestamp
}

pub enum BuildSystem {
    Cargo,      // Rust/Cargo
    Npm,        // Node.js/npm
    Pip,        // Python/pip
    Make,       // Makefile
    Docker,     // Docker
    Custom(String), // Custom build command
    Unknown,
}

pub enum BuildStatus {
    NotBuilt,
    Building,
    Built,
    BuildFailed(String),
}

pub enum ServiceStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error(String),
}
```

## Implementation Steps

### Phase 1: Backend Foundation
1. Create `ecosystem_manager` crate
2. Implement repo cloning (git)
3. Implement build system detection
4. Implement build execution
5. Create API endpoints

### Phase 2: Service Orchestration
1. Service discovery
2. Process management
3. Port management
4. Health checking

### Phase 3: Master Orchestrator Integration
1. Command routing for ecosystem services
2. Service registry
3. Dynamic command discovery

### Phase 4: Frontend
1. Create EcoSystemView component
2. Add to sidebar navigation
3. Implement UI components
4. Connect to backend APIs

## Build System Detection

Detect build system by checking for:
- `Cargo.toml` → Cargo
- `package.json` → npm/yarn
- `requirements.txt` or `setup.py` → pip
- `Makefile` → Make
- `Dockerfile` → Docker
- `pom.xml` → Maven (Java)
- `build.gradle` → Gradle (Java/Kotlin)

## Service Discovery

For each repo, detect services by:
- Checking for `main.rs` (Rust binary)
- Checking `package.json` scripts
- Checking for common service patterns
- Reading configuration files

## Master Orchestrator Integration

Commands from ecosystem services are routed through:
```
User Command → Master Orchestrator → Ecosystem Service
```

Format: `ecosystem {repo_id} {command}`

Example: `ecosystem my-service start` or `ecosystem my-service custom-command`
