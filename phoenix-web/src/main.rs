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
use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    path::{Component, Path, PathBuf},
    sync::Arc,
};
use tokio::sync::Mutex;
use tracing::{info, warn};

use llm_orchestrator::LLMOrchestrator;
use phoenix_identity::PhoenixIdentityManager;
use relationship_dynamics::{Partnership, RelationshipTemplate};
use system_access::{CommandResult, SystemAccessManager};
use vital_organ_vaults::VitalOrganVaults;

#[derive(Clone)]
struct AppState {
    #[allow(dead_code)]
    vaults: Arc<VitalOrganVaults>,
    phoenix_identity: Arc<PhoenixIdentityManager>,
    relationship: Arc<Mutex<Partnership>>,
    llm: Option<Arc<LLMOrchestrator>>,
    system: Arc<SystemAccessManager>,
    version: String,
}

#[derive(Debug, Deserialize)]
struct CommandRequest {
    command: String,
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

async fn command_to_response_json(state: &AppState, command: &str) -> serde_json::Value {
    let cmd = normalize_command(command);
    if cmd.is_empty() {
        return json!({"type": "error", "message": "Empty command."});
    }

    let lower = cmd.to_ascii_lowercase();

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

    // Compose a single prompt. (OpenRouter request uses a single user message today.)
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
    prompt.push_str(&format!("User: {cmd}\nPhoenix:", cmd = cmd));

    match llm.speak(&prompt, None).await {
        Ok(text) => json!({"type": "chat.reply", "message": text}),
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Frontend/backend UI port is intentionally fixed to avoid configuration drift.
    // If you need a different port, change it here and in `frontend/vite.config.ts`.
    let bind = "127.0.0.1:8888".to_string();

    let vaults = Arc::new(VitalOrganVaults::awaken());
    let v_recall = vaults.clone();
    let v_store = vaults.clone();
    let phoenix_identity = Arc::new(PhoenixIdentityManager::awaken(move |k| v_recall.recall_soul(k)));

    let relationship = Partnership::new(RelationshipTemplate::SupportivePartnership, Some(&*vaults));
    let relationship = Arc::new(Mutex::new(relationship));

    let llm = match LLMOrchestrator::awaken() {
        Ok(llm) => Some(Arc::new(llm)),
        Err(e) => {
            warn!("LLM disabled: {e}");
            None
        }
    };

    let state = AppState {
        vaults: v_store,
        phoenix_identity,
        relationship,
        llm,
        system: Arc::new(SystemAccessManager::new()),
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

