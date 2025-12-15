# EcoSystem Implementation Plan

## Overview

This document outlines the complete implementation plan for the EcoSystem feature, which allows users to import, manage, build, and orchestrate GitHub repositories within Phoenix AGI.

## Architecture

### Backend Components

1. **EcosystemManager** (`ecosystem_manager/src/lib.rs`)
   - ✅ Created: Repo cloning, build detection, service management
   - Handles: Cargo, npm, pip, Make, Docker, Maven, Gradle
   - Process management for running services

2. **API Endpoints** (to be added to `phoenix-web/src/main.rs`)
   - `POST /api/ecosystem/import` - Import a GitHub repo
   - `GET /api/ecosystem/list` - List all repos
   - `GET /api/ecosystem/{id}` - Get repo details
   - `POST /api/ecosystem/{id}/build` - Build a repo
   - `POST /api/ecosystem/{id}/start` - Start service
   - `POST /api/ecosystem/{id}/stop` - Stop service
   - `DELETE /api/ecosystem/{id}` - Remove repo
   - `POST /api/ecosystem/{id}/command` - Execute custom command

3. **Master Orchestrator Integration**
   - Command routing: `ecosystem {repo_id} {command}`
   - Service discovery and registration
   - Dynamic command availability

### Frontend Components

1. **EcoSystemView Component** (`frontend/index.tsx`)
   - Repo import form (owner/repo/branch)
   - Repo list with status cards
   - Build/start/stop controls
   - Service status indicators
   - Command execution interface

2. **Navigation Integration**
   - Add to sidebar: "EcoSystem" menu item
   - Add to activeView type union
   - Route handling

## Implementation Steps

### Phase 1: Backend API (Current)
- [x] Create ecosystem_manager crate
- [x] Implement repo cloning
- [x] Implement build system detection
- [ ] Add API endpoints to phoenix-web
- [ ] Initialize EcosystemManager in main()
- [ ] Add ecosystem command routing

### Phase 2: Frontend UI
- [ ] Create EcoSystemView component
- [ ] Add to sidebar navigation
- [ ] Implement repo import form
- [ ] Implement repo list with cards
- [ ] Add build/start/stop controls
- [ ] Add status indicators

### Phase 3: Master Orchestrator Integration
- [ ] Add ecosystem command routing
- [ ] Service discovery
- [ ] Dynamic command registration

## Data Flow

```
User Action (Frontend)
    ↓
POST /api/ecosystem/import { owner, repo, branch }
    ↓
EcosystemManager::import_repo()
    ├─→ git clone
    ├─→ detect_build_system()
    ├─→ discover_commands()
    └─→ Return RepoMetadata
    ↓
Frontend: Display in repo list
    ↓
User: Click "Build"
    ↓
POST /api/ecosystem/{id}/build
    ↓
EcosystemManager::build_repo()
    └─→ Execute build command
    ↓
User: Click "Start"
    ↓
POST /api/ecosystem/{id}/start
    ↓
EcosystemManager::start_service()
    └─→ Spawn process
    ↓
Service Running
    ↓
Master Orchestrator can route commands
```

## Command Routing

### Format
```
ecosystem {repo_id} {command} [args...]
```

### Examples
- `ecosystem my-service start`
- `ecosystem my-service stop`
- `ecosystem my-service custom-command arg1 arg2`
- `ecosystem my-service build`

### Integration Point
Add to `command_to_response_json()`:
```rust
if lower.starts_with("ecosystem ") {
    // Parse: ecosystem {repo_id} {command} [args...]
    // Route to EcosystemManager::execute_command()
}
```

## Next Steps

1. Complete backend API endpoints
2. Initialize EcosystemManager in main()
3. Add command routing
4. Create frontend component
5. Add to navigation
6. Test end-to-end flow
