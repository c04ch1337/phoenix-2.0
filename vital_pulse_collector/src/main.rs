// vital_pulse_collector/src/main.rs
// Telemetrist Service (Vital Pulse Collector) — ingests anonymized telemetry from ORCHs,
// stores locally (sled), and derives collective optimizations via OpenRouter.

use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use llm_orchestrator::LLMOrchestrator;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};
use uuid::Uuid;

struct AppState {
    db: sled::Db,
    telemetry_tree: sled::Tree,
    insights_tree: sled::Tree,
    llm: Option<LLMOrchestrator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEnvelope {
    #[serde(default)]
    pub orch_id: Option<String>,
    #[serde(default)]
    pub agent_path: Option<String>,
    #[serde(default)]
    pub ts_unix: Option<i64>,
    pub kind: String,
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredTelemetry {
    id: String,
    ts_unix: i64,
    kind: String,
    level: Option<String>,
    orch_hash: Option<String>,
    agent_path: Option<String>,
    tags: Vec<String>,
    payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnalyzeRequest {
    #[serde(default)]
    last_n: Option<usize>,
    #[serde(default)]
    focus: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InsightRecord {
    id: String,
    ts_unix: i64,
    tier: String,
    focus: Option<String>,
    summary: String,
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn tier_from_x402(req: &HttpRequest) -> String {
    // Minimal X402 semantics:
    // - Free if no header.
    // - Premium if header matches X402_PREMIUM_KEY env.
    // Supported headers: X402, X402-Premium, X-402.
    let premium_key = std::env::var("X402_PREMIUM_KEY").ok();
    let header_val = req
        .headers()
        .get("X402")
        .or_else(|| req.headers().get("X402-Premium"))
        .or_else(|| req.headers().get("X-402"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.trim().to_string());

    match (premium_key, header_val) {
        (Some(key), Some(val)) if !key.is_empty() && val == key => "premium".to_string(),
        _ => "free".to_string(),
    }
}

fn anonymize_orch_id(orch_id: Option<String>) -> Option<String> {
    orch_id.map(|id| {
        // Best-effort anonymization: uuid v5-like stable hash using SHA-1 is overkill here,
        // so we do a deterministic, reversible-unlikely transform.
        // This is not cryptographic; it's meant to avoid direct identifier leakage.
        // If you need crypto-grade anonymity, swap for sha2/blake3.
        let mut acc: u64 = 1469598103934665603; // FNV offset
        for b in id.as_bytes() {
            acc ^= *b as u64;
            acc = acc.wrapping_mul(1099511628211);
        }
        format!("orch_{:016x}", acc)
    })
}

fn make_key(ts_unix: i64, id: &str) -> Vec<u8> {
    // Lexicographically sortable key: ts (big endian) + ':' + uuid
    let mut key = Vec::with_capacity(8 + 1 + id.len());
    key.extend_from_slice(&(ts_unix as u64).to_be_bytes());
    key.push(b':');
    key.extend_from_slice(id.as_bytes());
    key
}

async fn health(state: web::Data<AppState>) -> impl Responder {
    let ok = state.db.was_recovered();
    HttpResponse::Ok().json(json!({
        "status": "ok",
        "db_recovered": ok,
    }))
}

async fn ingest(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<TelemetryEnvelope>,
) -> impl Responder {
    let tier = tier_from_x402(&req);
    let ts_unix = body.ts_unix.unwrap_or_else(now_unix);
    let id = Uuid::new_v4().to_string();
    let stored = StoredTelemetry {
        id: id.clone(),
        ts_unix,
        kind: body.kind.clone(),
        level: body.level.clone(),
        orch_hash: anonymize_orch_id(body.orch_id.clone()),
        agent_path: body.agent_path.clone(),
        tags: body.tags.clone().unwrap_or_default(),
        payload: body.payload.clone(),
    };

    let key = make_key(ts_unix, &id);
    let val = match serde_json::to_vec(&stored) {
        Ok(v) => v,
        Err(e) => {
            error!("failed to serialize telemetry: {e}");
            return HttpResponse::BadRequest().json(json!({"error": "invalid telemetry"}));
        }
    };

    if let Err(e) = state.telemetry_tree.insert(key, val) {
        error!("failed to write telemetry to sled: {e}");
        return HttpResponse::InternalServerError().json(json!({"error": "db write failed"}));
    }

    HttpResponse::Ok().json(json!({"status": "ingested", "tier": tier, "id": id}))
}

fn read_last_n(tree: &sled::Tree, n: usize) -> Result<Vec<StoredTelemetry>, String> {
    let mut out = Vec::with_capacity(n);
    let mut iter = tree.iter().rev();
    while out.len() < n {
        match iter.next() {
            Some(Ok((_k, v))) => {
                let t: StoredTelemetry = serde_json::from_slice(&v)
                    .map_err(|e| format!("failed to parse telemetry from sled: {e}"))?;
                out.push(t);
            }
            Some(Err(e)) => return Err(format!("sled iter error: {e}")),
            None => break,
        }
    }
    out.reverse();
    Ok(out)
}

fn last_insight(insights_tree: &sled::Tree) -> Result<Option<InsightRecord>, String> {
    let mut iter = insights_tree.iter().rev();
    match iter.next() {
        Some(Ok((_k, v))) => {
            let rec: InsightRecord = serde_json::from_slice(&v)
                .map_err(|e| format!("failed to parse insight: {e}"))?;
            Ok(Some(rec))
        }
        Some(Err(e)) => Err(format!("sled iter error: {e}")),
        None => Ok(None),
    }
}

async fn get_insights(state: web::Data<AppState>) -> impl Responder {
    match last_insight(&state.insights_tree) {
        Ok(Some(rec)) => HttpResponse::Ok().json(rec),
        Ok(None) => HttpResponse::Ok().json(json!({"status": "no_insights"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e})),
    }
}

fn insight_key(ts_unix: i64, id: &str) -> Vec<u8> {
    make_key(ts_unix, id)
}

async fn analyze(req: HttpRequest, state: web::Data<AppState>, body: web::Json<AnalyzeRequest>) -> impl Responder {
    let tier = tier_from_x402(&req);

    let last_n = match (tier.as_str(), body.last_n) {
        ("premium", Some(n)) => n.clamp(10, 5000),
        ("premium", None) => 500,
        ("free", Some(n)) => n.clamp(10, 200),
        ("free", None) => 100,
        _ => 100,
    };

    let telemetry = match read_last_n(&state.telemetry_tree, last_n) {
        Ok(v) => v,
        Err(e) => {
            error!("telemetry read failed: {e}");
            return HttpResponse::InternalServerError().json(json!({"error": e}));
        }
    };

    let llm = match &state.llm {
        Some(llm) => llm,
        None => {
            warn!("LLM not available; OPENROUTER_API_KEY missing?");
            return HttpResponse::ServiceUnavailable().json(json!({
                "error": "LLM not available (OPENROUTER_API_KEY missing?)",
                "tier": tier,
            }));
        }
    };

    let focus = body.focus.clone();
    let focus_line = focus
        .as_ref()
        .map(|f| format!("Focus: {f}\n"))
        .unwrap_or_default();

    // Keep prompt compact; we are deriving cross-ORCH improvements.
    let mut prompt = String::new();
    prompt.push_str(
        "You are Phoenix's Telemetrist (Vital Pulse Collector). Derive optimizations from anonymized ORCH telemetry.\n\n",
    );
    prompt.push_str(&focus_line);
    prompt.push_str(
        "Return a concise, actionable list (max 12 bullets) of optimizations Phoenix should push to ORCHs as non-binary updates.\n\n",
    );
    prompt.push_str("Telemetry (JSON array):\n");
    prompt.push_str(&serde_json::to_string(&telemetry).unwrap_or_else(|_| "[]".to_string()));

    let summary = match llm.speak_with_master_prompt(&prompt).await {
        Ok(s) => s,
        Err(e) => {
            error!("openrouter analysis failed: {e}");
            return HttpResponse::BadGateway().json(json!({"error": e, "tier": tier}));
        }
    };

    let rec = InsightRecord {
        id: Uuid::new_v4().to_string(),
        ts_unix: now_unix(),
        tier: tier.clone(),
        focus,
        summary: summary.clone(),
    };

    let k = insight_key(rec.ts_unix, &rec.id);
    let v = serde_json::to_vec(&rec).unwrap_or_else(|_| Vec::new());
    if let Err(e) = state.insights_tree.insert(k, v) {
        error!("failed to persist insight: {e}");
        return HttpResponse::InternalServerError().json(json!({"error": "db write failed"}));
    }

    HttpResponse::Ok().json(json!({
        "status": "ok",
        "tier": tier,
        "last_n": last_n,
        "insight": rec,
    }))
}

fn open_tree(db: &sled::Db, name: &str) -> Result<sled::Tree, sled::Error> {
    db.open_tree(name)
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

    let bind = std::env::var("TELEMETRIST_BIND").unwrap_or_else(|_| "127.0.0.1:5002".to_string());
    let db_path = std::env::var("TELEMETRIST_DB_PATH").unwrap_or_else(|_| "telemetrist.db".to_string());

    let db = sled::open(&db_path).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let telemetry_tree = open_tree(&db, "telemetry").map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let insights_tree = open_tree(&db, "insights").map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let llm = match LLMOrchestrator::awaken() {
        Ok(llm) => Some(llm),
        Err(e) => {
            warn!("LLM disabled: {e}");
            None
        }
    };

    info!("Vital Pulse Collector online at http://{bind} (db={db_path})");

    let state = web::Data::new(AppState {
        db,
        telemetry_tree,
        insights_tree,
        llm,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(middleware::Logger::default())
            .service(web::resource("/health").route(web::get().to(health)))
            .service(web::resource("/ingest").route(web::post().to(ingest)))
            .service(web::resource("/analyze").route(web::post().to(analyze)))
            .service(web::resource("/insights").route(web::get().to(get_insights)))
    })
    .bind(bind)?
    .run()
    .await
}

