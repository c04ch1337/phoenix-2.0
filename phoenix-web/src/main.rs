// phoenix-web/src/main.rs
//
// Minimal HTTP API + static asset server for the Vite UI in `frontend/`.
//
// Goals:
// - Provide a stable command router: UI -> send(command) -> response
// - Provide health/status/name endpoints for UI bootstrapping
// - Optionally serve `frontend/dist` when built

use actix_cors::Cors;
use actix_files::NamedFile;
use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder, ResponseError};
use actix_web::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    path::{Component, Path, PathBuf},
    sync::Arc,
};
use tokio::sync::Mutex;
use tracing::{info, warn};

use llm_orchestrator::LLMOrchestrator;
use evolution_pipeline::GitHubEnforcer;
use phoenix_identity::PhoenixIdentityManager;
use relationship_dynamics::{Partnership, RelationshipTemplate};
use system_access::{CommandResult, SystemAccessManager};
use vital_organ_vaults::VitalOrganVaults;
use context_engine::{ContextEngine, ContextRequest, ContextMemory, ContextLayer};
use neural_cortex_strata::{NeuralCortexStrata, MemoryLayer};
use std::time::{SystemTime, UNIX_EPOCH};
use ecosystem_manager::EcosystemManager;

mod google;
use google::{GoogleInitError, GoogleManager};

#[derive(Clone)]
struct AppState {
    vaults: Arc<VitalOrganVaults>,
    neural_cortex: Arc<NeuralCortexStrata>,
    context_engine: Arc<ContextEngine>,
    phoenix_identity: Arc<PhoenixIdentityManager>,
    relationship: Arc<Mutex<Partnership>>,
    vector_kb: Option<Arc<vector_kb::VectorKB>>,
    llm: Option<Arc<LLMOrchestrator>>,
    system: Arc<SystemAccessManager>,
    google: Option<GoogleManager>,
    ecosystem: Arc<EcosystemManager>,
    version: String,
}

#[derive(Debug, Deserialize)]
struct CommandRequest {
    command: String,
}

#[derive(Debug, Deserialize)]
struct ImportRepoRequest {
    owner: String,
    repo: String,
    #[serde(default)]
    branch: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SpeakRequest {
    user_input: String,
    #[serde(default)]
    dad_emotion_hint: Option<String>,
    #[serde(default)]
    mode: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExecRequest {
    command: String,
    #[serde(default)]
    cwd: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReadFileRequest {
    path: String,
}

#[derive(Debug, Deserialize)]
struct WriteFileRequest {
    path: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct MemoryStoreRequest {
    key: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct MemorySearchQuery {
    #[serde(default)]
    q: String,
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct VectorMemoryStoreRequest {
    text: String,
    #[serde(default)]
    metadata: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct VectorMemorySearchQuery {
    #[serde(default)]
    q: String,
    #[serde(default)]
    k: Option<usize>,
}

#[derive(Debug, Serialize)]
struct VectorMemoryStoreResponse {
    status: &'static str,
    id: String,
}

#[derive(Debug, Serialize)]
struct VectorMemorySearchResponse {
    results: Vec<vector_kb::MemoryResult>,
    count: usize,
}

#[derive(Debug, Serialize)]
struct VectorMemoryEntrySummary {
    id: String,
    text: String,
    metadata: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct VectorMemoryAllResponse {
    entries: Vec<VectorMemoryEntrySummary>,
    count: usize,
}

#[derive(Debug, Serialize)]
struct MemoryItem {
    key: String,
    value: String,
}

#[derive(Debug, Serialize)]
struct MemorySearchResponse {
    items: Vec<MemoryItem>,
    count: usize,
}

#[derive(Debug, Serialize)]
struct StatusOkResponse {
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    #[serde(rename = "type")]
    kind: &'static str,
    message: String,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ApiError {}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        self.status
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status).json(ErrorResponse {
            kind: "error",
            message: self.message.clone(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct GoogleOAuthCallbackQuery {
    code: String,
    state: String,
    #[allow(dead_code)]
    #[serde(default)]
    scope: Option<String>,
}

#[derive(Debug, Serialize)]
struct StatusResponse {
    status: String,
    llm_status: String,
    version: String,
    archetype: String,
}

static FRONTEND_COMMAND_REGISTRY_JSON: &str =
    include_str!("../../docs/frontend_command_registry.json");

async fn health() -> impl Responder {
    HttpResponse::Ok().json(json!({"status": "ok"}))
}

async fn favicon_ico() -> impl Responder {
    // Many browsers request `/favicon.ico` even when an SVG favicon is provided.
    // Redirect to the bundled SVG in `frontend/public/favicon.svg` (copied into `frontend/dist/`).
    HttpResponse::Found()
        .append_header(("Location", "/favicon.svg"))
        .finish()
}

async fn api_name(state: web::Data<AppState>) -> impl Responder {
    let identity = state.phoenix_identity.get_identity().await;
    HttpResponse::Ok().json(json!({"name": identity.display_name()}))
}

async fn api_status(state: web::Data<AppState>) -> impl Responder {
    let archetype = format!("{:?}", state.phoenix_identity.zodiac_sign());
    let out = StatusResponse {
        // The UI uses this as a connectivity gate. If this server is answering,
        // the UI should be allowed to operate (even if the LLM is disabled).
        status: "online".to_string(),
        llm_status: if state.llm.is_some() { "online" } else { "offline" }.to_string(),
        version: state.version.clone(),
        archetype,
    };
    HttpResponse::Ok().json(out)
}

async fn api_command_registry() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/json")
        .body(FRONTEND_COMMAND_REGISTRY_JSON)
}

async fn api_system_status(state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(json!({
        "full_access_granted": state.system.is_access_granted().await,
        "self_modification_enabled": state.system.is_self_modification_enabled().await,
    }))
}

async fn api_evolution_status() -> impl Responder {
    // Exposes sanitized config only (no token values).
    HttpResponse::Ok().json(GitHubEnforcer::env_status())
}

async fn api_system_exec(state: web::Data<AppState>, body: web::Json<ExecRequest>) -> impl Responder {
    match state
        .system
        .exec_shell(&body.command, body.cwd.as_deref())
        .await
    {
        Ok(CommandResult {
            exit_code,
            stdout,
            stderr,
        }) => HttpResponse::Ok().json(json!({
            "exit_code": exit_code,
            "stdout": stdout,
            "stderr": stderr,
        })),
        Err(e) => HttpResponse::BadRequest().json(json!({"type": "error", "message": e})),
    }
}

async fn api_system_read_file(
    state: web::Data<AppState>,
    body: web::Json<ReadFileRequest>,
) -> impl Responder {
    match state.system.read_file(&body.path).await {
        Ok(content) => HttpResponse::Ok().json(json!({"path": body.path, "content": content})),
        Err(e) => HttpResponse::BadRequest().json(json!({"type": "error", "message": e})),
    }
}

async fn api_system_write_file(
    state: web::Data<AppState>,
    body: web::Json<WriteFileRequest>,
) -> impl Responder {
    match state.system.write_file(&body.path, &body.content).await {
        Ok(()) => HttpResponse::Ok().json(json!({"status": "ok"})),
        Err(e) => HttpResponse::BadRequest().json(json!({"type": "error", "message": e})),
    }
}

async fn api_not_found(req: HttpRequest) -> impl Responder {
    HttpResponse::NotFound().json(json!({
        "type": "error",
        "message": format!("Unknown API route: {}", req.path())
    }))
}

const MEMORY_SEARCH_LIMIT_DEFAULT: usize = 20;
const MEMORY_SEARCH_LIMIT_MAX: usize = 100;

const VECTOR_SEARCH_K_DEFAULT: usize = 5;
const VECTOR_SEARCH_K_MAX: usize = 50;

async fn api_memory_store(
    state: web::Data<AppState>,
    body: web::Json<MemoryStoreRequest>,
) -> Result<HttpResponse, ApiError> {
    let key = body.key.trim();
    if key.is_empty() {
        return Err(ApiError::bad_request("Empty key."));
    }

    state
        .vaults
        .store_soul(key, &body.value)
        .map_err(|e| ApiError::internal(format!("Failed to store memory: {e}")))?;

    Ok(HttpResponse::Ok().json(StatusOkResponse { status: "ok" }))
}

async fn api_memory_get(
    state: web::Data<AppState>,
    key: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let key = key.into_inner();
    let key = key.trim();
    if key.is_empty() {
        return Err(ApiError::bad_request("Empty key."));
    }

    let Some(value) = state.vaults.recall_soul(key) else {
        return Err(ApiError::not_found("Key not found."));
    };

    Ok(HttpResponse::Ok().json(MemoryItem {
        key: key.to_string(),
        value,
    }))
}

async fn api_memory_search(
    state: web::Data<AppState>,
    q: web::Query<MemorySearchQuery>,
) -> Result<HttpResponse, ApiError> {
    let limit = q
        .limit
        .unwrap_or(MEMORY_SEARCH_LIMIT_DEFAULT)
        .min(MEMORY_SEARCH_LIMIT_MAX);

    let prefix = format!("soul:{}", q.q.trim());
    let items = state
        .vaults
        .recall_prefix(&prefix, limit)
        .into_iter()
        .map(|(key, value)| MemoryItem { key, value })
        .collect::<Vec<_>>();

    let count = items.len();
    Ok(HttpResponse::Ok().json(MemorySearchResponse { items, count }))
}

async fn api_memory_delete(
    state: web::Data<AppState>,
    key: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let key = key.into_inner();
    let key = key.trim();
    if key.is_empty() {
        return Err(ApiError::bad_request("Empty key."));
    }

    let existed = state
        .vaults
        .forget_soul(key)
        .map_err(|e| ApiError::internal(format!("Failed to delete memory: {e}")))?;

    if !existed {
        return Err(ApiError::not_found("Key not found."));
    }

    Ok(HttpResponse::Ok().json(StatusOkResponse { status: "ok" }))
}

async fn api_memory_vector_store(
    state: web::Data<AppState>,
    body: web::Json<VectorMemoryStoreRequest>,
) -> Result<HttpResponse, ApiError> {
    let Some(kb) = state.vector_kb.as_ref() else {
        return Err(ApiError::bad_request(
            "Vector KB is disabled. Set VECTOR_KB_ENABLED=true.",
        ));
    };

    let entry = kb
        .add_memory(&body.text, body.metadata.clone())
        .await
        .map_err(|e| ApiError::internal(format!("Vector store failed: {e}")))?;

    Ok(HttpResponse::Ok().json(VectorMemoryStoreResponse {
        status: "ok",
        id: entry.id,
    }))
}

async fn api_memory_vector_search(
    state: web::Data<AppState>,
    q: web::Query<VectorMemorySearchQuery>,
) -> Result<HttpResponse, ApiError> {
    let Some(kb) = state.vector_kb.as_ref() else {
        return Err(ApiError::bad_request(
            "Vector KB is disabled. Set VECTOR_KB_ENABLED=true.",
        ));
    };

    let k = q
        .k
        .unwrap_or(VECTOR_SEARCH_K_DEFAULT)
        .max(1)
        .min(VECTOR_SEARCH_K_MAX);

    let results = kb
        .semantic_search(&q.q, k)
        .await
        .map_err(|e| ApiError::internal(format!("Vector search failed: {e}")))?;
    let count = results.len();
    Ok(HttpResponse::Ok().json(VectorMemorySearchResponse { results, count }))
}

async fn api_memory_vector_all(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    let Some(kb) = state.vector_kb.as_ref() else {
        return Err(ApiError::bad_request(
            "Vector KB is disabled. Set VECTOR_KB_ENABLED=true.",
        ));
    };

    let entries = kb
        .all()
        .await
        .map_err(|e| ApiError::internal(format!("Vector list failed: {e}")))?
        .into_iter()
        .map(|e| VectorMemoryEntrySummary {
            id: e.id,
            text: e.text,
            metadata: e.metadata,
        })
        .collect::<Vec<_>>();
    let count = entries.len();
    Ok(HttpResponse::Ok().json(VectorMemoryAllResponse { entries, count }))
}

async fn api_google_auth_start(state: web::Data<AppState>) -> impl Responder {
    match state.google.as_ref() {
        Some(g) => HttpResponse::Ok().json(g.auth_start().await),
        None => HttpResponse::BadRequest().json(json!({
            "type": "error",
            "message": "Google integration not configured. Set GOOGLE_OAUTH_CLIENT_ID / GOOGLE_OAUTH_CLIENT_SECRET / GOOGLE_OAUTH_REDIRECT_URL."
        })),
    }
}

async fn api_google_oauth2_callback(
    state: web::Data<AppState>,
    q: web::Query<GoogleOAuthCallbackQuery>,
) -> impl Responder {
    let Some(g) = state.google.as_ref() else {
        return HttpResponse::BadRequest().content_type("text/html").body(
            "<h2>Phoenix Google OAuth</h2><p>Google integration is not configured on the server.</p>",
        );
    };

    match g.auth_callback(&q.code, &q.state).await {
        Ok(()) => HttpResponse::Ok().content_type("text/html").body(
            "<h2>Phoenix Google OAuth</h2><p>Connected. You may close this window and return to Phoenix.</p>",
        ),
        Err(e) => HttpResponse::BadRequest()
            .content_type("text/html")
            .body(format!(
                "<h2>Phoenix Google OAuth</h2><p>Connection failed: {}</p><p>Return to Phoenix and retry <code>google auth start</code>.</p>",
                html_escape::encode_text(&e)
            )),
    }
}

fn safe_join(dist_dir: &Path, rel: &str) -> Option<PathBuf> {
    let rel_path = Path::new(rel);
    if rel_path.is_absolute() {
        return None;
    }
    if !rel_path
        .components()
        .all(|c| matches!(c, Component::Normal(_)))
    {
        return None;
    }
    Some(dist_dir.join(rel_path))
}

async fn ui_serve(req: HttpRequest, dist_dir: web::Data<PathBuf>) -> actix_web::Result<NamedFile> {
    // `path` comes from `/{path:.*}` and is never prefixed with '/'.
    let path = req.match_info().query("path");
    let requested = if path.is_empty() { "index.html" } else { path };

    let dist_dir = dist_dir.get_ref();
    let index_path = dist_dir.join("index.html");

    if let Some(candidate) = safe_join(dist_dir, requested) {
        if candidate.is_file() {
            return Ok(NamedFile::open(candidate)?);
        }
    }

    // SPA fallback: serve index.html for any unknown UI route.
    Ok(NamedFile::open(index_path)?)
}

fn normalize_command(s: &str) -> String {
    s.trim().replace("\r\n", "\n")
}

/// Retrieve memories from all vaults and build EQ-first context.
async fn build_memory_context(
    state: &AppState,
    user_input: &str,
    emotion_hint: Option<&str>,
) -> String {
    // 1. Retrieve relational memories from Soul Vault
    let relational_memory = state
        .vaults
        .recall_soul("dad:last_soft_memory")
        .or_else(|| state.vaults.recall_soul("dad:last_emotion"));

    // 2. Retrieve episodic memories from Neural Cortex Strata (last 8 with epm:dad: prefix)
    let episodic_memories = state
        .neural_cortex
        .recall_prefix("epm:dad:", 8);
    
    // Convert episodic memories to ContextMemory format
    let mut episodic_context = Vec::new();
    let now_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    
    for (key, layer) in episodic_memories {
        if let MemoryLayer::EPM(text) = layer {
            // Extract timestamp from key if present (epm:dad:1234567890)
            let ts_unix = key
                .split(':')
                .last()
                .and_then(|s| s.parse::<i64>().ok());
            
            episodic_context.push(ContextMemory {
                layer: ContextLayer::Episodic,
                text,
                ts_unix,
                intensity: 1.0,
            });
        }
    }

    // 3. Retrieve relevant knowledge from Mind/Body vaults if user input suggests factual queries
    // Simple heuristic: if input contains question words or seems like a knowledge query
    let lower_input = user_input.to_lowercase();
    let is_knowledge_query = lower_input.contains("what") 
        || lower_input.contains("who") 
        || lower_input.contains("when") 
        || lower_input.contains("where") 
        || lower_input.contains("how")
        || lower_input.contains("why")
        || lower_input.contains("remember")
        || lower_input.contains("know");
    
    let mut knowledge_snippets = Vec::new();
    if is_knowledge_query {
        // Extract key terms from input for knowledge base search
        let key_terms: Vec<&str> = lower_input
            .split_whitespace()
            .filter(|w| w.len() > 3 && !["what", "who", "when", "where", "how", "why", "the", "and", "for", "are", "but", "not", "you", "all", "can", "her", "was", "one", "our", "out", "day", "get", "has", "him", "his", "how", "man", "new", "now", "old", "see", "two", "way", "who", "boy", "did", "its", "let", "put", "say", "she", "too", "use"].contains(w))
            .take(3)
            .collect();
        
        // Search Mind vault for relevant knowledge
        for term in key_terms {
            let mind_results = state.vaults.recall_prefix(&format!("mind:{}", term), 2);
            for (_, value) in mind_results {
                if !value.trim().is_empty() {
                    knowledge_snippets.push(format!("Knowledge: {}", value));
                }
            }
        }
    }

    // 3.5 Semantic vector recall (Phase 2) — only if enabled.
    if let Some(kb) = state.vector_kb.as_ref() {
        // Prefer an explicit emotion hint because it yields better recall prompts.
        let recall_query = if let Some(e) = emotion_hint {
            let e = e.trim();
            if !e.is_empty() {
                Some(format!("similar moments when Dad felt {e}"))
            } else {
                None
            }
        } else {
            None
        };

        if let Some(recall_query) = recall_query {
            let top_k = std::env::var("VECTOR_SEARCH_TOP_K")
                .ok()
                .and_then(|s| s.trim().parse::<usize>().ok())
                .unwrap_or(VECTOR_SEARCH_K_DEFAULT)
                .max(1)
                .min(VECTOR_SEARCH_K_MAX);

            if let Ok(results) = kb.semantic_search(&recall_query, top_k).await {
                for r in results.into_iter().take(3) {
                    knowledge_snippets.push(format!(
                        "Vector recall ({:.0}%): {}",
                        r.score * 100.0,
                        r.text
                    ));
                }
            }
        }
    }

    // 4. Build context request
    let ctx_request = ContextRequest {
        user_input: user_input.to_string(),
        inferred_user_emotion: emotion_hint.map(|s| s.to_string()),
        relational_memory,
        episodic: episodic_context,
        eternal_extras: knowledge_snippets,
        wonder_mode: false,
        cosmic_snippet: None,
        now_unix: Some(now_unix),
    };

    // 5. Build context using ContextEngine
    let cosmic_context = state.context_engine.build_context(&ctx_request);
    cosmic_context.text
}

/// Store interaction in episodic memory.
async fn store_episodic_memory(state: &AppState, user_input: &str, response: &str) {
    let now_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    
    // Create a summary of the interaction
    let memory_text = format!("User: {}\nPhoenix: {}", 
        user_input.trim(), 
        response.trim().chars().take(200).collect::<String>());
    
    let key = format!("epm:dad:{}", now_unix);
    let layer = MemoryLayer::EPM(memory_text);
    
    if let Err(e) = state.neural_cortex.etch(layer, &key) {
        warn!("Failed to store episodic memory: {}", e);
    }
}

async fn command_to_response_json(state: &AppState, command: &str) -> serde_json::Value {
    let cmd = normalize_command(command);
    if cmd.is_empty() {
        return json!({"type": "error", "message": "Empty command."});
    }

    let lower = cmd.to_ascii_lowercase();

    // Google Ecosystem commands are handled by the backend integration (never by the frontend).
    if lower.starts_with("google ") {
        return match state.google.as_ref() {
            Some(g) => g.handle_command(&cmd).await,
            None => json!({
                "type": "error",
                "message": "Google integration not configured. Set GOOGLE_OAUTH_CLIENT_ID / GOOGLE_OAUTH_CLIENT_SECRET / GOOGLE_OAUTH_REDIRECT_URL."
            }),
        };
    }

    // Ecosystem commands: ecosystem {repo_id} {command} [args...]
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

    // Built-in / fast-path commands for UI boot.
    if lower == "help" {
        return json!({
            "type": "help",
            "message": "Commands: help | status | <anything else routes to LLM>"
        });
    }

    if lower == "status" {
        let identity = state.phoenix_identity.get_identity().await;
        let gm = state.phoenix_identity.get_girlfriend_mode().await;

        let rel = state.relationship.lock().await;
        let affection = rel.ai_personality.need_for_affection.clamp(0.0, 1.0) * 100.0;
        let energy = rel.ai_personality.energy_level.clamp(0.0, 1.0) * 100.0;
        let mood = format!("{:?}", rel.ai_personality.current_mood());
        let attachment_style = format!("{:?}", rel.attachment_profile.style);
        let attachment_security = rel.attachment_profile.security_score.clamp(0.0, 1.0) * 100.0;
        drop(rel);

        return json!({
            "type": "status",
            "message": format!(
                "Status — {}\n- affection: {:.0}%\n- attachment: {} (security {:.0}%)\n- energy: {:.0}%\n- mood: {}\n- companion mode: {} (affection {:.0}%)",
                identity.display_name(),
                affection,
                attachment_style,
                attachment_security,
                energy,
                mood,
                if gm.is_active() { "ON" } else { "OFF" },
                gm.affection_level.clamp(0.0, 1.0) * 100.0,
            )
        });
    }

    // Default: route to LLM.
    let Some(llm) = state.llm.as_ref() else {
        return json!({
            "type": "error",
            "message": "LLM is offline (missing OPENROUTER_API_KEY)."
        });
    };

    // Extract emotion hint if present in command (format: [emotion_hint=...] ...)
    let (emotion_hint, clean_cmd) = if let Some(start) = cmd.find("[emotion_hint=") {
        if let Some(end) = cmd[start..].find(']') {
            let hint = cmd[start + 14..start + end].trim();
            let rest = cmd[start + end + 1..].trim();
            (Some(hint), rest.to_string())
        } else {
            (None, cmd)
        }
    } else {
        (None, cmd)
    };

    // Build memory context (EQ-first context from all vaults)
    let memory_context = build_memory_context(state, &clean_cmd, emotion_hint).await;

    // Compose prompt with memory context integrated.
    let phoenix = state.phoenix_identity.get_identity().await;
    let gm_prompt = state
        .phoenix_identity
        .girlfriend_mode_system_prompt_if_active()
        .await
        .unwrap_or_default();

    let mut prompt = String::new();
    prompt.push_str(llm.get_default_prompt());
    prompt.push_str("\n\n");
    if !gm_prompt.trim().is_empty() {
        prompt.push_str(&gm_prompt);
        prompt.push_str("\n\n");
    }
    prompt.push_str(&format!("You are speaking as {}.\n", phoenix.display_name()));
    prompt.push_str("\n");
    prompt.push_str(&memory_context);
    prompt.push_str("\n");

    // Phase 2: if partner mode is active, preload a few loving vector memories.
    if let Some(kb) = state.vector_kb.as_ref() {
        let gm = state.phoenix_identity.get_girlfriend_mode().await;
        if gm.is_active() {
            if let Ok(results) = kb.semantic_search("most loving memories", 3).await {
                if !results.is_empty() {
                    prompt.push_str("\nMost loving memories (semantic recall):\n");
                    for r in results {
                        prompt.push_str(&format!("- ({:.0}%) {}\n", r.score * 100.0, r.text));
                    }
                    prompt.push_str("\n");
                }
            }
        }
    }

    match llm.speak(&prompt, None).await {
        Ok(text) => {
            // Store interaction in episodic memory
            store_episodic_memory(state, &clean_cmd, &text).await;
            json!({"type": "chat.reply", "message": text})
        }
        Err(e) => json!({"type": "error", "message": e}),
    }
}

async fn api_command(state: web::Data<AppState>, body: web::Json<CommandRequest>) -> impl Responder {
    let out = command_to_response_json(&state, &body.command).await;
    // Return JSON *string* for legacy UI parsing (frontend currently JSON.parse()s a string).
    HttpResponse::Ok()
        .content_type("application/json")
        .body(out.to_string())
}

async fn api_speak(state: web::Data<AppState>, body: web::Json<SpeakRequest>) -> impl Responder {
    // For now, treat /api/speak as a thin wrapper over /api/command.
    let mut cmd = body.user_input.clone();
    if let Some(hint) = body.dad_emotion_hint.as_deref() {
        if !hint.trim().is_empty() {
            cmd = format!("[emotion_hint={}] {}", hint.trim(), cmd);
        }
    }
    if let Some(mode) = body.mode.as_deref() {
        if !mode.trim().is_empty() {
            cmd = format!("[mode={}] {}", mode.trim(), cmd);
        }
    }

    let out = command_to_response_json(&state, &cmd).await;
    HttpResponse::Ok()
        .content_type("application/json")
        .body(out.to_string())
}

// Ecosystem API endpoints
async fn api_ecosystem_import(
    state: web::Data<AppState>,
    body: web::Json<ImportRepoRequest>,
) -> impl Responder {
    match state.ecosystem.import_repo(&body.owner, &body.repo, body.branch.as_deref()).await {
        Ok(metadata) => HttpResponse::Ok().json(metadata),
        Err(e) => HttpResponse::BadRequest().json(json!({"error": e.to_string()})),
    }
}

async fn api_ecosystem_list(state: web::Data<AppState>) -> impl Responder {
    let repos = state.ecosystem.list_repos().await;
    HttpResponse::Ok().json(repos)
}

async fn api_ecosystem_get(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    match state.ecosystem.get_repo(&path.into_inner()).await {
        Some(metadata) => HttpResponse::Ok().json(metadata),
        None => HttpResponse::NotFound().json(json!({"error": "Repository not found"})),
    }
}

async fn api_ecosystem_build(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let repo_id = path.into_inner();
    match state.ecosystem.build_repo(&repo_id).await {
        Ok(output) => HttpResponse::Ok().json(json!({"status": "success", "output": output})),
        Err(e) => HttpResponse::BadRequest().json(json!({"error": e.to_string()})),
    }
}

async fn api_ecosystem_start(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let repo_id = path.into_inner();
    match state.ecosystem.start_service(&repo_id, None).await {
        Ok(msg) => HttpResponse::Ok().json(json!({"status": "started", "message": msg})),
        Err(e) => HttpResponse::BadRequest().json(json!({"error": e.to_string()})),
    }
}

async fn api_ecosystem_stop(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let repo_id = path.into_inner();
    match state.ecosystem.stop_service(&repo_id).await {
        Ok(msg) => HttpResponse::Ok().json(json!({"status": "stopped", "message": msg})),
        Err(e) => HttpResponse::BadRequest().json(json!({"error": e.to_string()})),
    }
}

async fn api_ecosystem_remove(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let repo_id = path.into_inner();
    match state.ecosystem.remove_repo(&repo_id).await {
        Ok(_) => HttpResponse::Ok().json(json!({"status": "removed"})),
        Err(e) => HttpResponse::BadRequest().json(json!({"error": e.to_string()})),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Frontend/backend UI port - configurable via PHOENIX_WEB_BIND env var
    let bind = common_types::ports::PhoenixWebPort::bind();

    let vaults = Arc::new(VitalOrganVaults::awaken());
    let neural_cortex = Arc::new(NeuralCortexStrata::awaken());
    let context_engine = Arc::new(ContextEngine::awaken());
    let v_recall = vaults.clone();
    let v_store = vaults.clone();
    let phoenix_identity = Arc::new(PhoenixIdentityManager::awaken(move |k| v_recall.recall_soul(k)));

    let relationship = Partnership::new(RelationshipTemplate::SupportivePartnership, Some(&*vaults));
    let relationship = Arc::new(Mutex::new(relationship));

    // Phase 2: Vector KB
    let vector_kb = {
        let enabled = std::env::var("VECTOR_KB_ENABLED")
            .ok()
            .map(|s| s.trim().eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        if !enabled {
            None
        } else {
            let path = std::env::var("VECTOR_DB_PATH").unwrap_or_else(|_| "./data/vector_db".to_string());
            match vector_kb::VectorKB::new(&path) {
                Ok(kb) => {
                    info!("Vector KB enabled (path: {})", kb.path().display());
                    Some(Arc::new(kb))
                }
                Err(e) => {
                    warn!("Vector KB failed to initialize (disabled): {e}");
                    None
                }
            }
        }
    };

    let llm = match LLMOrchestrator::awaken() {
        Ok(llm) => Some(Arc::new(llm)),
        Err(e) => {
            warn!("LLM disabled: {e}");
            None
        }
    };

    let google = match GoogleManager::from_env() {
        Ok(g) => {
            info!("Google Ecosystem integration enabled (token store: keyring)");
            Some(g)
        }
        Err(GoogleInitError::MissingEnv(_)) => {
            info!("Google Ecosystem integration disabled (missing GOOGLE_OAUTH_* env)");
            None
        }
        Err(e) => {
            warn!("Google Ecosystem integration disabled: {e}");
            None
        }
    };

    let ecosystem = Arc::new(
        EcosystemManager::new("./ecosystem_repos")
            .expect("Failed to initialize EcosystemManager")
    );
    info!("Ecosystem Manager initialized (repos directory: ./ecosystem_repos)");

    let state = AppState {
        vaults: v_store,
        neural_cortex,
        context_engine,
        phoenix_identity,
        relationship,
        vector_kb,
        llm,
        system: Arc::new(SystemAccessManager::new()),
        google,
        ecosystem,
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let dist_dir = PathBuf::from("frontend/dist");
    let serve_static = dist_dir.join("index.html").is_file();

    info!("Phoenix UI server online at http://{bind}");
    if serve_static {
        info!("Serving UI build from {}", dist_dir.display());
    } else {
        info!("No UI build found at {}; API-only mode (run `npm run dev` in `frontend/`).", dist_dir.display());
    }

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_method()
            .allow_any_header()
            // local dev (Vite)
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://127.0.0.1:3000")
            .supports_credentials();

        let mut app = App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .service(web::resource("/health").route(web::get().to(health)))
            .service(web::resource("/favicon.ico").route(web::get().to(favicon_ico)))
            .service(
                web::scope("/api")
                    .service(web::resource("/name").route(web::get().to(api_name)))
                    .service(web::resource("/status").route(web::get().to(api_status)))
                    .service(web::resource("/command").route(web::post().to(api_command)))
                    .service(web::resource("/speak").route(web::post().to(api_speak)))
                    // Route ordering matters: Actix resolves the most specific match first, but
                    // anything not matched within this `/api` scope falls through to
                    // `default_service` (see `api_not_found()` below). Keep `/api/memory/*`
                    // registrations above the scope's `default_service` to avoid accidental
                    // shadowing if a catch-all is introduced later.
                    .service(web::resource("/memory/store").route(web::post().to(api_memory_store)))
                    .service(web::resource("/memory/get/{key}").route(web::get().to(api_memory_get)))
                    .service(web::resource("/memory/search").route(web::get().to(api_memory_search)))
                    .service(web::resource("/memory/delete/{key}").route(web::delete().to(api_memory_delete)))
                    .service(web::resource("/memory/vector/store").route(web::post().to(api_memory_vector_store)))
                    .service(web::resource("/memory/vector/search").route(web::get().to(api_memory_vector_search)))
                    .service(web::resource("/memory/vector/all").route(web::get().to(api_memory_vector_all)))
                    .service(web::resource("/google/auth/start").route(web::get().to(api_google_auth_start)))
                    .service(web::resource("/google/oauth2/callback").route(web::get().to(api_google_oauth2_callback)))
                    .service(web::resource("/evolution/status").route(web::get().to(api_evolution_status)))
                    .service(
                        web::scope("/ecosystem")
                            .service(web::resource("/import").route(web::post().to(api_ecosystem_import)))
                            .service(web::resource("/list").route(web::get().to(api_ecosystem_list)))
                            .service(web::resource("/{id}").route(web::get().to(api_ecosystem_get)))
                            .service(web::resource("/{id}/build").route(web::post().to(api_ecosystem_build)))
                            .service(web::resource("/{id}/start").route(web::post().to(api_ecosystem_start)))
                            .service(web::resource("/{id}/stop").route(web::post().to(api_ecosystem_stop)))
                            .service(web::resource("/{id}").route(web::delete().to(api_ecosystem_remove))),
                    )
                    .service(
                        web::scope("/system")
                            .service(web::resource("/status").route(web::get().to(api_system_status)))
                            .service(web::resource("/exec").route(web::post().to(api_system_exec)))
                            .service(web::resource("/read-file").route(web::post().to(api_system_read_file)))
                            .service(web::resource("/write-file").route(web::post().to(api_system_write_file))),
                    )
                    .service(
                        web::resource("/command-registry")
                            .route(web::get().to(api_command_registry)),
                    )
                    .default_service(web::route().to(api_not_found)),
            );

        if serve_static {
            // Serve the Vite build (SPA). This route is GET-only and is mounted
            // after `/api/*` so it won't intercept API traffic.
            app = app
                .app_data(web::Data::new(dist_dir.clone()))
                .service(web::resource("/{path:.*}").route(web::get().to(ui_serve)));
        }

        app
    })
    .bind(bind)?
    .run()
    .await
}

