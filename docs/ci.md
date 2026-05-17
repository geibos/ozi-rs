# Continuous integration

This document describes the GitHub Actions setup that gates merges to `main` and produces release artifacts. It is the source of truth for what CI does, how to update toolchains, and how to publish a release.

## Workflows

Two workflows live under `.github/workflows/`:

| File | Triggers | Purpose |
|------|----------|---------|
| `ci.yml` | `pull_request` → `main`, `push` → `main`, `workflow_dispatch` | All correctness gates: lint, type-check, tests, OpenSpec validation, security audit, cross-platform smoke build |
| `release.yml` | `push` of tag matching `v*` | Build signed-but-not-notarized Tauri bundles for macOS (universal) + Windows and attach them to a **draft** GitHub Release |

Both workflows pin the Rust toolchain via `rust-toolchain.toml` and Node.js via `.nvmrc`, so local and CI environments resolve to the same versions.

## `ci.yml` jobs

All four jobs run in parallel. Concurrency is `ci-${{ github.workflow }}-${{ github.ref }}` with `cancel-in-progress: true` — pushing a new commit to a PR cancels the older run.

### 1. `lint-and-test` (ubuntu-latest)

Runs the same `just` recipes you use locally, in this order (fail-fast):

1. `cargo fmt --all --manifest-path src-tauri/Cargo.toml -- --check`
2. `just clippy` — `cargo clippy -- -D warnings`
3. `just check-rust` — `cargo check`
4. `just test-rust` — backend unit tests
5. `just lint` — ESLint flat config
6. `just check-ui` — `svelte-kit sync` + `svelte-check`
7. `just test-ui` — Vitest

Tauri Linux system deps (`libwebkit2gtk-4.1`, `libsoup-3.0`, `librsvg2`, `libayatana-appindicator3`, `build-essential`, …) are installed via `apt-get`.

### 2. `openspec-validate` (ubuntu-latest)

Installs the `@fission-ai/openspec` CLI from npm and runs `openspec validate <change> --strict` for every directory under `openspec/changes/` (except `archive/`). A single failing change fails the job.

### 3. `security-audit` (ubuntu-latest)

- **Rust:** `cargo audit --file src-tauri/Cargo.lock`. Vulnerabilities (severity advisories) fail the job; informational warnings (unmaintained, unsound) do **not** — they would otherwise produce dozens of unfixable noise from transitive Tauri / gtk-rs dependencies. To ignore a specific advisory, see [Adding an ignored advisory](#adding-an-ignored-advisory).
- **Frontend:** `npm audit --omit=dev --audit-level=high`. Only production dependencies are considered, only high / critical advisories block. Dev dependencies (eslint, vitest, …) are not shipped to users.

### 4. `tauri-build-smoke` (ubuntu, macos, windows)

Matrix build of `npm run tauri build -- --no-bundle` on all three runner OSes with `fail-fast: false`. This catches platform-specific linkage / compile errors (WebView2 on Windows, WKWebView on macOS, webkit2gtk on Linux) without paying the cost of a full bundle on every PR.

## `release.yml`

Triggered by `git push origin v<x.y.z>`. Matrix:

| Platform | Args | Artifacts |
|----------|------|-----------|
| `macos-latest` | `--target universal-apple-darwin` | `.dmg`, `.app.tar.gz` (Intel + ARM in one bundle) |
| `windows-latest` | _(none)_ | `.msi`, `.exe` |

Uses [`tauri-apps/tauri-action@v0`](https://github.com/tauri-apps/tauri-action) with `releaseDraft: true` — a human reviewer must publish the release after verifying artifacts. The workflow uses the built-in `GITHUB_TOKEN`; no extra secrets are required.

**Out of scope of this workflow** (separate change required when needed):

- macOS code signing + notarization (requires `APPLE_*` secrets).
- Windows code signing (requires certificate + `SignTool`).
- Linux bundles (`.AppImage`, `.deb`, `.rpm`).

## Triggers matrix

| Event | `ci.yml` | `release.yml` |
|-------|:--------:|:-------------:|
| Pull request → `main` | ✅ | — |
| Push → `main` | ✅ | — |
| Manual (`workflow_dispatch`) | ✅ | — |
| Push tag `v*` | — | ✅ |
| Push tag not matching `v*` | — | — |

## Branch protection rules (manual setup)

After the workflows land on `main`, configure branch protection in **Settings → Branches → Branch protection rules → `main`**:

1. **Require status checks to pass before merging.** Mark these as required:
   - `Lint & test (Linux)`
   - `OpenSpec validate`
   - `Security audit (cargo + npm)`
   - `Tauri smoke build (ubuntu-latest)`
   - `Tauri smoke build (macos-latest)`
   - `Tauri smoke build (windows-latest)`
2. **Require branches to be up to date** before merging (catches integration regressions).
3. **Require linear history** (matches the OpenSpec workflow expectation that archives are atomic commits).
4. **Do not allow bypassing** the above for administrators unless absolutely needed.

These are configured in the GitHub UI, not in this repository — they cannot be checked in.

## Local equivalents

You can reproduce CI gates locally with `just`:

```bash
just clippy        # cargo clippy -- -D warnings
just check         # cargo check + svelte-check
just lint          # ESLint
just test          # cargo test + Vitest
just ci            # all of the above (shortcut)
```

`openspec validate <change> --strict` runs the same gate as the OpenSpec job. `cargo audit` and `npm audit --omit=dev --audit-level=high` reproduce the security job — install `cargo-audit` once with `cargo install cargo-audit --locked`.

## Updating the pinned toolchain

The Rust channel is pinned in `rust-toolchain.toml` (currently `1.95.0`). Node is pinned in `.nvmrc` (currently `22`, the active LTS line "Jod").

**Procedure for bumping Rust:**

1. Edit `rust-toolchain.toml` → set new `channel`.
2. Run `rustup show` locally to materialize the new toolchain.
3. Run `just clippy` — fix any new lints (clippy upgrades sometimes add new diagnostics).
4. Run `just check`, `just test`. Open PR.
5. After merge, watch the first `main` run to confirm cached / cold builds both pass.

**Procedure for bumping Node:**

1. Edit `.nvmrc` → set new major (e.g. `24` when it goes LTS).
2. Run `nvm use` locally, then `npm ci` to refresh `node_modules` on the new runtime.
3. Run `just lint`, `just check-ui`, `just test-ui`. Adjust `@types/node` in `package.json` if needed.
4. Open PR.

## Adding an ignored advisory

If `cargo audit` fails on a transitive vulnerability with no available fix:

1. Open `.cargo/audit.toml`.
2. Add the advisory ID to the `ignore` array.
3. Include a comment that explains:
   - **Why** the advisory cannot be addressed (no upstream fix; dependency pinned by Tauri; etc.).
   - **When** the entry should be reconsidered (next Tauri / reqwest / rustls bump, specific date, or upstream issue URL).
4. Re-run `cargo audit --file src-tauri/Cargo.lock` locally — exit code must be 0.
5. At every dependency bump, revisit the file and remove entries whose upstream is now fixed.

Example block at the time of writing:

```toml
[advisories]
ignore = [
    # rustls-webpki 0.103.10 — three advisories from 2026-04;
    # tauri → reqwest → quinn → rustls-webpki, no direct fix path until
    # Tauri ships a release with rustls 0.23.next. Recheck on every Tauri bump.
    "RUSTSEC-2026-0098",
    "RUSTSEC-2026-0099",
    "RUSTSEC-2026-0104",
]
```

For npm advisories that need to be ignored, use `npm` overrides in `package.json` (only when no fix is available upstream) or, if absolutely necessary, scope them out via `.npmrc`. Always leave a comment explaining the decision in the PR body.

## Release procedure

1. Ensure `main` is green on `ci.yml`.
2. Bump version in `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml` (and `package.json` if relevant); commit + merge through a normal PR.
3. Tag locally: `git tag v<x.y.z>` and `git push origin v<x.y.z>`.
4. The `release.yml` workflow starts automatically. It builds on macOS-universal and Windows and creates a **draft** release named `v<x.y.z>`.
5. Open the draft release in the GitHub UI, verify the attached artifacts (sizes, file types), edit the release notes, then click **Publish release**.
6. If a release build fails on either runner, fix the issue on `main` and re-tag (delete the failed tag first if it was created): `git tag -d v<x.y.z>; git push origin :refs/tags/v<x.y.z>`.

## Expected timings

On cached runs, target wall-clock:

| Job | Typical | Worst case |
|-----|---------|-----------|
| `lint-and-test` (Linux) | 6–10 min | 15 min (cold cache) |
| `openspec-validate` | 30 s | 1 min |
| `security-audit` | 1–2 min | 4 min |
| `tauri-build-smoke` (Linux) | 4–7 min | 12 min |
| `tauri-build-smoke` (macOS) | 6–10 min | 15 min |
| `tauri-build-smoke` (Windows) | 8–12 min | 20 min |
| `release.yml` (per platform, full bundle) | 12–18 min | 35 min |

Jobs run in parallel, so total PR wall-clock is dominated by the slowest job (usually `tauri-build-smoke (windows-latest)`).
