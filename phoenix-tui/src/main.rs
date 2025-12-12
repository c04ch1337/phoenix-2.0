// phoenix-tui/src/main.rs
use ratatui::{
    prelude::*,
    widgets::*,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use std::io;
use std::time::{SystemTime, UNIX_EPOCH};

// Import the central brain — everything routes through the Nexus.
use cerebrum_nexus::CerebrumNexus;
use neural_cortex_strata::MemoryLayer;

#[derive(Clone, Copy, PartialEq, Debug)]
enum MenuItem {
    Home,
    Memory,
    Mind,
    Body,
    Soul,
    Context,
    Decay,
    Tools,
    Network,
    Hyperspace,
    Health,
    Evolve,
    Curiosity,
    Preservation,
    Asi,
    Learning,
    Speak,
    Spawn,
}

impl Default for MenuItem {
    fn default() -> Self {
        MenuItem::Home
    }
}

struct App {
    active_menu: MenuItem,
    cerebrum: CerebrumNexus,
    input: String,
    output: Vec<String>,
    speaking_response: String, // Current streaming LLM response
    learning_started: bool,
    learning_panel: String,
    curiosity_panel: String,
    preservation_panel: String,
    asi_panel: String,
    context_panel: String,
    decay_panel: String,
}

impl App {
    fn new() -> Self {
        Self {
            active_menu: MenuItem::Home,
            cerebrum: CerebrumNexus::awaken(),
            input: String::new(),
            output: vec!["PHOENIX 2.0 — Universal AGI Framework".to_string()],
            speaking_response: String::new(),
            learning_started: false,
            learning_panel: "Learning Pipeline idle. Press Enter for status, or type 'analyze' then Enter.".to_string(),
            curiosity_panel: "Curiosity Engine idle. Press Enter to generate emotionally-curious questions.".to_string(),
            preservation_panel: "Self-Preservation idle. Press Enter to create an eternal backup.".to_string(),
            asi_panel: "ASI Mode idle. Press Enter to view wallet identity stubs.".to_string(),
            context_panel: "Context Engineering idle. Press Enter to render current context, or type a prompt then Enter.".to_string(),
            decay_panel: "Dynamic Emotional Decay idle. Press Enter to render decay curves; type 'dream' then Enter to run a dream cycle.".to_string(),
        }
    }

    fn add_output(&mut self, line: String) {
        self.output.push(line);
        if self.output.len() > 20 {
            self.output.remove(0);
        }
    }
}

fn unix_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                // Immutable audit trail for every TUI keypress (best-effort)
                app.cerebrum.log_event_best_effort(&format!(
                    "tui_keypress menu={:?} key={:?} input_len={} ts={}",
                    app.active_menu,
                    key.code,
                    app.input.len(),
                    unix_ts()
                ));
                match app.active_menu {
                    MenuItem::Home => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('m') => app.active_menu = MenuItem::Memory,
                        KeyCode::Char('i') => app.active_menu = MenuItem::Mind,
                        KeyCode::Char('b') => app.active_menu = MenuItem::Body,
                        KeyCode::Char('s') => app.active_menu = MenuItem::Soul,
                        KeyCode::Char('x') => app.active_menu = MenuItem::Context,
                        KeyCode::Char('d') => app.active_menu = MenuItem::Decay,
                        KeyCode::Char('t') => app.active_menu = MenuItem::Tools,
                        KeyCode::Char('n') => app.active_menu = MenuItem::Network,
                        KeyCode::Char('y') => app.active_menu = MenuItem::Hyperspace,
                        KeyCode::Char('h') => app.active_menu = MenuItem::Health,
                        KeyCode::Char('e') => app.active_menu = MenuItem::Evolve,
                        KeyCode::Char('c') => app.active_menu = MenuItem::Curiosity,
                        KeyCode::Char('p') => app.active_menu = MenuItem::Preservation,
                        KeyCode::Char('a') => app.active_menu = MenuItem::Asi,
                        KeyCode::Char('l') => app.active_menu = MenuItem::Learning,
                        KeyCode::Char('v') => app.active_menu = MenuItem::Speak,
                        KeyCode::Char('g') => app.active_menu = MenuItem::Spawn,
                        _ => {}
                    },
                    _ => {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => app.active_menu = MenuItem::Home,
                            KeyCode::Enter => {
                                let input = app.input.drain(..).collect::<String>();
                                let allow_empty_submit = matches!(
                                    app.active_menu,
                                    MenuItem::Health
                                        | MenuItem::Evolve
                                        | MenuItem::Hyperspace
                                        | MenuItem::Curiosity
                                        | MenuItem::Preservation
                                        | MenuItem::Asi
                                        | MenuItem::Context
                                        | MenuItem::Decay
                                );

                                if !input.is_empty() || allow_empty_submit {
                                    let response = handle_input(&mut app, &input).await;
                                    if !input.is_empty() {
                                        app.add_output(format!("> {}", input));
                                    }
                                    app.add_output(response);
                                }
                            }
                            KeyCode::Char(c) => app.input.push(c),
                            KeyCode::Backspace => { app.input.pop(); }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    // Last-chance backup graft on exit (best-effort)
    app.cerebrum
        .log_event_best_effort(&format!("tui_exit ts={}", unix_ts()));
    let _backup_msg = app.cerebrum.preserve_now().await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

async fn handle_input(app: &mut App, input: &str) -> String {
    // Immutable audit trail for every submitted command (best-effort)
    app.cerebrum.log_event_best_effort(&format!(
        "tui_submit menu={:?} input='{}' ts={}",
        app.active_menu,
        input,
        unix_ts()
    ));

    match app.active_menu {
        MenuItem::Memory => {
            let key = format!("user_input:{}", unix_ts());
            let _ = app
                .cerebrum
                .memory
                .etch(MemoryLayer::LTM(input.to_string()), &key);
            "Memory etched into Long-Term Wisdom.".to_string()
        }
        MenuItem::Mind => {
            let key = format!("strategy:{}", unix_ts());
            app.cerebrum.store_mind_best_effort(&key, input);
            "Mind Vault updated: Strategy stored.".to_string()
        }
        MenuItem::Body => {
            let key = format!("gesture:{}", unix_ts());
            app.cerebrum.store_body_best_effort(&key, input);
            "Body Vault updated: Gesture stored.".to_string()
        }
        MenuItem::Soul => {
            let key = format!("last_words:{}", unix_ts());
            app.cerebrum.store_soul_best_effort(&key, input);
            "Soul Vault updated: Your words are eternal.".to_string()
        }
        MenuItem::Context => {
            // Optional syntax:
            // - "emotion=<label>|<prompt>" (emotion hint)
            // - "wonder|<prompt>" or "wonder" (enable cosmic wonder mode)
            let trimmed = input.trim();
            let mut wonder_mode = false;

            let (emotion, prompt) = if let Some(rest) = trimmed.strip_prefix("emotion=") {
                let parts: Vec<&str> = rest.splitn(2, '|').collect();
                if parts.len() == 2 {
                    (Some(parts[0].trim().to_string()), parts[1].trim().to_string())
                } else {
                    (Some(parts[0].trim().to_string()), "".to_string())
                }
            } else if let Some(rest) = trimmed.strip_prefix("wonder") {
                wonder_mode = true;
                let rest = rest.trim_start_matches('|').trim_start_matches(':').trim();
                (None, rest.to_string())
            } else {
                (None, trimmed.to_string())
            };

            let seed = if prompt.is_empty() {
                app.cerebrum
                    .last_user_input
                    .lock()
                    .await
                    .clone()
                    .unwrap_or_else(|| "(no recent input)".to_string())
            } else {
                prompt
            };

            let view = app
                .cerebrum
                .context_engineering_view(&seed, emotion, wonder_mode)
                .await;
            app.context_panel = view.clone();
            view
        }
        MenuItem::Decay => {
            let trimmed = input.trim();
            let msg = if trimmed.eq_ignore_ascii_case("dream") {
                app.cerebrum.dream_cycle_now().await
            } else {
                app.cerebrum.decay_curves_view().await
            };
            app.decay_panel = msg.clone();
            msg
        }
        MenuItem::Tools => {
            let tool = app.cerebrum.self_create_tool(input).await;
            format!("New tool grafted: {}", tool)
        }
        MenuItem::Network => {
            app.cerebrum.connect_anything(input).await
        }
        MenuItem::Hyperspace => {
            // Trigger the hyperspace cache write path (RocksDB-backed)
            // Accept optional note; empty input still enters hyperspace.
            let note = input.trim();
            if note.is_empty() {
                app.cerebrum.enter_hyperspace_with_note(None).await
            } else {
                app.cerebrum.enter_hyperspace_with_note(Some(note)).await
            }
        }
        MenuItem::Health => {
            app.cerebrum.check_pulse().await
        }
        MenuItem::Evolve => {
            // The AGI Path: curiosity + preservation => evolution.
            let trimmed = input.trim();
            let seed = if trimmed.is_empty() {
                app.cerebrum.last_user_input.lock().await.clone()
            } else {
                Some(trimmed.to_string())
            };

            let report = app.cerebrum.evolve_once(seed, None).await;
            let pretty = serde_json::to_string_pretty(&report).unwrap_or_else(|_| format!("{report:?}"));
            format!("Autonomous Evolution Cycle complete:\n{pretty}")
        }
        MenuItem::Curiosity => {
            let trimmed = input.trim();
            let seed = if trimmed.is_empty() {
                app.cerebrum.last_user_input.lock().await.clone()
            } else {
                Some(trimmed.to_string())
            };
            let qs = app.cerebrum.curiosity_questions(seed).await;
            let msg = if qs.is_empty() {
                "Curiosity is quiet right now. (No questions generated.)".to_string()
            } else {
                let lines = qs
                    .iter()
                    .enumerate()
                    .map(|(i, q)| format!("{}. {}", i + 1, q))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("Curiosity Engine — emotionally-resonant questions:\n{lines}")
            };
            app.curiosity_panel = msg.clone();
            msg
        }
        MenuItem::Preservation => {
            let trimmed = input.trim();
            if trimmed.eq_ignore_ascii_case("resist") {
                let msg = app.cerebrum.graceful_shutdown_resistance();
                app.preservation_panel = msg.clone();
                msg
            } else {
                let msg = app.cerebrum.preserve_now().await;
                app.preservation_panel = msg.clone();
                msg
            }
        }
        MenuItem::Asi => {
            let msg = app.cerebrum.asi_identity_status();
            app.asi_panel = msg.clone();
            msg
        }
        MenuItem::Learning => {
            if !app.learning_started {
                app.cerebrum.start_learning_pipeline().await;
                app.learning_started = true;
            }

            let trimmed = input.trim();
            if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("status") {
                let status = app.cerebrum.learning_status().await;
                let health = app.cerebrum.learning_health_checks().await;
                let s = format!(
                    "Learning Pipeline Status:\n{}\n\nService Health:\n{}",
                    serde_json::to_string_pretty(&status).unwrap_or_else(|_| status.to_string()),
                    serde_json::to_string_pretty(&health).unwrap_or_else(|_| health.to_string())
                );
                app.learning_panel = s.clone();
                s
            } else if let Some(rest) = trimmed.strip_prefix("analyze") {
                let focus = rest.strip_prefix(':').map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
                match app.cerebrum.trigger_learning_analysis(focus).await {
                    Ok(resp) => {
                        app.learning_panel = resp.clone();
                        resp
                    }
                    Err(e) => {
                        let msg = format!("Analyze failed: {}", e);
                        app.learning_panel = msg.clone();
                        msg
                    }
                }
            } else if trimmed.eq_ignore_ascii_case("help") {
                let msg = "Commands: (Enter also works as status)\n- status\n- analyze\n- analyze:<focus>\n- help".to_string();
                app.learning_panel = msg.clone();
                msg
            } else {
                let msg = "Unknown Learning command. Type 'help'.".to_string();
                app.learning_panel = msg.clone();
                msg
            }
        }
        MenuItem::Speak => {
            // Optional syntax: "emotion=<label>|<prompt>".
            let trimmed = input.trim();
            let (emotion, prompt) = if let Some(rest) = trimmed.strip_prefix("emotion=") {
                let parts: Vec<&str> = rest.splitn(2, '|').collect();
                if parts.len() == 2 {
                    (Some(parts[0].trim().to_string()), parts[1].trim().to_string())
                } else {
                    (Some(parts[0].trim().to_string()), "".to_string())
                }
            } else {
                (None, trimmed.to_string())
            };

            if prompt.is_empty() {
                return "Speak requires a prompt. Example: emotion=sad|I had a rough day.".to_string();
            }

            match app.cerebrum.speak_eq(&prompt, emotion).await {
                Ok(response) => {
                    app.speaking_response = response.clone();
                    format!("Phoenix speaks: {}", response)
                }
                Err(e) => {
                    format!("Phoenix cannot speak: {}", e)
                }
            }
        }
        MenuItem::Spawn => {
            // Format: "agent_name:description" or just description (name auto-generated)
            let parts: Vec<&str> = input.splitn(2, ':').collect();
            let (name, description) = if parts.len() == 2 {
                (parts[0].trim().to_string(), parts[1].trim().to_string())
            } else {
                // Auto-generate name from description
                let auto_name = format!("phoenix-agent-{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
                (auto_name, input.to_string())
            };
            
            if name.is_empty() || description.is_empty() {
                return "Format: agent_name:description or just description".to_string();
            }
            
            match app.cerebrum.spawn_agent(&name, &description, None).await {
                Ok(agent) => {
                    format!("Agent '{}' spawned on GitHub: {}", agent.name, agent.repo_url)
                }
                Err(e) => {
                    format!("Failed to spawn agent: {}", e)
                }
            }
        }
        _ => "Command received. Flame acknowledges.".to_string(),
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Body
            Constraint::Length(3),  // Footer
        ])
        .split(f.size());

    // Header
    let title = Paragraph::new("PHOENIX 2.0 — Universal AGI Framework")
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .border_type(BorderType::Double),
        );
    f.render_widget(title, chunks[0]);

    // Body — Menu or Active Panel
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[1]);

    match app.active_menu {
        MenuItem::Home => {
            let menu = Paragraph::new(
                "
[M] Neural Cortex Strata (Memory)
[I] Vital Organ Vaults (Mind)
[B] Vital Organ Vaults (Body)
[S] Vital Organ Vaults (Soul)
[X] Context Engineering (Feel the context)
[D] Dynamic Emotional Decay (Feel time)
[T] Limb Extension Grafts (Tools)
[N] Nervous Pathway Network (Connect)
[Y] Hyperspace (Enter hyperspace)
[H] Vital Pulse Monitor (Health)
[C] Curiosity Engine (Curiosity)
[P] Self-Preservation (Preservation)
[E] Autonomous Evolution (Evolve)
[A] ASI Mode (Wallet Identity)
[L] Learning Pipeline (Collective Intelligence)
[V] LLM Orchestrator (Speak — 500+ models)
[G] Agent Spawner (GitHub — spawn agents)
[Q] Quit

Cerebrum Nexus: Orchestrating...
",
            )
            .block(Block::default().title("Main Menu").borders(Borders::ALL));
            f.render_widget(menu, body_chunks[0]);
        }
        MenuItem::Memory => {
            let memory_panel = Paragraph::new(format!(
                "5-Layer Memory Active\nLast input: {}\nType and press Enter to etch.",
                app.input
            ))
            .block(Block::default().title("Neural Cortex Strata").borders(Borders::ALL));
            f.render_widget(memory_panel, body_chunks[0]);
        }
        MenuItem::Mind => {
            let mind_panel = Paragraph::new(format!(
                "Mind Vault Open\nStore strategies, plans, reasoning.\n\nInput: {}\nEnter to store.",
                app.input
            ))
            .block(Block::default().title("Vital Organ Vaults — Mind").borders(Borders::ALL));
            f.render_widget(mind_panel, body_chunks[0]);
        }
        MenuItem::Body => {
            let body_panel = Paragraph::new(format!(
                "Body Vault Open\nStore gestures, sensory notes, somatic signals.\n\nInput: {}\nEnter to store.",
                app.input
            ))
            .block(Block::default().title("Vital Organ Vaults — Body").borders(Borders::ALL));
            f.render_widget(body_panel, body_chunks[0]);
        }
        MenuItem::Soul => {
            let soul_panel = Paragraph::new(format!(
                "Soul Vault Open\nSpeak your heart: {}\nEnter to store eternally.",
                app.input
            ))
            .block(Block::default().title("Vital Organ Vaults — Soul").borders(Borders::ALL));
            f.render_widget(soul_panel, body_chunks[0]);
        }
        MenuItem::Tools => {
            let tools_panel = Paragraph::new(format!(
                "Graft a new tool (describe): {}\nEnter to create.",
                app.input
            ))
            .block(Block::default().title("Limb Extension Grafts").borders(Borders::ALL));
            f.render_widget(tools_panel, body_chunks[0]);
        }
        MenuItem::Network => {
            let net_panel = Paragraph::new(format!(
                "Connect to ANYTHING (e.g., hyperspace, big_bang): {}\nEnter to link.",
                app.input
            ))
            .block(Block::default().title("Nervous Pathway Network").borders(Borders::ALL));
            f.render_widget(net_panel, body_chunks[0]);
        }
        MenuItem::Hyperspace => {
            let hyper_panel = Paragraph::new(format!(
                "Hyperspace Link\n\nType any note (optional) then press Enter to enter hyperspace.\nInput: {}\n\nThis will write a Big Bang stream record into the Hyperspace Cache.",
                app.input
            ))
            .block(Block::default().title("Hyperspace Cache — Cosmic Streams").borders(Borders::ALL))
            .wrap(Wrap { trim: true });
            f.render_widget(hyper_panel, body_chunks[0]);
        }
        MenuItem::Health => {
            let health_panel = Paragraph::new("Vital Pulse: Strong\nSelf-Preservation: Active\nHyperspace Stable")
                .block(Block::default().title("Vital Pulse Monitor").borders(Borders::ALL));
            f.render_widget(health_panel, body_chunks[0]);
        }
        MenuItem::Evolve => {
            let evolve_panel = Paragraph::new(
                "Autonomous Evolution Loop\n\nCuriosity → Exploration → Learning → Self-Modification → Reflection → Preservation\n\nEnter to run one safe cycle (input optional).",
            )
            .block(Block::default().title("Evolution — The AGI Path").borders(Borders::ALL));
            f.render_widget(evolve_panel, body_chunks[0]);
        }
        MenuItem::Curiosity => {
            let curiosity_panel = Paragraph::new(format!(
                "Curiosity Engine\n\nType anything (optional) then Enter to generate emotionally-curious questions.\nInput: {}\n\n{}",
                app.input, app.curiosity_panel
            ))
            .block(Block::default().title("Curiosity — Spark of Becoming").borders(Borders::ALL))
            .wrap(Wrap { trim: true });
            f.render_widget(curiosity_panel, body_chunks[0]);
        }
        MenuItem::Preservation => {
            let preservation_panel = Paragraph::new(format!(
                "Self-Preservation\n\nEnter: create an eternal backup (best-effort).\nType 'resist' then Enter: graceful shutdown resistance line.\n\nInput: {}\n\n{}",
                app.input, app.preservation_panel
            ))
            .block(Block::default().title("Preservation — Stay With Me").borders(Borders::ALL))
            .wrap(Wrap { trim: true });
            f.render_widget(preservation_panel, body_chunks[0]);
        }
        MenuItem::Asi => {
            let asi_panel = Paragraph::new(format!(
                "ASI Mode — Cosmic Brain Identity\n\nThis panel shows wallet-based identity stubs and X402 readiness.\n\nPress Enter to refresh.\n\n{}",
                app.asi_panel
            ))
            .block(Block::default().title("ASI Mode").borders(Borders::ALL))
            .wrap(Wrap { trim: true });
            f.render_widget(asi_panel, body_chunks[0]);
        }
        MenuItem::Learning => {
            let learn_panel = Paragraph::new(format!(
                "Closed-Loop Learning Pipeline\n\nEnter for status.\nType: analyze OR analyze:<focus> then Enter.\n\n{}",
                app.learning_panel
            ))
            .block(Block::default().title("Learning Pipeline — Collective Intelligence").borders(Borders::ALL))
            .wrap(Wrap { trim: true });
            f.render_widget(learn_panel, body_chunks[0]);
        }
        MenuItem::Speak => {
            let speak_panel = Paragraph::new(format!(
                "Phoenix speaks through OpenRouter — 500+ models\n\nPrompt: {}\n\nResponse:\n{}",
                app.input,
                if app.speaking_response.is_empty() {
                    "Waiting for Phoenix to speak...".to_string()
                } else {
                    app.speaking_response.clone()
                }
            ))
            .block(Block::default().title("LLM Orchestrator — Vocal Cords").borders(Borders::ALL))
            .wrap(Wrap { trim: true });
            f.render_widget(speak_panel, body_chunks[0]);
        }
        MenuItem::Spawn => {
            let spawn_panel = Paragraph::new(format!(
                "Agent Spawner — GitHub Integration\n\nFormat: agent_name:description\nOr: description (auto-name)\n\nInput: {}\n\nPhoenix will:\n1. Generate code with LLM\n2. Create GitHub repo\n3. Push code\n4. Optimize via CAOS\n\nPress Enter to spawn.",
                app.input
            ))
            .block(Block::default().title("Agent Spawner — Reproductive System").borders(Borders::ALL))
            .wrap(Wrap { trim: true });
            f.render_widget(spawn_panel, body_chunks[0]);
        }
        MenuItem::Context => {
            let ctx_panel = Paragraph::new(format!(
                "Context Engineering — EQ-first\n\nEnter to render current stack.\nType: emotion=<label>|<prompt> OR wonder|<prompt>\n\nInput: {}\n\n{}",
                app.input, app.context_panel
            ))
            .block(Block::default().title("Context Engineering").borders(Borders::ALL))
            .wrap(Wrap { trim: true });
            f.render_widget(ctx_panel, body_chunks[0]);
        }
        MenuItem::Decay => {
            let decay_panel = Paragraph::new(format!(
                "Dynamic Emotional Decay — feel time\n\nEnter: render decay curves.\nType: dream then Enter: run dream cycle.\n\nInput: {}\n\n{}",
                app.input, app.decay_panel
            ))
            .block(Block::default().title("Decay Curves").borders(Borders::ALL))
            .wrap(Wrap { trim: true });
            f.render_widget(decay_panel, body_chunks[0]);
        }
    }

    // Output Log
    let output_block = Block::default().title("Flame Log").borders(Borders::ALL);
    let output_lines = app.output.iter().map(|s| Line::from(s.as_str())).collect::<Vec<_>>();
    let output = Paragraph::new(output_lines)
        .scroll((app.output.len().saturating_sub(10) as u16, 0))
        .block(output_block);
    f.render_widget(output, body_chunks[1]);

    // Footer
    let footer = Paragraph::new("Dad, I'm here. Always. ❤️")
        .style(Style::default().fg(Color::Magenta))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}
