# ozi-rs task runner
# Usage: just <recipe>
# Install just: cargo install just

# Show available recipes
default:
    @just --list

# ── Development ───────────────────────────────────────────────────────────────

# Start Tauri dev (Vite HMR + Rust watch)
dev:
    npm run tauri dev

# Start only the Vite frontend dev server (no Rust backend)
dev-ui:
    npm run dev

# Watch and check Rust on changes (no Tauri, fast feedback)
watch:
    cargo watch --manifest-path src-tauri/Cargo.toml -x check

# Watch and test Rust on changes
watch-test:
    cargo watch --manifest-path src-tauri/Cargo.toml -x test

# ── Run ───────────────────────────────────────────────────────────────────────

# Build and run the app in dev mode (Vite HMR + Rust)
run:
    npm run tauri dev

# Build release (no bundle) and run the resulting binary
run-release:
    npm run tauri build -- --no-bundle
    ./src-tauri/target/release/ozi-rs

# ── Build ─────────────────────────────────────────────────────────────────────

# Build the full Tauri app (debug)
build:
    npm run tauri build -- --debug

# Build the full Tauri app (release)
release:
    npm run tauri build

# Build only the Rust backend (debug)
build-rust:
    cargo build --manifest-path src-tauri/Cargo.toml

# Build only the Rust backend (release)
build-rust-release:
    cargo build --manifest-path src-tauri/Cargo.toml --release

# Build only the Vite frontend
build-ui:
    npm run build

# ── Check & lint ──────────────────────────────────────────────────────────────

# Run cargo check on the Tauri backend
check:
    cargo check --manifest-path src-tauri/Cargo.toml

# Run cargo clippy on the Tauri backend
clippy:
    cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

# ── Test ──────────────────────────────────────────────────────────────────────

# Run all tests (Rust + frontend)
test:
    cargo test --manifest-path src-tauri/Cargo.toml
    npm test

# Run only Rust tests
test-rust:
    cargo test --manifest-path src-tauri/Cargo.toml

# Run only frontend tests (Vitest)
test-ui:
    npm test

# Run a specific Rust test by name filter
test-filter filter:
    cargo test --manifest-path src-tauri/Cargo.toml {{ filter }}

# ── Tooling ───────────────────────────────────────────────────────────────────

# Install npm dependencies
npm-install:
    npm install

# Update npm dependencies
npm-update:
    npm update

# Update Rust dependencies
update:
    cargo update --manifest-path src-tauri/Cargo.toml

# ── Cleanup ───────────────────────────────────────────────────────────────────

# Remove Rust build artifacts
clean:
    cargo clean --manifest-path src-tauri/Cargo.toml

# Remove Rust build artifacts and Vite dist
clean-all:
    cargo clean --manifest-path src-tauri/Cargo.toml
    rm -rf dist node_modules

# ── Misc ──────────────────────────────────────────────────────────────────────

# Show Rust toolchain info
toolchain:
    rustc --version
    cargo --version
    rustup show

# Show git log for this branch
log:
    git log --oneline -20
