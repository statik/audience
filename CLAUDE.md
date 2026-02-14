# Audience - PTZ Camera Controller

## Commit Messages

Always use [Conventional Commits](https://www.conventionalcommits.org/) format. This is enforced by commitlint in both a local git hook and CI.

Format: `<type>(<optional scope>): <description>`

Types:
- `feat:` — A new feature (triggers a minor release)
- `fix:` — A bug fix (triggers a patch release)
- `docs:` — Documentation only
- `style:` — Formatting, missing semicolons, etc. (no code change)
- `refactor:` — Code change that neither fixes a bug nor adds a feature
- `perf:` — Performance improvement
- `test:` — Adding or updating tests
- `chore:` — Build process, tooling, dependencies
- `ci:` — CI/CD configuration

Breaking changes: Add `BREAKING CHANGE:` in the commit body or `!` after the type (e.g. `feat!: remove legacy API`) to trigger a major release.

Examples:
```
feat: add preset recall buttons to camera panel
fix(visca): correct pan speed calculation for slow movements
docs: update release process documentation
chore: upgrade tauri to v2.3
feat!: drop support for VISCA over serial
```

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
