# Check all formatting and linting (frontend + Rust)
check: check-frontend check-rust

# Run all tests
test: test-frontend test-rust

# Check frontend formatting and linting
check-frontend:
    npx eslint src/
    npx tsc --noEmit

# Check Rust formatting and linting
check-rust:
    cargo fmt --check --manifest-path src-tauri/Cargo.toml
    cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

# Run frontend tests
test-frontend:
    npx vitest run

# Run Rust tests
test-rust:
    cargo test --manifest-path src-tauri/Cargo.toml

# Run local dev version of the app
dev:
    npx tauri dev

# Auto-fix formatting
fmt:
    cargo fmt --manifest-path src-tauri/Cargo.toml
    npx eslint src/ --fix
