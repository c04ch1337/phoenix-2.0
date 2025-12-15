#!/usr/bin/env bash
set -euo pipefail

echo "Building Phoenix frontend (Vite)..."
cd frontend
npm install
npm run build

