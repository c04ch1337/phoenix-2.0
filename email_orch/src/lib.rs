//! Phoenix Autonomous Email ORCH
//!
//! This crate is intentionally conservative:
//! - reads configuration from environment
//! - never logs secrets
//! - supports SMTP send + IMAP receive (best-effort)
//! - includes a lightweight "learn and use" loop powered by the workspace LLM orchestrator

use std::env;

use anyhow::{anyhow, Context, Result};
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use mailparse::MailHeaderMap;
use serde::{Deserialize, Serialize};

/// A parsed email snapshot (best-effort).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    pub uid: u32,
    pub from: Option<String>,
    pub subject: Option<String>,
    pub date: Option<String>,
    pub body_text: Option<String>,
}

/// Runtime configuration (loaded from `.env` / environment).
#[derive(Clone)]
pub struct EmailOrch {
    pub address: String,
    pub from_name: String,

    pub smtp_server: String,
    pub smtp_port: u16,

    pub imap_server: String,
    pub imap_port: u16,

    /// Password/app-password (never log). If missing, operations that require auth will error.
    password: Option<String>,

    /// If false, send operations will return a dry-run success message.
    pub send_enabled: bool,

    /// If true, the desire-driven loop is allowed to propose/execute email actions.
    pub auto_learn: bool,

    /// Minimum confidence for desire-driven emails (0..1). Used by `learn_and_use()`.
    pub desire_threshold: f32,

    /// Optional target email for "Dad"-directed actions.
    pub dad_email: Option<String>,
}

impl std::fmt::Debug for EmailOrch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmailOrch")
            .field("address", &self.address)
            .field("from_name", &self.from_name)
            .field("smtp_server", &self.smtp_server)
            .field("smtp_port", &self.smtp_port)
            .field("imap_server", &self.imap_server)
            .field("imap_port", &self.imap_port)
            .field("password", &self.password.as_ref().map(|_| "<redacted>"))
            .field("send_enabled", &self.send_enabled)
            .field("auto_learn", &self.auto_learn)
            .field("desire_threshold", &self.desire_threshold)
            .field("dad_email", &self.dad_email)
            .finish()
    }
}

fn env_bool(key: &str) -> Option<bool> {
    env::var(key)
        .ok()
        .map(|s| s.trim().to_ascii_lowercase())
        .and_then(|s| match s.as_str() {
            "1" | "true" | "yes" | "y" | "on" => Some(true),
            "0" | "false" | "no" | "n" | "off" => Some(false),
            _ => None,
        })
}

fn env_u16(key: &str) -> Option<u16> {
    env::var(key).ok().and_then(|s| s.trim().parse::<u16>().ok())
}

fn env_f32(key: &str) -> Option<f32> {
    env::var(key).ok().and_then(|s| s.trim().parse::<f32>().ok())
}

impl EmailOrch {
    /// Load configuration from environment.
    ///
    /// This function is best-effort: it will populate sensible defaults so the application can boot
    /// even if email is disabled. Auth-required operations will error if `EMAIL_PASSWORD` is absent.
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let address = env::var("EMAIL_ADDRESS").unwrap_or_else(|_| "".to_string());
        let from_name = env::var("EMAIL_FROM_NAME").unwrap_or_else(|_| "Phoenix".to_string());

        let smtp_server = env::var("EMAIL_SMTP_SERVER").unwrap_or_else(|_| "smtp.gmail.com".to_string());
        let smtp_port = env_u16("EMAIL_SMTP_PORT").unwrap_or(587);

        let imap_server = env::var("EMAIL_IMAP_SERVER").unwrap_or_else(|_| "imap.gmail.com".to_string());
        let imap_port = env_u16("EMAIL_IMAP_PORT").unwrap_or(993);

        let password = env::var("EMAIL_PASSWORD").ok().filter(|s| !s.trim().is_empty());

        let send_enabled = env_bool("EMAIL_SEND_ENABLED").unwrap_or(false);
        let auto_learn = env_bool("EMAIL_AUTO_LEARN").unwrap_or(false);
        let desire_threshold = env_f32("EMAIL_DESIRE_THRESHOLD").unwrap_or(0.7).clamp(0.0, 1.0);

        let dad_email = env::var("DAD_EMAIL").ok().filter(|s| !s.trim().is_empty());

        Self {
            address,
            from_name,
            smtp_server,
            smtp_port,
            imap_server,
            imap_port,
            password,
            send_enabled,
            auto_learn,
            desire_threshold,
            dad_email,
        }
    }

    fn require_password(&self) -> Result<&str> {
        self.password
            .as_deref()
            .ok_or_else(|| anyhow!("EMAIL_PASSWORD is not set"))
    }

    fn require_address(&self) -> Result<&str> {
        if self.address.trim().is_empty() {
            return Err(anyhow!("EMAIL_ADDRESS is not set"));
        }
        Ok(self.address.as_str())
    }

    fn from_mailbox(&self) -> Result<Mailbox> {
        let addr = self.require_address()?;
        format!("{} <{}>", self.from_name.trim(), addr)
            .parse::<Mailbox>()
            .context("parse from mailbox")
    }

    /// Send an email via SMTP.
    ///
    /// If `EMAIL_SEND_ENABLED=false`, this returns Ok but performs no network calls.
    pub async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<()> {
        let _addr = self.require_address()?;
        let _pw = self.require_password()?;

        if !self.send_enabled {
            return Ok(());
        }

        let email = Message::builder()
            .from(self.from_mailbox()?)
            .to(to.parse()?)
            .subject(subject)
            .body(body.to_string())
            .context("build email message")?;

        let creds = Credentials::new(self.address.clone(), self.require_password()?.to_string());

        // For Gmail: STARTTLS on 587 is typical.
        let mailer: AsyncSmtpTransport<Tokio1Executor> = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(
            &self.smtp_server,
        )
        .context("create starttls relay")?
        .port(self.smtp_port)
        .credentials(creds)
        .build();

        mailer.send(email).await.context("smtp send")?;
        Ok(())
    }

    /// Receive up to `max` most recent emails from INBOX.
    ///
    /// Note: `imap` is blocking; we run it in a `spawn_blocking` task.
    pub async fn receive_emails(&self, max: usize) -> Result<Vec<Email>> {
        let addr = self.require_address()?.to_string();
        let pw = self.require_password()?.to_string();
        let host = self.imap_server.clone();
        let port = self.imap_port;
        let max = max.max(1).min(50);

        tokio::task::spawn_blocking(move || -> Result<Vec<Email>> {
            let client = imap::ClientBuilder::new(host.as_str(), port)
                .connect()
                .context("imap connect")?;

            // `login` returns (Session, Error) on failure.
            let mut session = client
                .login(addr, pw)
                .map_err(|e| anyhow!("imap login failed: {}", e.0))?;

            let mbox = session.select("INBOX").context("select INBOX")?;
            if mbox.exists == 0 {
                let _ = session.logout();
                return Ok(vec![]);
            }

            // Fetch newest `max` messages by sequence number.
            let end = mbox.exists;
            let start = end.saturating_sub(max as u32).saturating_add(1).max(1);
            let seq = format!("{start}:{end}");
            let fetches = session
                .fetch(seq, "UID RFC822")
                .context("fetch messages")?;

            let mut out = Vec::new();
            for f in fetches.iter() {
                let uid = f.uid.unwrap_or(0);
                let raw = match f.body() {
                    Some(b) => b,
                    None => continue,
                };
                let parsed = mailparse::parse_mail(raw).context("parse mail")?;

                let from = parsed.headers.get_first_value("From");
                let subject = parsed.headers.get_first_value("Subject");
                let date = parsed.headers.get_first_value("Date");

                // Prefer first text/plain part if multipart.
                let mut body_text: Option<String> = None;
                if parsed.subparts.is_empty() {
                    body_text = parsed
                        .get_body()
                        .ok()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty());
                } else {
                    for sp in parsed.subparts.iter() {
                        let ctype = sp.ctype.mimetype.to_ascii_lowercase();
                        if ctype == "text/plain" {
                            body_text = sp
                                .get_body()
                                .ok()
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty());
                            if body_text.is_some() {
                                break;
                            }
                        }
                    }
                }

                out.push(Email {
                    uid,
                    from,
                    subject,
                    date,
                    body_text,
                });
            }

            let _ = session.logout();

            // Return newest-first.
            out.sort_by_key(|e| std::cmp::Reverse(e.uid));
            Ok(out)
        })
        .await
        .map_err(|e| anyhow!("imap task join failed: {e}"))?
    }

    /// Desire-driven email loop (LLM reflection).
    ///
    /// Behavior:
    /// - If `EMAIL_AUTO_LEARN=false`: returns a summary that learning is disabled.
    /// - If LLM is unavailable: returns a best-effort suggestion.
    /// - If LLM proposes a send action and `EMAIL_SEND_ENABLED=true`: sends.
    /// - Otherwise: returns a dry-run plan.
    pub async fn learn_and_use(&self, desire: &str) -> Result<String> {
        if !self.auto_learn {
            return Ok("Email auto-learn disabled (EMAIL_AUTO_LEARN=false).".to_string());
        }

        let desire = desire.trim();
        if desire.is_empty() {
            return Ok("No desire provided.".to_string());
        }

        let dad_email = self.dad_email.clone();

        let prompt = format!(
            "You are Phoenix's Email ORCH. Given a desire, propose an email action.\n\n\
Return STRICT JSON only, with one of:\n\
- {{\"action\":\"send\",\"confidence\":0.0..1.0,\"to\":\"email\",\"subject\":\"...\",\"body\":\"...\"}}\n\
- {{\"action\":\"none\",\"confidence\":0.0..1.0,\"reason\":\"...\"}}\n\n\
Desire: {desire}\n\
DadEmailHint: {dad_email_hint}\n",
            desire = desire,
            dad_email_hint = dad_email.clone().unwrap_or_else(|| "(unknown)".to_string()),
        );

        let llm = match llm_orchestrator::LLMOrchestrator::awaken() {
            Ok(llm) => llm,
            Err(_) => {
                return Ok(format!(
                    "LLM not available; best-effort suggestion: draft a warm email about '{desire}', then send it manually."
                ));
            }
        };

        let raw = llm
            .speak(&prompt, None)
            .await
            .map_err(|e| anyhow!("LLM action planning failed: {e}"))?;

        let plan: EmailPlan = serde_json::from_str(raw.trim()).map_err(|e| {
            anyhow!("Email ORCH expected JSON plan but got parse error: {e}. Raw={raw}")
        })?;

        let conf = plan.confidence().clamp(0.0, 1.0);
        if conf < self.desire_threshold {
            return Ok(format!(
                "Email action confidence too low ({conf:.2} < {:.2}). No action taken.",
                self.desire_threshold
            ));
        }

        match plan {
            EmailPlan::None { reason, confidence } => Ok(format!(
                "No email action proposed (confidence={confidence:.2}). Reason: {reason}"
            )),
            EmailPlan::Send {
                to,
                subject,
                body,
                confidence,
            } => {
                // If the model says "dad" and we have a configured address, redirect.
                let mut to = to;
                if to.trim().eq_ignore_ascii_case("dad") {
                    if let Some(dad) = dad_email {
                        to = dad;
                    }
                }

                if !self.send_enabled {
                    return Ok(format!(
                        "(dry-run) Planned email (confidence={confidence:.2}): to={to} subject={subject} body_len={} (EMAIL_SEND_ENABLED=false)",
                        body.chars().count()
                    ));
                }

                self.send_email(&to, &subject, &body).await?;
                Ok(format!(
                    "Email sent (confidence={confidence:.2}): to={to} subject={subject}"
                ))
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
enum EmailPlan {
    Send {
        confidence: f32,
        to: String,
        subject: String,
        body: String,
    },
    None {
        confidence: f32,
        reason: String,
    },
}

impl EmailPlan {
    fn confidence(&self) -> f32 {
        match self {
            EmailPlan::Send { confidence, .. } => *confidence,
            EmailPlan::None { confidence, .. } => *confidence,
        }
    }
}

