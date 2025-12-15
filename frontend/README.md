<div align="center">
<img width="1200" height="475" alt="GHBanner" src="https://github.com/user-attachments/assets/0aa67016-6eaf-458a-adb2-6e31a0763ed6" />
</div>

# Run and deploy your AI Studio app

This contains everything you need to run your app locally.

View your app in AI Studio: https://ai.studio/apps/drive/1woXeutm4o_0KtddP_V5S9qUpagZoLnVX

## Run Locally

**Prerequisites:** Node.js, Rust toolchain (for the Phoenix backend)


### 1) Start the Phoenix UI backend (HTTP API)

From the repo root:

```bash
cargo run --bin phoenix-web
```

The backend listens on `http://127.0.0.1:8888`.

### 2) Start the Vite frontend (dev server)

From `frontend/`:

```bash
npm install
npm run dev
```

Open `http://localhost:3000`.

The dev server proxies `/api/*` to the backend automatically (see [`frontend/vite.config.ts`](frontend/vite.config.ts:1)).

### 3) Enable real Phoenix chat (optional)

Set `OPENROUTER_API_KEY` in the repo root `.env` file (see [`.env.example`](../.env.example:1)).

Without an OpenRouter key, the UI backend will report `offline` and return an error for LLM-backed commands.
