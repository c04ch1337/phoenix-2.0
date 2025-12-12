// synaptic_pulse_distributor/src/main.rs
// Config Update Service (Synaptic Pulse Distributor) — pushes non-binary updates to ORCHs via WebSocket.

use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_ws::{Message, ProtocolError};
use futures_util::StreamExt as _;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{error, info};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<UpdateEnvelope>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeHello {
    pub orch_id: String,
    #[serde(default)]
    pub agent_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEnvelope {
    pub update_id: String,
    pub ts_unix: i64,
    #[serde(default)]
    pub target_orch: Option<String>,
    #[serde(default)]
    pub target_agent_prefix: Option<String>,
    #[serde(default)]
    pub cascade: bool,
    /// update_type: "json_patch" | "yaml_graft" | "model_tweak" | "prompt_tweak" | "notice"
    pub update_type: String,
    /// tier_required: "free" | "premium"
    #[serde(default = "default_tier_required")]
    pub tier_required: String,
    pub payload: serde_json::Value,
}

fn default_tier_required() -> String {
    "free".to_string()
}

fn now_unix() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn tier_from_x402(req: &HttpRequest) -> String {
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

async fn health() -> impl Responder {
    HttpResponse::Ok().json(json!({"status": "ok"}))
}

#[derive(Debug, Clone, Deserialize)]
struct PublishRequest {
    #[serde(default)]
    target_orch: Option<String>,
    #[serde(default)]
    target_agent_prefix: Option<String>,
    #[serde(default)]
    cascade: bool,
    update_type: String,
    #[serde(default = "default_tier_required")]
    tier_required: String,
    payload: serde_json::Value,
}

async fn publish(req: HttpRequest, state: web::Data<AppState>, body: web::Json<PublishRequest>) -> impl Responder {
    let caller_tier = tier_from_x402(&req);
    if body.tier_required == "premium" && caller_tier != "premium" {
        return HttpResponse::PaymentRequired().json(json!({
            "error": "premium tier required",
            "required": "premium",
        }));
    }

    let env = UpdateEnvelope {
        update_id: Uuid::new_v4().to_string(),
        ts_unix: now_unix(),
        target_orch: body.target_orch.clone(),
        target_agent_prefix: body.target_agent_prefix.clone(),
        cascade: body.cascade,
        update_type: body.update_type.clone(),
        tier_required: body.tier_required.clone(),
        payload: body.payload.clone(),
    };

    match state.tx.send(env.clone()) {
        Ok(fanout) => HttpResponse::Ok().json(json!({"status": "published", "fanout": fanout, "update": env})),
        Err(e) => {
            error!("publish failed: {e}");
            HttpResponse::InternalServerError().json(json!({"error": "publish failed"}))
        }
    }
}

async fn subscribe(req: HttpRequest, body: web::Payload, state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    // Each connection gets its own broadcast receiver.
    let mut rx = state.tx.subscribe();
    let conn_id = Uuid::new_v4().to_string();
    info!("ws connected conn_id={conn_id}");

    // Handshake state from client (optional but recommended).
    let mut hello: Option<SubscribeHello> = None;

    // Single task: forward broadcasts + handle inbound frames + keepalive.
    actix_web::rt::spawn(async move {
        let mut last_pong = tokio::time::Instant::now();
        let mut ping_interval = tokio::time::interval(Duration::from_secs(15));

        loop {
            tokio::select! {
                _ = ping_interval.tick() => {
                    if last_pong.elapsed() > Duration::from_secs(60) {
                        let _ = session.close(None).await;
                        break;
                    }
                    let _ = session.ping(b"phoenix").await;
                }
                recv = rx.recv() => {
                    match recv {
                        Ok(update) => {
                            let txt = match serde_json::to_string(&update) {
                                Ok(s) => s,
                                Err(_) => continue,
                            };
                            if session.text(txt).await.is_err() {
                                break;
                            }
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            let _ = session
                                .text(json!({"type": "warning", "warning": "lagged", "skipped": skipped}).to_string())
                                .await;
                        }
                    }
                }
                msg = msg_stream.next() => {
                    let Some(msg) = msg else { break; };
                    match msg {
                        Ok(Message::Text(txt)) => {
                            if hello.is_none() {
                                if let Ok(h) = serde_json::from_str::<SubscribeHello>(&txt) {
                                    hello = Some(h);
                                    let _ = session.text(json!({"type":"hello_ack"}).to_string()).await;
                                }
                            }
                        }
                        Ok(Message::Pong(_)) => {
                            last_pong = tokio::time::Instant::now();
                        }
                        Ok(Message::Ping(bytes)) => {
                            let _ = session.pong(&bytes).await;
                        }
                        Ok(Message::Close(reason)) => {
                            let _ = session.close(reason).await;
                            break;
                        }
                        Ok(Message::Binary(_)) => {}
                        Ok(Message::Continuation(_)) => {}
                        Ok(Message::Nop) => {}
                        Err(ProtocolError::Overflow) => {
                            let _ = session.close(None).await;
                            break;
                        }
                        Err(_) => break,
                    }
                }
            }
        }

        info!("ws disconnected conn_id={conn_id}");
    });

    Ok(response)
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

    let bind = std::env::var("PULSE_DISTRIBUTOR_BIND").unwrap_or_else(|_| "127.0.0.1:5003".to_string());

    let (tx, _rx) = broadcast::channel::<UpdateEnvelope>(2048);
    let state = web::Data::new(AppState { tx });

    info!("Synaptic Pulse Distributor online at ws://{bind}/subscribe");

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(middleware::Logger::default())
            .service(web::resource("/health").route(web::get().to(health)))
            .service(web::resource("/publish").route(web::post().to(publish)))
            .service(web::resource("/subscribe").route(web::get().to(subscribe)))
    })
    .bind(bind)?
    .run()
    .await
}

