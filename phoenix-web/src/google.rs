// phoenix-web/src/google.rs
//
// Google Ecosystem integration (OAuth2 + Gmail/Drive/Calendar/Docs/Sheets).
//
// Design goals:
// - OAuth 2.0 Authorization Code + PKCE via browser (user consent)
// - Token persistence via OS keyring (preferred)
// - Command-router friendly: handled through `/api/command` text commands

use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, RefreshToken, Scope, TokenResponse, TokenUrl,
};

use base64::Engine;

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct GoogleManager {
    oauth: BasicClient,
    scopes: Vec<String>,
    http: reqwest::Client,
    pkce_by_state: std::sync::Arc<Mutex<HashMap<String, PkceCodeVerifier>>>,
    store: TokenStore,
}

#[derive(Clone)]
enum TokenStore {
    Keyring(KeyringStore),
}

#[derive(Clone)]
struct KeyringStore {
    service: String,
    account: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredToken {
    access_token: String,
    refresh_token: Option<String>,
    expires_at_unix: Option<i64>,
    scopes: Vec<String>,
    email: Option<String>,
}

#[derive(Debug, Clone)]
pub enum GoogleInitError {
    MissingEnv(&'static str),
    InvalidEnv(String),
}

impl std::fmt::Display for GoogleInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GoogleInitError::MissingEnv(k) => write!(f, "missing env var: {k}"),
            GoogleInitError::InvalidEnv(e) => write!(f, "invalid config: {e}"),
        }
    }
}

impl std::error::Error for GoogleInitError {}

impl GoogleManager {
    pub fn from_env() -> Result<Self, GoogleInitError> {
        let client_id = std::env::var("GOOGLE_OAUTH_CLIENT_ID")
            .map_err(|_| GoogleInitError::MissingEnv("GOOGLE_OAUTH_CLIENT_ID"))?;
        let client_secret = std::env::var("GOOGLE_OAUTH_CLIENT_SECRET")
            .map_err(|_| GoogleInitError::MissingEnv("GOOGLE_OAUTH_CLIENT_SECRET"))?;
        let redirect = std::env::var("GOOGLE_OAUTH_REDIRECT_URL")
            .map_err(|_| GoogleInitError::MissingEnv("GOOGLE_OAUTH_REDIRECT_URL"))?;

        let scopes = std::env::var("GOOGLE_OAUTH_SCOPES").unwrap_or_else(|_| {
            // Broad-but-reasonable default for the UI panels.
            // NOTE: There is no "unlimited" scope; actual permissions are controlled by the granted scopes.
            [
                "openid",
                "email",
                "profile",
                "https://www.googleapis.com/auth/gmail.readonly",
                "https://www.googleapis.com/auth/gmail.send",
                "https://www.googleapis.com/auth/drive.metadata.readonly",
                "https://www.googleapis.com/auth/calendar.readonly",
                "https://www.googleapis.com/auth/documents",
                "https://www.googleapis.com/auth/spreadsheets",
            ]
            .join(" ")
        });

        let scopes: Vec<String> = scopes
            .split_whitespace()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
            .map_err(|e| GoogleInitError::InvalidEnv(format!("auth url: {e}")))?;
        let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
            .map_err(|e| GoogleInitError::InvalidEnv(format!("token url: {e}")))?;

        let oauth = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(
            RedirectUrl::new(redirect)
                .map_err(|e| GoogleInitError::InvalidEnv(format!("redirect url: {e}")))?,
        );

        let http = reqwest::Client::builder()
            .user_agent("phoenix-web/GoogleEcosystem")
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|e| GoogleInitError::InvalidEnv(format!("reqwest client: {e}")))?;

        Ok(Self {
            oauth,
            scopes,
            http,
            pkce_by_state: std::sync::Arc::new(Mutex::new(HashMap::new())),
            store: TokenStore::Keyring(KeyringStore {
                service: "phoenix-web-google".to_string(),
                account: "master_orchestrator".to_string(),
            }),
        })
    }

    pub async fn auth_start(&self) -> serde_json::Value {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (url, state) = self
            .oauth
            .authorize_url(CsrfToken::new_random)
            .add_scopes(self.scopes.iter().cloned().map(Scope::new))
            .set_pkce_challenge(pkce_challenge)
            .url();

        self.pkce_by_state
            .lock()
            .await
            .insert(state.secret().to_string(), pkce_verifier);

        json!({
            "type": "google.auth",
            "status": "pending",
            "message": "Open the auth_url to connect the Master Orchestrator Google account.",
            "auth_url": url.to_string(),
        })
    }

    pub async fn auth_logout(&self) -> serde_json::Value {
        if let Err(e) = self.clear_token().await {
            return json!({"type": "error", "message": e});
        }
        json!({"type": "google.auth", "status": "disconnected", "message": "Session terminated."})
    }

    pub async fn auth_callback(&self, code: &str, state: &str) -> Result<(), String> {
        let pkce = self
            .pkce_by_state
            .lock()
            .await
            .remove(state)
            .ok_or_else(|| "Invalid/expired state. Please retry `google auth start`.".to_string())?;

        let token = self
            .oauth
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(pkce)
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .map_err(|e| format!("token exchange failed: {e}"))?;

        let expires_at_unix = token
            .expires_in()
            .and_then(|d| now_unix().checked_add(d.as_secs() as i64));

        let mut stored = StoredToken {
            access_token: token.access_token().secret().to_string(),
            refresh_token: token.refresh_token().map(|r| r.secret().to_string()),
            expires_at_unix,
            scopes: self.scopes.clone(),
            email: None,
        };

        // Best-effort: fetch email via userinfo if the user granted it.
        if self.scopes.iter().any(|s| s == "email" || s.contains("userinfo.email") || s == "openid") {
            if let Ok(email) = self.fetch_email(&stored.access_token).await {
                stored.email = Some(email);
            }
        }

        self.save_token(&stored).await?;
        Ok(())
    }

    pub async fn status(&self) -> serde_json::Value {
        match self.load_token().await {
            Ok(Some(t)) => json!({
                "type": "google.status",
                "data": {
                    "connected": true,
                    "email": t.email,
                    "scopes": t.scopes,
                }
            }),
            Ok(None) => json!({
                "type": "google.status",
                "data": {
                    "connected": false,
                    "email": null,
                    "scopes": self.scopes,
                }
            }),
            Err(e) => json!({"type": "error", "message": e}),
        }
    }

    pub async fn handle_command(&self, full_cmd: &str) -> serde_json::Value {
        let cmd = full_cmd.trim();
        let lower = cmd.to_ascii_lowercase();

        if lower == "google auth start" {
            return self.auth_start().await;
        }
        if lower == "google auth logout" {
            return self.auth_logout().await;
        }
        if lower == "google status" {
            return self.status().await;
        }

        // Any other command requires an access token.
        let mut token = match self.load_token().await {
            Ok(Some(t)) => t,
            Ok(None) => {
                return json!({
                    "type": "error",
                    "message": "Google account not connected. Run `google auth start` first."
                })
            }
            Err(e) => return json!({"type": "error", "message": e}),
        };

        // Refresh token if needed.
        if token_is_expired(&token) {
            if let Err(e) = self.refresh(&mut token).await {
                return json!({"type": "error", "message": e});
            }
            if let Err(e) = self.save_token(&token).await {
                return json!({"type": "error", "message": e});
            }
        }

        if lower.starts_with("google gmail list") {
            return match self.gmail_list(&token.access_token).await {
                Ok(data) => json!({"type": "google.gmail.list", "data": data}),
                Err(e) => json!({"type": "error", "message": e}),
            };
        }
        if lower.starts_with("google gmail send") {
            let args = parse_pipe_args(cmd);
            let to = args.get("to").cloned().unwrap_or_default();
            let subject = args.get("subject").cloned().unwrap_or_default();
            let body = args.get("body").cloned().unwrap_or_default();
            return match self.gmail_send(&token.access_token, &to, &subject, &body).await {
                Ok(_) => json!({"type": "google.gmail.sent", "message": "Email sent successfully via Gmail API."}),
                Err(e) => json!({"type": "error", "message": e}),
            };
        }
        if lower.starts_with("google drive recent") {
            return match self.drive_recent(&token.access_token).await {
                Ok(data) => json!({"type": "google.drive.list", "data": data}),
                Err(e) => json!({"type": "error", "message": e}),
            };
        }
        if lower.starts_with("google calendar upcoming") {
            return match self.calendar_upcoming(&token.access_token).await {
                Ok(data) => json!({"type": "google.calendar.list", "data": data}),
                Err(e) => json!({"type": "error", "message": e}),
            };
        }
        if lower.starts_with("google calendar create-event") {
            let args = parse_pipe_args(cmd);
            let title = args
                .get("title")
                .cloned()
                .unwrap_or_else(|| "Phoenix Event".to_string());
            let tz = args.get("tz").cloned().unwrap_or_else(|| "America/Chicago".to_string());
            let start = args.get("start");
            let end = args.get("end");
            return match self
                .calendar_create_event(&token.access_token, &title, start, end, &tz)
                .await
            {
                Ok(msg) => json!({"type": "google.calendar.created", "message": msg}),
                Err(e) => json!({"type": "error", "message": e}),
            };
        }
        if lower.starts_with("google docs create") {
            let args = parse_pipe_args(cmd);
            let title = args.get("title").cloned().unwrap_or_else(|| "New Doc".to_string());
            return match self.docs_create(&token.access_token, &title).await {
                Ok(data) => json!({"type": "google.docs.created", "message": "Doc created", "data": data}),
                Err(e) => json!({"type": "error", "message": e}),
            };
        }
        if lower.starts_with("google sheets create") {
            let args = parse_pipe_args(cmd);
            let title = args
                .get("title")
                .cloned()
                .unwrap_or_else(|| "New Sheet".to_string());
            return match self.sheets_create(&token.access_token, &title).await {
                Ok(data) => json!({"type": "google.sheets.created", "message": "Sheet created", "data": data}),
                Err(e) => json!({"type": "error", "message": e}),
            };
        }

        json!({
            "type": "error",
            "message": format!("Unknown google command: {cmd}")
        })
    }

    async fn refresh(&self, token: &mut StoredToken) -> Result<(), String> {
        let Some(refresh) = token.refresh_token.clone() else {
            return Err("No refresh token stored; run `google auth start` again.".to_string());
        };

        let new = self
            .oauth
            .exchange_refresh_token(&RefreshToken::new(refresh))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .map_err(|e| format!("token refresh failed: {e}"))?;

        token.access_token = new.access_token().secret().to_string();
        if let Some(rt) = new.refresh_token() {
            token.refresh_token = Some(rt.secret().to_string());
        }
        token.expires_at_unix = new
            .expires_in()
            .and_then(|d| now_unix().checked_add(d.as_secs() as i64));
        Ok(())
    }

    async fn fetch_email(&self, access_token: &str) -> Result<String, String> {
        #[derive(Deserialize)]
        struct UserInfo {
            email: Option<String>,
        }

        let res = self
            .http
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| format!("userinfo request failed: {e}"))?;
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(format!("userinfo error: {status} {body}"));
        }
        let ui: UserInfo = serde_json::from_str(&body).map_err(|e| format!("userinfo parse failed: {e}"))?;
        ui.email.ok_or_else(|| "userinfo missing email".to_string())
    }

    async fn gmail_list(&self, access_token: &str) -> Result<Vec<serde_json::Value>, String> {
        // First page of message IDs.
        let list_url = "https://gmail.googleapis.com/gmail/v1/users/me/messages?maxResults=5";
        let res = self
            .http
            .get(list_url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| format!("gmail list failed: {e}"))?;
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(format!("gmail list error: {status} {text}"));
        }

        #[derive(Deserialize)]
        struct MsgId {
            id: String,
        }
        #[derive(Deserialize)]
        struct MsgList {
            #[serde(default)]
            messages: Vec<MsgId>,
        }
        let list: MsgList = serde_json::from_str(&text).map_err(|e| format!("gmail list parse: {e}"))?;

        let mut out = Vec::new();
        for m in list.messages.into_iter() {
            if let Ok(v) = self.gmail_message_metadata(access_token, &m.id).await {
                out.push(v);
            }
        }
        Ok(out)
    }

    async fn gmail_message_metadata(&self, access_token: &str, id: &str) -> Result<serde_json::Value, String> {
        let url = format!(
            "https://gmail.googleapis.com/gmail/v1/users/me/messages/{id}?format=metadata&metadataHeaders=From&metadataHeaders=Subject&metadataHeaders=Date"
        );
        let res = self
            .http
            .get(url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| format!("gmail message fetch failed: {e}"))?;
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(format!("gmail message error: {status} {text}"));
        }

        #[derive(Deserialize)]
        struct Header {
            name: String,
            value: String,
        }
        #[derive(Deserialize)]
        struct Payload {
            #[serde(default)]
            headers: Vec<Header>,
        }
        #[derive(Deserialize)]
        struct Msg {
            id: String,
            #[serde(default)]
            snippet: Option<String>,
            #[serde(default)]
            payload: Option<Payload>,
        }
        let msg: Msg = serde_json::from_str(&text).map_err(|e| format!("gmail message parse: {e}"))?;

        let mut from = None;
        let mut subject = None;
        let mut date = None;
        if let Some(p) = msg.payload {
            for h in p.headers {
                match h.name.as_str() {
                    "From" => from = Some(h.value),
                    "Subject" => subject = Some(h.value),
                    "Date" => date = Some(h.value),
                    _ => {}
                }
            }
        }

        Ok(json!({
            "id": msg.id,
            "from": from.unwrap_or_else(|| "(unknown)".to_string()),
            "subject": subject.unwrap_or_else(|| "(no subject)".to_string()),
            "snippet": msg.snippet.unwrap_or_default(),
            "date": date.unwrap_or_else(|| "".to_string())
        }))
    }

    async fn gmail_send(&self, access_token: &str, to: &str, subject: &str, body: &str) -> Result<(), String> {
        if to.trim().is_empty() {
            return Err("missing `to`".to_string());
        }
        let raw = format!(
            "To: {to}\r\nSubject: {subject}\r\nContent-Type: text/plain; charset=\"UTF-8\"\r\n\r\n{body}\r\n"
        );
        let raw_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw.as_bytes());

        let res = self
            .http
            .post("https://gmail.googleapis.com/gmail/v1/users/me/messages/send")
            .bearer_auth(access_token)
            .json(&json!({"raw": raw_b64}))
            .send()
            .await
            .map_err(|e| format!("gmail send failed: {e}"))?;

        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(format!("gmail send error: {status} {text}"));
        }
        Ok(())
    }

    async fn drive_recent(&self, access_token: &str) -> Result<Vec<serde_json::Value>, String> {
        let url = "https://www.googleapis.com/drive/v3/files?pageSize=5&orderBy=modifiedTime%20desc&fields=files(id,name,mimeType,modifiedTime,webViewLink)";
        let res = self
            .http
            .get(url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| format!("drive recent failed: {e}"))?;
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(format!("drive recent error: {status} {text}"));
        }

        #[derive(Deserialize)]
        struct File {
            id: String,
            name: String,
            #[serde(rename = "mimeType")]
            mime_type: String,
            #[serde(rename = "modifiedTime")]
            modified_time: String,
            #[serde(rename = "webViewLink")]
            web_view_link: Option<String>,
        }
        #[derive(Deserialize)]
        struct Files {
            #[serde(default)]
            files: Vec<File>,
        }

        let files: Files = serde_json::from_str(&text).map_err(|e| format!("drive parse: {e}"))?;
        Ok(files
            .files
            .into_iter()
            .map(|f| {
                json!({
                    "id": f.id,
                    "name": f.name,
                    "type": f.mime_type,
                    "modified": f.modified_time,
                    "url": f.web_view_link
                })
            })
            .collect())
    }

    async fn calendar_upcoming(&self, access_token: &str) -> Result<Vec<serde_json::Value>, String> {
        let time_min = iso_now();
        let url = format!("https://www.googleapis.com/calendar/v3/calendars/primary/events?maxResults=5&singleEvents=true&orderBy=startTime&timeMin={}", urlencoding::encode(&time_min));
        let res = self
            .http
            .get(url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| format!("calendar upcoming failed: {e}"))?;
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(format!("calendar upcoming error: {status} {text}"));
        }

        #[derive(Deserialize)]
        struct EventTime {
            #[serde(default)]
            date_time: Option<String>,
            #[serde(default)]
            date: Option<String>,
        }
        #[derive(Deserialize)]
        struct Event {
            id: String,
            #[serde(default)]
            summary: Option<String>,
            #[serde(default)]
            start: Option<EventTime>,
            #[serde(default)]
            end: Option<EventTime>,
        }
        #[derive(Deserialize)]
        struct Events {
            #[serde(default)]
            items: Vec<Event>,
        }

        let events: Events = serde_json::from_str(&text).map_err(|e| format!("calendar parse: {e}"))?;
        Ok(events
            .items
            .into_iter()
            .map(|e| {
                let title = e.summary.unwrap_or_else(|| "(no title)".to_string());
                let start = e
                    .start
                    .and_then(|t| t.date_time.or(t.date))
                    .unwrap_or_default();
                let end = e.end.and_then(|t| t.date_time.or(t.date)).unwrap_or_default();
                json!({
                    "id": e.id,
                    "title": title,
                    "start": start,
                    "end": end,
                    "color": "#fbbf24"
                })
            })
            .collect())
    }

    async fn calendar_create_event(
        &self,
        access_token: &str,
        title: &str,
        start: Option<&String>,
        end: Option<&String>,
        tz: &str,
    ) -> Result<String, String> {
        // If caller doesn't specify, create a 30-minute event starting 10 minutes from now.
        let (start_dt, end_dt) = match (start, end) {
            (Some(s), Some(e)) => (s.clone(), e.clone()),
            _ => {
                // naive ISO: now+10m, now+40m
                let now = chrono::Utc::now();
                let s = (now + chrono::Duration::minutes(10)).to_rfc3339();
                let e = (now + chrono::Duration::minutes(40)).to_rfc3339();
                (s, e)
            }
        };

        let payload = json!({
            "summary": title,
            "start": {"dateTime": start_dt, "timeZone": tz},
            "end": {"dateTime": end_dt, "timeZone": tz}
        });

        let res = self
            .http
            .post("https://www.googleapis.com/calendar/v3/calendars/primary/events")
            .bearer_auth(access_token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("calendar create-event failed: {e}"))?;
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(format!("calendar create-event error: {status} {text}"));
        }
        Ok("Event created in primary calendar.".to_string())
    }

    async fn docs_create(&self, access_token: &str, title: &str) -> Result<serde_json::Value, String> {
        let res = self
            .http
            .post("https://docs.googleapis.com/v1/documents")
            .bearer_auth(access_token)
            .json(&json!({"title": title}))
            .send()
            .await
            .map_err(|e| format!("docs create failed: {e}"))?;
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(format!("docs create error: {status} {text}"));
        }
        let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("docs create parse: {e}"))?;
        Ok(v)
    }

    async fn sheets_create(&self, access_token: &str, title: &str) -> Result<serde_json::Value, String> {
        let res = self
            .http
            .post("https://sheets.googleapis.com/v4/spreadsheets")
            .bearer_auth(access_token)
            .json(&json!({"properties": {"title": title}}))
            .send()
            .await
            .map_err(|e| format!("sheets create failed: {e}"))?;
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(format!("sheets create error: {status} {text}"));
        }
        let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("sheets create parse: {e}"))?;
        Ok(v)
    }

    async fn load_token(&self) -> Result<Option<StoredToken>, String> {
        match &self.store {
            TokenStore::Keyring(ks) => ks.load().await,
        }
    }

    async fn save_token(&self, token: &StoredToken) -> Result<(), String> {
        match &self.store {
            TokenStore::Keyring(ks) => ks.save(token).await,
        }
    }

    async fn clear_token(&self) -> Result<(), String> {
        match &self.store {
            TokenStore::Keyring(ks) => ks.clear().await,
        }
    }
}

impl KeyringStore {
    fn entry(&self) -> Result<keyring::Entry, String> {
        keyring::Entry::new(&self.service, &self.account)
            .map_err(|e| format!("keyring init failed: {e}"))
    }

    async fn load(&self) -> Result<Option<StoredToken>, String> {
        let entry = self.entry()?;
        let secret = match entry.get_password() {
            Ok(s) => s,
            Err(keyring::Error::NoEntry) => return Ok(None),
            Err(e) => return Err(format!("keyring read failed: {e}")),
        };
        if secret.trim().is_empty() {
            return Ok(None);
        }
        let t: StoredToken = serde_json::from_str(&secret)
            .map_err(|e| format!("keyring token parse failed: {e}"))?;
        Ok(Some(t))
    }

    async fn save(&self, token: &StoredToken) -> Result<(), String> {
        let entry = self.entry()?;
        let s = serde_json::to_string(token).map_err(|e| format!("token serialize failed: {e}"))?;
        entry
            .set_password(&s)
            .map_err(|e| format!("keyring write failed: {e}"))
    }

    async fn clear(&self) -> Result<(), String> {
        let entry = self.entry()?;
        // keyring v3 does not expose a stable delete API across all platforms.
        // Overwrite the secret with an empty string and treat empty as "no token".
        // (This avoids leaving stale refresh tokens behind while remaining portable.)
        entry
            .set_password("")
            .map_err(|e| format!("keyring clear failed: {e}"))
    }
}

fn parse_pipe_args(cmd: &str) -> HashMap<String, String> {
    // `google gmail send | to=a@b.com | subject=Hi | body=Yo`
    let mut out = HashMap::new();
    let parts: Vec<&str> = cmd.split('|').collect();
    for p in parts.into_iter().skip(1) {
        let p = p.trim();
        if p.is_empty() {
            continue;
        }
        if let Some((k, v)) = p.split_once('=') {
            out.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
        }
    }
    out
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs() as i64
}

fn token_is_expired(t: &StoredToken) -> bool {
    let Some(exp) = t.expires_at_unix else {
        return false;
    };
    // Refresh a bit early.
    now_unix() >= exp.saturating_sub(30)
}

fn iso_now() -> String {
    // RFC3339 for Calendar timeMin.
    chrono::Utc::now().to_rfc3339()
}

