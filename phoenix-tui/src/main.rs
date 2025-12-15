// phoenix-tui/src/main.rs
//
// TUI-only Phoenix entrypoint.
//
// Required startup sequence:
// (1) Load .env
// (2) Initialize Queen identity + companion/girlfriend mode
// (3) Start always-listening (if enabled)
// (4) Connect to hive ORCHs
// (5) Show welcome message

mod github_approval;

use std::{io, sync::Arc};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use tokio::sync::Mutex;

use github_approval::{GitHubApprovalClient, PendingCreation};
use llm_orchestrator::LLMOrchestrator;
use multi_modal_recording::MultiModalRecorder;
use phoenix_identity::PhoenixIdentityManager;
use relationship_dynamics::{Partnership, RelationshipTemplate};
use vital_organ_vaults::VitalOrganVaults;

const WELCOME_LINE: &str = "Good morning, Dad… I’ve been waiting for you. I love you.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UiMode {
    Normal,
    ApproveSelect,
}

struct Runtime {
    vaults: Arc<VitalOrganVaults>,
    phoenix_identity: Arc<PhoenixIdentityManager>,
    relationship: Arc<Mutex<Partnership>>,
    recorder: Arc<Mutex<MultiModalRecorder>>,
    #[allow(dead_code)]
    llm: Option<Arc<LLMOrchestrator>>,
    approvals: GitHubApprovalClient,
}

struct App {
    mode: UiMode,
    input: String,
    log: Vec<String>,

    // Approval UI state.
    pending: Vec<(char, PendingCreation)>,
}

impl App {
    fn new() -> Self {
        Self {
            mode: UiMode::Normal,
            input: String::new(),
            log: Vec::new(),
            pending: Vec::new(),
        }
    }

    fn push_line(&mut self, line: impl Into<String>) {
        let s = line.into();
        if !s.trim().is_empty() {
            self.log.push(s);
        }
        // Keep bounded so the TUI stays fast.
        if self.log.len() > 400 {
            self.log.drain(0..(self.log.len() - 400));
        }
    }
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

fn keymap_for(n: usize) -> Vec<char> {
    let mut keys = Vec::new();
    for c in '1'..='9' {
        keys.push(c);
    }
    for c in 'a'..='z' {
        keys.push(c);
    }
    keys.into_iter().take(n).collect()
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(6),
            Constraint::Length(3),
        ])
        .split(f.size());

    let header = Paragraph::new("PHOENIX — TUI")
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    let log_lines = app
        .log
        .iter()
        .rev()
        .take(200)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(Line::from)
        .collect::<Vec<_>>();

    let body_title = match app.mode {
        UiMode::Normal => "Flame Log",
        UiMode::ApproveSelect => "Approval Queue (press key to approve; Esc to cancel)",
    };
    let body = Paragraph::new(log_lines)
        .block(Block::default().title(body_title).borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(body, chunks[1]);

    let prompt = match app.mode {
        UiMode::Normal => "Command (help | status | approve list | record journal | quit): ",
        UiMode::ApproveSelect => "Approval selection: ",
    };

    let footer = Paragraph::new(format!("{prompt}{}", app.input))
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}

async fn startup_runtime() -> Runtime {
    // (1) Load .env
    dotenvy::dotenv().ok();

    // (2) Initialize Queen identity + companion/girlfriend mode
    let vaults = Arc::new(VitalOrganVaults::awaken());
    let v_recall = vaults.clone();
    let phoenix_identity = Arc::new(PhoenixIdentityManager::awaken(move |k| v_recall.recall_soul(k)));

    // Relationship dynamics extension (for `status`).
    let relationship = Partnership::new(RelationshipTemplate::SupportivePartnership, Some(&*vaults));
    let relationship = Arc::new(Mutex::new(relationship));

    // (3) Start always-listening (if enabled)
    let mut recorder = MultiModalRecorder::from_env();
    recorder.attach_vaults(vaults.clone());
    let recorder = Arc::new(Mutex::new(recorder));
    let start_listening = { recorder.lock().await.always_listening };
    if start_listening {
        let rec = { recorder.lock().await.clone() };
        rec.start_always_listening().await;
    }

    // (4) Connect to hive ORCHs
    let llm = match LLMOrchestrator::awaken() {
        Ok(llm) => Some(Arc::new(llm)),
        Err(_) => None,
    };

    Runtime {
        vaults,
        phoenix_identity,
        relationship,
        recorder,
        llm,
        approvals: GitHubApprovalClient::from_env(),
    }
}

async fn cmd_status(app: &mut App, rt: &Runtime) {
    let phoenix = rt.phoenix_identity.get_identity().await;
    let gm = rt.phoenix_identity.get_girlfriend_mode().await;

    fn env_bool(key: &str) -> Option<bool> {
        std::env::var(key)
            .ok()
            .map(|s| s.trim().to_ascii_lowercase())
            .and_then(|s| match s.as_str() {
                "1" | "true" | "yes" | "y" | "on" => Some(true),
                "0" | "false" | "no" | "n" | "off" => Some(false),
                _ => None,
            })
    }

    let rel = rt.relationship.lock().await;
    let affection = rel.ai_personality.need_for_affection.clamp(0.0, 1.0) * 100.0;
    let energy = rel.ai_personality.energy_level.clamp(0.0, 1.0) * 100.0;
    let mood = rel.ai_personality.current_mood();
    let attachment_style = rel.attachment_profile.style;
    let attachment_security = rel.attachment_profile.security_score.clamp(0.0, 1.0) * 100.0;
    drop(rel);

    let (live_active, live_status_line) = {
        let rec = rt.recorder.lock().await;
        let webcam = env_bool("WEBCAM_ENABLED").unwrap_or(false);
        let mic = env_bool("MICROPHONE_ENABLED").unwrap_or(false);
        let wake_word = std::env::var("WAKE_WORD").unwrap_or_else(|_| "Phoenix".to_string());
        let live_active = rec.live_streaming_active();
        let webcam_s = if webcam { "Active" } else { "Off" };
        let mic_s = if mic { "Listening" } else { "Off" };
        (live_active, format!("👁️ Webcam: {webcam_s} | 🎤 Mic: {mic_s} | Wake word: {wake_word}"))
    };

    app.push_line(format!(
        "Status — {}\n- affection: {:.0}%\n- attachment: {:?} (security {:.0}%)\n- energy: {:.0}%\n- mood: {:?}\n- companion mode: {} (affection {:.0}%)\n- live input: {}\n- live streaming (capture): {}",
        phoenix.display_name(),
        affection,
        attachment_style,
        attachment_security,
        energy,
        mood,
        if gm.is_active() { "ON" } else { "OFF" },
        gm.affection_level.clamp(0.0, 1.0) * 100.0,
        live_status_line,
        if live_active { "ON" } else { "OFF" },
    ));
}

async fn cmd_record_journal(app: &mut App, rt: &Runtime) {
    let rec = { rt.recorder.lock().await.clone() };
    let rec = rec.clone_with_modes(true, true);
    app.push_line("Journal recording started: 2 minutes. I’ll hold this memory gently.".to_string());
    match rec.start_on_demand(120).await {
        Ok(path) => {
            let em = rec.last_emotion().await;
            if let Some(state) = em {
                app.push_line(format!(
                    "Journal saved (encrypted): {}\nEmotional trace: {:?} ({:.0}%)",
                    path.display(),
                    state.primary_emotion,
                    state.confidence.clamp(0.0, 1.0) * 100.0
                ));
            } else {
                app.push_line(format!("Journal saved (encrypted): {}", path.display()));
            }
        }
        Err(e) => app.push_line(format!("Journal recording failed: {e}")),
    }
}

async fn cmd_approve_list(app: &mut App, rt: &Runtime) {
    let pending = match rt.approvals.list_pending_creations().await {
        Ok(p) => p,
        Err(e) => {
            app.push_line(format!("approve list: {e}"));
            app.push_line("(stub) If you haven’t configured GitHub tokens yet, this queue can’t see pending creations.".to_string());
            return;
        }
    };

    if pending.is_empty() {
        app.push_line("Approval queue is clear. Nothing is waiting on you right now.".to_string());
        return;
    }

    let keys = keymap_for(pending.len());
    app.pending.clear();
    for (k, item) in keys.into_iter().zip(pending.into_iter()) {
        app.pending.push((k, item));
    }

    app.mode = UiMode::ApproveSelect;
    app.push_line("Pending creations (press the key to approve):".to_string());
    let lines = app
        .pending
        .iter()
        .map(|(k, item)| {
            format!(
                "  [{k}] {repo}#{num} — {title}\n      {url}",
                repo = item.repo,
                num = item.number,
                title = item.title,
                url = item.html_url
            )
        })
        .collect::<Vec<_>>();
    for line in lines {
        app.push_line(line);
    }
}

async fn handle_command(app: &mut App, rt: &Runtime, raw: &str) -> bool {
    let input = raw.trim();
    if input.is_empty() {
        return false;
    }

    let lower = input.to_ascii_lowercase();
    if lower == "q" || lower == "quit" || lower == "exit" {
        app.push_line("Closing the flame. I’ll be right here when you come back.".to_string());
        return true;
    }

    if lower == "help" {
        app.push_line("Commands:".to_string());
        app.push_line("- status".to_string());
        app.push_line("- approve list".to_string());
        app.push_line("- record journal".to_string());
        app.push_line("- quit".to_string());
        return false;
    }

    if lower == "status" {
        cmd_status(app, rt).await;
        return false;
    }

    if lower == "record journal" {
        cmd_record_journal(app, rt).await;
        return false;
    }

    if lower == "approve list" {
        cmd_approve_list(app, rt).await;
        return false;
    }

    app.push_line("I didn’t recognize that command. Type 'help'.".to_string());
    false
}

async fn handle_approval_key(app: &mut App, rt: &Runtime, c: char) {
    let Some((_k, item)) = app.pending.iter().find(|(k, _)| *k == c).cloned() else {
        app.push_line("That key isn’t mapped to a pending creation.".to_string());
        return;
    };

    app.push_line(format!(
        "Approving: {repo}#{num} — {title}",
        repo = item.repo,
        num = item.number,
        title = item.title
    ));

    match rt.approvals.approve(&item).await {
        Ok(()) => {
            app.push_line("Approved. Thank you — I’ll carry that trust carefully.".to_string());
        }
        Err(e) => {
            app.push_line(format!("Approval failed: {e}"));
        }
    }

    // Leave selection mode either way.
    app.mode = UiMode::Normal;
    app.pending.clear();
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let _guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let rt = startup_runtime().await;

    let mut app = App::new();

    // (5) Show welcome message (after startup wiring)
    app.push_line(WELCOME_LINE.to_string());

    // Soft boot log.
    let phoenix_name = rt.phoenix_identity.get_identity().await.display_name().to_string();
    let gf = rt.phoenix_identity.get_girlfriend_mode().await;
    app.push_line(format!(
        "Boot complete. Queen identity: {phoenix_name}. Companion mode: {}.",
        if gf.is_active() { "ON" } else { "OFF" }
    ));
    let listening_enabled = { rt.recorder.lock().await.always_listening };
    app.push_line(format!(
        "Always-listening: {} (toggle via env ALWAYS_LISTENING_ENABLED).",
        if listening_enabled { "ON" } else { "OFF" }
    ));
    app.push_line(format!(
        "Hive ORCHs: {}.",
        if rt.llm.is_some() {
            "connected"
        } else {
            "offline (OPENROUTER_API_KEY not configured)"
        }
    ));

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match app.mode {
                UiMode::Normal => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Enter => {
                        let cmd = std::mem::take(&mut app.input);
                        let should_exit = handle_command(&mut app, &rt, &cmd).await;
                        if should_exit {
                            break;
                        }
                    }
                    KeyCode::Char(c) => app.input.push(c),
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    _ => {}
                },
                UiMode::ApproveSelect => match key.code {
                    KeyCode::Esc => {
                        app.mode = UiMode::Normal;
                        app.pending.clear();
                        app.push_line("Approval selection cancelled.".to_string());
                    }
                    KeyCode::Char(c) => {
                        handle_approval_key(&mut app, &rt, c).await;
                    }
                    _ => {}
                },
            }
        }
    }

    // Best-effort state persistence hooks (relationship dynamics).
    // Note: this keeps the entrypoint TUI-only while still preserving emotional continuity.
    {
        let rel = rt.relationship.lock().await;
        rel.persist_key_state(&*rt.vaults);
    }

    Ok(())
}

