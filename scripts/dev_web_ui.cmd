@echo off
REM Starts backend + frontend dev server in separate terminals.
REM Requires: Rust toolchain + Node.js.

start "phoenix-web" cmd /k cargo run --bin phoenix-web
start "phoenix-frontend" cmd /k cd frontend ^&^& npm install ^&^& npm run dev

