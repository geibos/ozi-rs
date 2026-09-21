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

# Build release (no bundle) and run the resulting binary.
#
# Split into independent layers so each cache (Vite output hash, Cargo
# incremental) is consulted on its own terms — bypasses `tauri build`'s
# unconditional `beforeBuildCommand` re-run. See
# `openspec/specs/build-tooling/spec.md` and `scripts/dev-build-frontend.sh`.
# The canonical bundled path (`just release`, `just build`) still uses
# `npm run tauri build` and is unaffected.
# `--features custom-protocol` is load-bearing: Tauri 2 embeds production
# assets only under that feature (the tauri CLI enables it implicitly).
# Without it the context is "dev" and, with `devUrl` set, embeds an EMPTY
# asset set — the app launches into a blank white window.
run-release:
    @./scripts/dev-build-frontend.sh
    cargo build --release --manifest-path src-tauri/Cargo.toml --features custom-protocol
    ./target/release/ozi-rs

# E2E smoke gate: drives the real bundled app through the core workflow
# (launch → Tracks tab → draw 3 points → cancel) via Appium Mac2. This is the
# pre-merge gate for any change touching the app: run it before merging.
# Requires a `just build` artifact and a running Appium server
# (`appium --address 127.0.0.1 --port 4723`, driver: `appium driver install mac2`).
smoke:
    cargo test --manifest-path tools/ozi-rs-mcp/Cargo.toml --test smoke_core_workflow -- --ignored --nocapture

# ── Build ─────────────────────────────────────────────────────────────────────

# Build the full Tauri app (debug)
build:
    npm run tauri build -- --debug
    @just sign-dev

# Sign the debug bundle with the local development identity, when one exists.
# Without a stable identity every rebuild is a new program to macOS, so the
# Documents-access prompt returns on each build and blocks the window from
# opening. Create the identity once with ./scripts/setup-dev-signing.sh.
sign-dev:
    #!/usr/bin/env bash
    set -euo pipefail
    app="target/debug/bundle/macos/ozi-rs.app"
    identity="ozi-rs Local Dev"
    if [ ! -d "$app" ]; then exit 0; fi
    if ! security find-identity -v -p codesigning | grep -qF "$identity"; then
      echo "note: no '$identity' signing identity; leaving the ad-hoc signature."
      echo "      run ./scripts/setup-dev-signing.sh to stop the repeated permission prompts."
      exit 0
    fi
    codesign --force --deep --options runtime --sign "$identity" "$app"
    codesign --verify --deep --strict "$app"
    echo "signed $app with '$identity'"

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
check-rust:
    cargo check --manifest-path src-tauri/Cargo.toml

# Type-check Svelte components with svelte-check
check-ui:
    npm exec -- svelte-kit sync
    npm exec -- svelte-check --tsconfig ./tsconfig.json

# Run both Rust and Svelte type-checks
check: check-rust check-ui

# Run cargo clippy on the Tauri backend
clippy:
    cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

# Lint frontend with ESLint (Svelte + TS flat config)
lint:
    npm exec -- eslint .

# Format the workspace with Prettier
fmt:
    npm exec -- prettier --write .

# Check Rust formatting without writing (the same gate CI runs)
fmt-check:
    cargo fmt --all --manifest-path src-tauri/Cargo.toml -- --check

# Run all CI-equivalent gates locally (fmt + clippy + check + lint + test)
ci: fmt-check clippy check lint test

# Run local release warning hygiene gate
check-release-warnings:
    ./scripts/check-release-warnings.sh

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

# Serve the real frontend in a browser, answering IPC from the fixtures.
#
# No Rust build, no Appium, no window to catch in the right state: open
# http://localhost:5273 and look. The data is what the backend sends, because
# the fixtures come from the same mappers the commands use. It proves how a
# screen looks and behaves, not that the Rust side works — `just smoke` does
# that.
stand:
    npm exec -- vite --config vite.stand.config.ts

# Regenerate the frontend test fixtures from the Rust core.
#
# The generator is a test, and it fails by design when it rewrote a stale
# fixture — that is what makes `just ci` catch a DTO change the frontend
# mocks have not seen. Here the failure is the expected outcome, so the exit
# code is ignored and the diff is what to look at.
fixtures:
    -cargo test --manifest-path src-tauri/Cargo.toml fixtures_are_up_to_date
    @echo "fixtures written to src/test/fixtures — review the diff" 

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
