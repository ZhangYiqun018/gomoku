# Gomoku (Tauri Desktop)

A 15×15 Gomoku desktop app with human vs AI play, LLM opponents, Elo ladder, and self‑play calibration. Built with Tauri (Rust) + React (Vite).

## Features
- Human vs AI (12 calibrated heuristic levels)
- LLM opponents with per‑profile config (base URL, model, sampling)
- Elo ladder with W‑D‑L and win rate tracking
- Self‑play calibration (AI vs AI, optional LLM participation)
- Save/Load games in JSON
- Multi‑user profiles (separate ratings + data)

## Tech Stack
- **Frontend:** React + TypeScript + Vite
- **Backend:** Rust (Tauri)
- **LLM bridge:** Node sidecar (`scripts/llm_proxy.mjs`) using OpenAI SDK

## Project Structure
- `src/` — React UI
- `src-tauri/src/` — Rust backend
  - `ai.rs` — heuristic AI + search
  - `engine.rs` — game state
  - `rating.rs` — Elo + self‑play
  - `llm.rs` — LLM move selection
  - `users.rs` — user profiles + settings
- `scripts/llm_proxy.mjs` — Node bridge for LLM calls
- `data/` — runtime user data (ignored by git)

## Prerequisites
- **Rust** stable (via rustup)
- **Bun** (used by Tauri build hooks)
- **Node.js** 22+ (Vite requires 20.19+; 22 recommended)

On macOS, install Xcode Command Line Tools if needed:
```
xcode-select --install
```

## Development
Install deps:
```
bun install
```

Run desktop dev:
```
bun run tauri:dev
```

Run web dev (UI only):
```
bun run dev
```

## Build
```
bun run tauri:build
```

## LLM Notes
LLM moves are generated via a Node sidecar that calls the OpenAI SDK. Ensure `node` is available on PATH at runtime if you use LLM profiles.

## Releases (GitHub Actions)
Tag a release to build Windows + macOS and publish to GitHub Releases:
```
git tag v0.1.0

git push origin v0.1.0
```

## Data Location
Per‑user data is stored in `data/users/<id>/` and includes ratings and settings. This folder is intentionally git‑ignored.
