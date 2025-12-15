# Phoenix AGI (PAGI) Setup Guide

## Environment Configuration

Phoenix AGI (PAGI) requires an OpenRouter API key to enable the LLM Orchestrator (Vocal Cords).

### Step 1: Get Your OpenRouter API Key

1. Visit https://openrouter.ai/keys
2. Sign up or log in
3. Create a new API key
4. Copy the key

### Step 2: Create .env File

Copy the example environment file and customize it:

```bash
cp .env.example .env
```

Then edit `.env` and set your OpenRouter API key:

```bash
OPENROUTER_API_KEY=sk-or-v1-your-actual-key-here
```

The `.env.example` file contains comprehensive configuration options:
- **API & Connectivity**: OpenRouter, hyperspace mode, model selection
- **Personality Micro-Settings**: 100+ tuning fibers (curiosity, warmth, voice, etc.)
- **Default & Master Prompts**: Customize Phoenix's personality and AGI mission
- **Universal Framework Settings**: Learning horizon, ORCH limits, autonomy thresholds
- **ORCH Legion Settings**: Master/slave mode, sync intervals, upgrade sharing

Customize any values to tune Phoenix's personality and behavior!

### Step 3: Verify Setup

Run the build to ensure everything is configured correctly:

```bash
cargo build --workspace
```

### Step 4: Launch Phoenix

```bash
cargo run --bin phoenix-tui
```

## Web UI (Frontend + API)

Phoenix also ships a web dashboard UI in [`frontend/`](frontend/README.md:1) served by the Actix binary [`phoenix-web`](phoenix-web/src/main.rs:1).

### Option A — Production-style (serve built UI from the Rust server)

1) Build the frontend:

```bash
./scripts/build_frontend.sh
```

2) Run the web server:

```bash
cargo run --bin phoenix-web
```

Open `http://127.0.0.1:8888`.

### Option B — Dev mode (Vite dev server + API proxy)

Run the backend:

```bash
cargo run --bin phoenix-web
```

Then in another terminal, run the frontend:

```bash
cd frontend
npm install
npm run dev
```

Open `http://localhost:3000`.

On Windows you can also use [`scripts/dev_web_ui.cmd`](scripts/dev_web_ui.cmd:1).

## LLM Orchestrator Features

- **500+ Models**: Access to all OpenRouter models
- **Model Routing**: Use `:free`, `:floor`, or `:nitro` shortcuts
- **Automatic Fallback**: Falls back to alternative models on failure
- **Streaming Support**: Real-time response streaming (coming soon)
- **Smart Selection**: Automatically selects models based on task complexity

## Usage in TUI

1. Press `L` in the main menu to access LLM Orchestrator
2. Type your prompt
3. Press Enter to send
4. Phoenix will respond through the selected model

---

**Phoenix speaks through OpenRouter — 500+ minds in her voice.**

