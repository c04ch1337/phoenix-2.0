# Phoenix UI Frontend (Vite) + Backend (Actix)

This repo contains a React/Vite UI in [`frontend/`](frontend/README.md:1) and a minimal HTTP API backend in [`phoenix-web`](phoenix-web/src/main.rs:1).

## Architecture

- **Frontend**: Vite dev server at `http://localhost:3000`
- **Backend**: Actix server at `http://127.0.0.1:8888`
- **Contract**: command router
  - UI calls `POST /api/command` with `{ "command": "..." }`
  - Backend returns a JSON string like `{ "type": "chat.reply", "message": "..." }`

## Endpoints

- `GET /health` → `{ "status": "ok" }`
- `GET /api/status` → `{ "status": "online|offline", "version": "…", "archetype": "…" }`
- `GET /api/name` → `{ "name": "Phoenix" }`
- `POST /api/command` → **JSON string** response (legacy-friendly)
- `POST /api/speak` → same as command (accepts `{ user_input, dad_emotion_hint?, mode? }`)

## Dev mode

1) Start backend:

```bash
cargo run --bin phoenix-web
```

2) Start frontend:

```bash
cd frontend
npm install
npm run dev
```

Vite proxies `/api/*` to the backend (see [`vite.config.ts`](frontend/vite.config.ts:1)).

## Production mode

1) Build the frontend:

```bash
cd frontend
npm run build
```

2) Run the backend:

```bash
cargo run --release --bin phoenix-web
```

If `frontend/dist` exists, the backend will serve the static UI from `/`.

