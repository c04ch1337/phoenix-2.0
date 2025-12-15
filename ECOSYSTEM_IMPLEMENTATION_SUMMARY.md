# EcoSystem Implementation Summary

## ✅ Completed

### Backend Foundation
1. **Created `ecosystem_manager` crate**
   - Location: `ecosystem_manager/src/lib.rs`
   - Features:
     - Repo cloning via git
     - Build system detection (Cargo, npm, pip, Make, Docker, Maven, Gradle)
     - Build execution
     - Service management (start/stop)
     - Command discovery
     - Process management

2. **Added to workspace**
   - Updated `Cargo.toml` workspace members
   - Added dependency to `phoenix-web/Cargo.toml`
   - Added to `AppState` struct

## 🔄 Next Steps (Implementation Required)

### Backend API Endpoints

Add to `phoenix-web/src/main.rs`:

1. **Import Repo Endpoint**
```rust
#[derive(Debug, Deserialize)]
struct ImportRepoRequest {
    owner: String,
    repo: String,
    branch: Option<String>,
}

async fn api_ecosystem_import(
    state: web::Data<AppState>,
    body: web::Json<ImportRepoRequest>,
) -> impl Responder {
    match state.ecosystem.import_repo(&body.owner, &body.repo, body.branch.as_deref()).await {
        Ok(metadata) => HttpResponse::Ok().json(metadata),
        Err(e) => HttpResponse::BadRequest().json(json!({"error": e.to_string()})),
    }
}
```

2. **List Repos Endpoint**
```rust
async fn api_ecosystem_list(state: web::Data<AppState>) -> impl Responder {
    let repos = state.ecosystem.list_repos().await;
    HttpResponse::Ok().json(repos)
}
```

3. **Build Repo Endpoint**
```rust
async fn api_ecosystem_build(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    match state.ecosystem.build_repo(&path.into_inner()).await {
        Ok(output) => HttpResponse::Ok().json(json!({"status": "success", "output": output})),
        Err(e) => HttpResponse::BadRequest().json(json!({"error": e.to_string()})),
    }
}
```

4. **Start/Stop Service Endpoints**
```rust
async fn api_ecosystem_start(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    match state.ecosystem.start_service(&path.into_inner(), None).await {
        Ok(msg) => HttpResponse::Ok().json(json!({"status": "started", "message": msg})),
        Err(e) => HttpResponse::BadRequest().json(json!({"error": e.to_string()})),
    }
}

async fn api_ecosystem_stop(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    match state.ecosystem.stop_service(&path.into_inner()).await {
        Ok(msg) => HttpResponse::Ok().json(json!({"status": "stopped", "message": msg})),
        Err(e) => HttpResponse::BadRequest().json(json!({"error": e.to_string()})),
    }
}
```

5. **Remove Repo Endpoint**
```rust
async fn api_ecosystem_remove(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    match state.ecosystem.remove_repo(&path.into_inner()).await {
        Ok(_) => HttpResponse::Ok().json(json!({"status": "removed"})),
        Err(e) => HttpResponse::BadRequest().json(json!({"error": e.to_string()})),
    }
}
```

### Initialize EcosystemManager in main()

Add after other initializations:
```rust
let ecosystem = Arc::new(
    EcosystemManager::new("./ecosystem_repos")
        .expect("Failed to initialize EcosystemManager")
);
```

Add to AppState initialization:
```rust
let state = AppState {
    // ... existing fields ...
    ecosystem,
    // ...
};
```

### Register API Routes

Add to route registration:
```rust
.service(
    web::scope("/ecosystem")
        .service(web::resource("/import").route(web::post().to(api_ecosystem_import)))
        .service(web::resource("/list").route(web::get().to(api_ecosystem_list)))
        .service(web::resource("/{id}/build").route(web::post().to(api_ecosystem_build)))
        .service(web::resource("/{id}/start").route(web::post().to(api_ecosystem_start)))
        .service(web::resource("/{id}/stop").route(web::post().to(api_ecosystem_stop)))
        .service(web::resource("/{id}").route(web::delete().to(api_ecosystem_remove))),
)
```

### Command Routing Integration

Add to `command_to_response_json()`:
```rust
// Ecosystem commands
if lower.starts_with("ecosystem ") {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.len() < 3 {
        return json!({
            "type": "error",
            "message": "Usage: ecosystem {repo_id} {command} [args...]"
        });
    }
    
    let repo_id = parts[1];
    let command = parts[2];
    let args: Vec<String> = parts[3..].iter().map(|s| s.to_string()).collect();
    
    return match state.ecosystem.execute_command(repo_id, command, args).await {
        Ok(output) => json!({"type": "ecosystem.result", "message": output}),
        Err(e) => json!({"type": "error", "message": e.to_string()}),
    };
}
```

## Frontend Implementation

### 1. Create EcoSystemView Component

Location: `frontend/index.tsx`

Key features:
- Repo import form (owner/repo/branch)
- Repo list with status cards
- Build/start/stop buttons
- Service status indicators
- Command execution interface

### 2. Add to Navigation

Update `activeView` type:
```typescript
const [activeView, setActiveView] = useState<'chat' | 'archetype' | 'settings' | 'memories' | 'orchestrator' | 'studio' | 'google' | 'devtools' | 'ecosystem'>('chat');
```

Add sidebar item:
```typescript
<SidebarItem icon={GitBranch} label="EcoSystem" active={activeView === 'ecosystem'} onClick={() => handleNavigation('ecosystem')} />
```

Add route:
```typescript
{activeView === 'ecosystem' && <EcoSystemView />}
```

## Testing Checklist

- [ ] Backend compiles
- [ ] EcosystemManager initializes
- [ ] Can clone a GitHub repo
- [ ] Build system detection works
- [ ] Can build a repo
- [ ] Can start/stop services
- [ ] API endpoints respond correctly
- [ ] Frontend displays repo list
- [ ] Can import repo from UI
- [ ] Can build from UI
- [ ] Can start/stop from UI
- [ ] Master Orchestrator routes ecosystem commands

## File Structure

```
ecosystem_manager/
├── Cargo.toml
└── src/
    └── lib.rs

phoenix-web/
├── Cargo.toml (updated)
└── src/
    └── main.rs (needs API endpoints + initialization)

frontend/
└── index.tsx (needs EcoSystemView component)
```
