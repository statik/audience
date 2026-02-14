# Audience - PTZ Camera Controller

## Before Committing

Always run formatting and lint checks before committing:

```
just check
```

Or individually:

```
just check-frontend   # ESLint + TypeScript typecheck
just check-rust       # cargo fmt --check + clippy
```

To auto-fix formatting: `just fmt`

## Running Tests

```
just test
```

## Project Structure

- `src/` — React/TypeScript frontend (Vite, Tailwind CSS, Zustand)
- `src-tauri/` — Rust backend (Tauri v2)

## Key Commands

- `npm run dev` — Start Vite dev server
- `npm run build` — Build frontend
- `cargo build --manifest-path src-tauri/Cargo.toml` — Build Tauri backend
- `npx vitest run` — Frontend tests
- `cargo test --manifest-path src-tauri/Cargo.toml` — Rust tests
