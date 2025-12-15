# Frontend Plug-and-Play Stub (TUI Feature Surface)

This document is a **frontend-facing contract** for what Phoenix exposes today via the TUI and what exists as callable “feature handlers” in the core brain (Cerebrum Nexus).

Goal: a frontend developer can implement a UI by treating Phoenix as a **command router**:

```text
Frontend -> send(command_string) -> Phoenix -> returns(response_string)
```

Later you can swap the transport (TUI input line, HTTP, WebSocket, Tauri IPC) without rewriting UI logic.

---

## 1) The Minimal Command Router Contract

### Request (string)

- A single line command, e.g. `status`.

### Response (string)

- A human-readable string intended for display in a chat panel or terminal.

### Optional: Structured Command Envelope (recommended)

For frontends that want to avoid parsing strings:

```json
{
  "command": "skills.run",
  "args": {
    "id": "<uuid>",
    "input": "..."
  }
}
```

This repo does not yet provide a network server, but the command list below gives you stable identifiers and handler mapping.

---

## 2) TUI Commands (Implemented in `phoenix-tui`)

These are currently handled in [`handle_command()`](phoenix-tui/src/main.rs:307).

### `help`

- Displays available commands.

### `status`

- Handler: [`cmd_status()`](phoenix-tui/src/main.rs:195)
- Returns a multi-line status summary including:
  - Relationship dynamics stats (affection, energy, mood, attachment)
  - Companion mode state (partner/girlfriend mode)
  - Live input config (webcam/mic/wake word)

### `record journal`

- Handler: [`cmd_record_journal()`](phoenix-tui/src/main.rs:244)
- Effect:
  - Starts a 120-second audio+video recording (if features are enabled)
  - Saves encrypted file and stores an emotional trace

### `approve list`

- Handler: [`cmd_approve_list()`](phoenix-tui/src/main.rs:266)
- Effect:
  - Lists pending GitHub PR approvals (if configured)
  - Enters “Approval Select” mode

### “Approval Select mode” (single keypress)

- Handler: [`handle_approval_key()`](phoenix-tui/src/main.rs:347)
- Effect:
  - Approves the selected PR

### `quit` / `exit` / `q`

- Exits TUI.

---

## 3) Core Brain Feature Handlers (Available in `cerebrum_nexus`)

These are implemented in [`CerebrumNexus`](cerebrum_nexus/src/lib.rs:64). Frontends can expose them once you route input through `CerebrumNexus::speak_eq()`.

### Chat (primary)

- Handler: [`CerebrumNexus::speak_eq()`](cerebrum_nexus/src/lib.rs:1199)
- Notes:
  - Auto-stores episodic traces
  - Builds EQ-first context
  - Runs self-critic
  - Records utility timeline
  - Best-effort: learns skill candidates from high-love interactions

### Skills

- Command router: [`CerebrumNexus::handle_skill_command()`](cerebrum_nexus/src/lib.rs:1454)
- View: [`CerebrumNexus::skills_view()`](cerebrum_nexus/src/lib.rs:1552)

Supported commands:

- `skills` / `skills list`
- `skills run <uuid> | input=...`
- `skills prefs list`
- `skills prefs add <text>`
- `skills prefs clear`

Implementation crate: [`skill_system`](skill_system/src/lib.rs:1)

### Dreaming / Healing

- Lucid dreaming view: [`CerebrumNexus::lucid_view()`](cerebrum_nexus/src/lib.rs:743)
- Lucid dreaming command: [`CerebrumNexus::lucid_command()`](cerebrum_nexus/src/lib.rs:827)
- Shared dreaming view: [`CerebrumNexus::shared_dream_view()`](cerebrum_nexus/src/lib.rs:754)
- Shared dreaming command: [`CerebrumNexus::shared_dream_command()`](cerebrum_nexus/src/lib.rs:763)
- Healing view: [`CerebrumNexus::healing_view()`](cerebrum_nexus/src/lib.rs:674)
- Healing command: [`CerebrumNexus::healing_command()`](cerebrum_nexus/src/lib.rs:689)
- Dream recordings view: [`CerebrumNexus::dream_recordings_view()`](cerebrum_nexus/src/lib.rs:873)
- Dream recordings command: [`CerebrumNexus::dream_recordings_command()`](cerebrum_nexus/src/lib.rs:898)

### Multi-modal perception (URLs)

- Handler: [`CerebrumNexus::perceive_command()`](cerebrum_nexus/src/lib.rs:948)

Supported commands:

- `show image <url>`
- `show audio <url>`
- `show video <url>`
- `text <anything>`

### Context engineering panels

- Context view: [`CerebrumNexus::context_engineering_view()`](cerebrum_nexus/src/lib.rs:1718)
- Emotional decay curves: [`CerebrumNexus::decay_curves_view()`](cerebrum_nexus/src/lib.rs:1783)

### Dream cycle (memory reinforcement)

- Handler: [`CerebrumNexus::dream_cycle_now()`](cerebrum_nexus/src/lib.rs:1880)

### Learning pipeline

- Status JSON: [`CerebrumNexus::learning_status()`](cerebrum_nexus/src/lib.rs:1949)
- Health checks JSON: [`CerebrumNexus::learning_health_checks()`](cerebrum_nexus/src/lib.rs:1992)
- Trigger analysis: [`CerebrumNexus::trigger_learning_analysis()`](cerebrum_nexus/src/lib.rs:1963)

### Email ORCH (optional)

- Router: [`CerebrumNexus::handle_email_command()`](cerebrum_nexus/src/lib.rs:1493)

---

## 4) Frontend “Plug-and-Play” UI Modules (Suggested)

If you build a desktop or web UI, you can treat each of these as a self-contained page/tab:

1. **Chat** → `speak_eq`
2. **Skills** → `skills list/run/prefs`
3. **Dreams** → `lucid`, `dream`, `heal`, `list dreams`
4. **Perception** → `show image/audio/video`
5. **Recording** → `record journal` (TUI now; can be generalized)
6. **Approvals** → `approve list` + selection
7. **Context/Memory Debug** → context view + decay curves + dream cycle
8. **Learning Pipeline** → health/status JSON

---

## 5) Machine-Readable Registry

See the JSON registry file for a structured list of commands and suggested payload shapes:

- [`docs/frontend_command_registry.json`](docs/frontend_command_registry.json:1)

