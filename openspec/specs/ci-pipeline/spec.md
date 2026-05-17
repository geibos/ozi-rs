# ci-pipeline Specification

## Purpose
TBD - created by archiving change add-github-ci. Update Purpose after archive.
## Requirements
### Requirement: Continuous integration pipeline on pull requests and main

The repository SHALL provide a GitHub Actions workflow that runs on every pull request targeting `main`, every push to `main`, and on manual `workflow_dispatch` invocation. The workflow MUST execute the following gates and MUST fail the build if any gate fails:

- Rust formatting check (`cargo fmt --all -- --check`).
- Rust linting via `just clippy` (which enforces `cargo clippy -- -D warnings`).
- Rust type-check via `just check-rust`.
- Rust unit tests via `just test-rust`.
- Frontend lint via `just lint`.
- Frontend type-check via `just check-ui` (svelte-check).
- Frontend unit tests via `just test-ui` (Vitest).
- OpenSpec validation: `openspec validate --strict` MUST pass for every active change directory under `openspec/changes/`.
- A cross-platform Tauri smoke build (`npm run tauri build -- --no-bundle`) MUST succeed on `ubuntu-latest`, `macos-latest`, and `windows-latest`.

The workflow SHALL cache Cargo registry / target directories and the npm cache to keep typical cached PR runs under 15 minutes.

#### Scenario: Pull request fails on clippy warning

- **WHEN** a pull request introduces Rust code that produces a clippy warning
- **THEN** the CI workflow MUST fail the `clippy` step and report it as a required check on the pull request

#### Scenario: Pull request fails on frontend type error

- **WHEN** a pull request introduces a TypeScript / Svelte type error
- **THEN** the CI workflow MUST fail the `check-ui` step and the pull request MUST be blocked from merging

#### Scenario: Pull request fails on invalid OpenSpec change

- **WHEN** a pull request includes a change directory under `openspec/changes/` whose `proposal.md`, `tasks.md`, or `specs/**/*.md` fails `openspec validate --strict`
- **THEN** the CI workflow MUST fail the OpenSpec gate

#### Scenario: Push to main runs full gates

- **WHEN** a commit is pushed directly to `main`
- **THEN** the CI workflow MUST run all gates listed above and surface failures via the commit status

#### Scenario: Manual workflow_dispatch

- **WHEN** a maintainer triggers the CI workflow manually via the GitHub Actions UI
- **THEN** the workflow MUST run with the same gates as a push, on the selected branch

#### Scenario: Concurrent runs on the same branch

- **WHEN** a new commit is pushed to a branch with an in-progress CI run for the same ref
- **THEN** the in-progress run for that ref MUST be cancelled before the new run starts

### Requirement: Dependency security audit

The CI workflow SHALL run security audits for both ecosystems on every pull request and push to `main`:

- `cargo audit` against `src-tauri/Cargo.lock`, failing the build on any unignored advisory.
- `npm audit --omit=dev --audit-level=high` against `package-lock.json`, failing the build on any high or critical advisory in production dependencies.

The workflow MUST support an explicit ignore list for advisories without an available fix (Rust: `.cargo/audit.toml`; npm: documented overrides). Ignored advisories MUST include a written justification.

#### Scenario: Cargo audit detects a critical advisory

- **WHEN** a transitive Rust dependency in `src-tauri/Cargo.lock` matches an active RustSec advisory that is not on the ignore list
- **THEN** the `cargo audit` step MUST fail the workflow

#### Scenario: Frontend dev dependency advisory is ignored

- **WHEN** a dev-only npm dependency has a high-severity advisory
- **THEN** `npm audit --omit=dev` MUST NOT fail the workflow, because dev dependencies are not shipped to users

#### Scenario: Ignored advisory is justified

- **WHEN** a Rust advisory is added to `.cargo/audit.toml` ignore list
- **THEN** the entry MUST include a comment explaining why the advisory is ignored and when it can be reconsidered

### Requirement: Release workflow for tagged versions

The repository SHALL provide a GitHub Actions workflow that triggers on pushed tags matching `v*` (semantic version prefix). The workflow MUST build Tauri release bundles for the platforms agreed for distribution and attach them to a draft GitHub Release for that tag.

Required artifacts:

- macOS universal binary: `.dmg` installer and `.app.tar.gz` bundle (target `universal-apple-darwin`).
- Windows x86_64: `.msi` installer and `.exe` executable.

The release SHALL be created in draft state so a human reviewer can verify artifacts before publishing. The workflow MUST use the built-in `GITHUB_TOKEN` and MUST NOT require additional secrets in this scope (code signing is explicitly out of scope and SHALL be added in a separate change).

#### Scenario: Tag push produces draft release

- **WHEN** a maintainer pushes a tag matching `v*` (e.g. `v0.2.0`)
- **THEN** the release workflow MUST build artifacts on `macos-latest` and `windows-latest` and attach them to a draft GitHub Release named after the tag

#### Scenario: Release build fails on either platform

- **WHEN** the Tauri bundle build fails on either `macos-latest` or `windows-latest`
- **THEN** the workflow MUST fail and the draft release MUST NOT be created (or MUST be marked failed)

#### Scenario: Non-tag pushes do not run release workflow

- **WHEN** a push happens on any branch or a tag not matching `v*`
- **THEN** the release workflow MUST NOT run

### Requirement: Reproducible toolchain pinning

The repository SHALL pin the Rust toolchain via `rust-toolchain.toml` (channel, version, and required components `clippy` and `rustfmt`) and the Node.js version via `.nvmrc`. The CI workflow MUST consume those pins so that local and CI environments resolve to the same toolchain versions.

#### Scenario: CI uses pinned Rust version

- **WHEN** the CI workflow installs the Rust toolchain
- **THEN** it MUST resolve to the version declared in `rust-toolchain.toml`, not to an unbounded `stable`

#### Scenario: CI uses pinned Node version

- **WHEN** the CI workflow sets up Node.js
- **THEN** the Node version MUST match the version declared in `.nvmrc`

#### Scenario: Updating the toolchain pin

- **WHEN** a maintainer updates `rust-toolchain.toml` or `.nvmrc`
- **THEN** the next CI run MUST use the new pin without additional workflow changes

### Requirement: CI documentation

The repository SHALL provide `docs/ci.md` describing the CI gates, required status checks, release process, toolchain update procedure, and how to add an ignored security advisory. `AGENTS.md` and `CLAUDE.md` SHALL link to this document from their workflow sections.

#### Scenario: Contributor reads CI gates

- **WHEN** a new contributor opens `docs/ci.md`
- **THEN** they MUST find the full list of CI gates, the trigger matrix, the toolchain update steps, and the release procedure in that document

