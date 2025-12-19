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

### Troubleshooting `.env` not applying

If Phoenix behaves like configuration is being ignored (name/prompt defaults, “missing OPENROUTER_API_KEY”, etc.), the two most common causes are:

1) **The process is started from a different working directory** (so `.env` isn’t found).
   - Fix: run from the repo root, or set `PHOENIX_DOTENV_PATH`.

2) **Optional variables are present but blank** (e.g., `PHOENIX_CUSTOM_NAME=`), which historically could override fallbacks.
   - Fix: update to the latest code (empty values are now treated as “unset”), or delete the blank lines in `.env`.

Debug options:

```env
# Print a safe startup snapshot (does not print secret values)
PHOENIX_ENV_DEBUG=true

# Pin the dotenv file explicitly (helpful for Windows shortcuts/services)
PHOENIX_DOTENV_PATH=C:/Users/you/path/to/phoenix-2.0/.env
```

The [`/.env.example`](.env.example:1) file documents additional optional configuration.

#### GitHub-first evolution / creation approvals (optional)

Some “creation” workflows (agent/tool scaffolding and evolution) are intentionally GitHub-first:

branch → PR → CI → human approval → merge

If you want those flows enabled, you’ll need to configure GitHub env vars in your `.env`. See the
GitHub section in [`/.env.example`](.env.example:1).

The web server exposes a sanitized view of this config at:

- `GET /api/evolution/status` (served by [`phoenix-web`](phoenix-web/src/main.rs:1))

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

## Google Ecosystem (Gmail / Drive / Calendar / Docs / Sheets)

Phoenix’s web UI ships with a “Google Ecosystem” dashboard panel. Real connectivity is implemented server-side in [`phoenix-web`](phoenix-web/src/main.rs:1) and requires OAuth2 configuration.

### 1) Create OAuth credentials

In Google Cloud Console:

1. Create/select a project
2. Enable APIs:
   - Gmail API
   - Google Drive API
   - Google Calendar API
   - Google Docs API
   - Google Sheets API
3. Configure OAuth consent screen (Internal/External depending on your org)
4. Create **OAuth Client ID** → **Web application**
5. Add an **Authorized redirect URI** that matches your local server:

   - `http://127.0.0.1:8888/api/google/oauth2/callback`

### 2) Set env vars

Add these to your `.env` (see [`/.env.example`](.env.example:1)):

```env
GOOGLE_OAUTH_CLIENT_ID=...
GOOGLE_OAUTH_CLIENT_SECRET=...
GOOGLE_OAUTH_REDIRECT_URL=http://127.0.0.1:8888/api/google/oauth2/callback

# Optional: space-separated scopes (bigger scopes == more power and more risk)
GOOGLE_OAUTH_SCOPES=openid email profile https://www.googleapis.com/auth/gmail.readonly https://www.googleapis.com/auth/gmail.send https://www.googleapis.com/auth/drive.metadata.readonly https://www.googleapis.com/auth/calendar.readonly https://www.googleapis.com/auth/documents https://www.googleapis.com/auth/spreadsheets
```

Tokens are stored in the OS keyring (Windows Credential Manager on Windows).

### 3) Connect from the UI

1. Run the backend: `cargo run --bin phoenix-web`
2. Open the Web UI
3. Go to **Google Ecosystem** → **Connect Google Account**
4. Complete the consent in the opened browser window
5. The panel will auto-poll and flip to “Connected” once the callback completes

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

