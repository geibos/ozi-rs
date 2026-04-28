## 2026-04-27 Task: 1 capture/classify

- Evidence files for downstream tasks: `.sisyphus/evidence/task-1-run-release-raw.log` and `.sisyphus/evidence/task-1-warning-inventory.md`.
- Required warning-like scanner found 10 regex matches for `warning|warn|deprecated|security|csp|bundle|error` case-insensitively; all 10 are represented exactly once in the inventory.
- Only frontend/Vite/Svelte warnings were actionable in the captured release build/startup output. Task 2 owns 11 actionable rows total: 3 scanner-matched frontend rows plus 8 supplemental Svelte diagnostics that are warning output but do not contain the scanner terms on the primary message line.
- Tasks 3/4/5 have no captured actionable warning rows: no Tauri config/platform warnings, no Rust compiler/dependency warnings, and no release startup warning/error-like rows were captured. Startup emitted only `INFO` lines from `ozi_rs_lib::application`.
- Frontend locations for Task 2: `src/app.css:90` (`scrollbar-color: var(--ctp-surface2) transparent;` causing esbuild CSS minify warnings), `src/components/TracksPanel.svelte:199` (`autofocus`), `src/components/Console.svelte:4` (`scrollEl` should use `$state(...)`), `src/components/MapView.svelte:830/840/845` (unused `.track-point-marker` selectors), `src/components/WaypointsPanel.svelte:93` (`autofocus`), and `src/components/SymbolPicker.svelte:77` (click handler a11y diagnostics).
- Vite chunk-size warning context for Task 2: generated `App-*.js` chunk was around 841.95 kB after minification (`App-BwVPn97T.js` in this capture), above Vite/Rollup's 500 kB advisory threshold.
- Seven scanner matches are exact no-fix rows: dirty-file/status or build artifact/command strings such as `BundleLoaderView`, `--no-bundle`, and `task-4-error-variants.txt`, not actual warning output.

## 2026-04-27 Task: 2 frontend build warnings

- Svelte 5 warning fixes stayed local to captured rows: `Console.svelte` uses `$state` for the bound `scrollEl`; track/waypoint rename inputs use a tiny `use:focusOnMount` action instead of `autofocus`; `SymbolPicker.svelte` no longer needs a click handler on the popover wrapper.
- `MapView.svelte` track point marker styles are still used by DOM created with `document.createElement`; wrapping those selectors in `:global(...)` preserves MapLibre marker styling while satisfying Svelte's component CSS analysis.
- The Vite chunk warning was app-structure-related: splitting `maplibre-gl` with targeted `manualChunks` reduced the app chunk to 36.78 kB. The remaining 802.88 kB `maplibre` vendor chunk is intentional core map-engine payload, so `chunkSizeWarningLimit` was set to 850 with rationale in `.sisyphus/evidence/task-2-chunk-rationale.md`.

## 2026-04-27 Task: 5 startup no-op

- Task 5 made no runtime/logging/source changes because Task 1 assigned no startup warning/error-like scanner matches to it; bounded startup emitted only two `INFO` diagnostics from `ozi_rs_lib::application`.
- `pgrep -fl '(^|/)ozi-rs$' || true` produced no output during Task 5 verification, so no `ozi-rs` process remained.
- Keep final bounded startup verification in Task 6 even though Task 5 is a documented no-op, so release-clean evidence is refreshed after all warning fixes.

## 2026-04-27 Task: 3 Tauri no-op

- Task 3 made no Tauri config/source changes because Task 1 assigned zero captured warning rows to Tauri config/platform. Task 6 should rely on `.sisyphus/evidence/task-3-tauri-no-warnings.md` plus final bounded release output rather than expecting a code/config diff for this slice.

## 2026-04-27 Task: 6 release warning gate file creation

- Added `scripts/check-release-warnings.sh` and the `just check-release-warnings` recipe for the final release warning hygiene gate; heavy release build/startup verification is intentionally left for Atlas.

## 2026-04-27 Task: 6 verification fix

- Fixed the final release build mismatch by aligning `@tauri-apps/plugin-dialog` to `^2.7.0`, matching the Cargo-resolved `tauri-plugin-dialog v2.7.0`; `package-lock.json` now resolves the NPM plugin to `2.7.0`.
- Updated `scripts/check-release-warnings.sh` so `.sisyphus/evidence/task-6-final-run-release.log` records build capture boundaries plus bounded startup capture and termination status without broadening the warning false-positive allowlist.
- `just check-release-warnings` passed after the dependency alignment; the first cold run still surfaced Cargo compile progress false positives for package names containing scanner terms, then the warm verified run produced a clean gate log.

## 2026-04-27 Task: 6 cold-build scanner hardening

- Hardened `scripts/check-release-warnings.sh` with a narrow Cargo progress filter that only removes scanner matches shaped like `Compiling|Checking|Building <crate> v<version>`; this classifies normal dependency names such as `thiserror` and `security-framework` without hiding compiler diagnostics, Tauri failures, startup output, or arbitrary warning/error/security lines.

## 2026-04-27 Task: 6 F1 evidence follow-up

- Added durable F1 verification evidence: `.sisyphus/evidence/task-6-warning-gate-negative.log` embeds the synthetic warning-line excerpt and records nonzero `exit status: 1`; `.sisyphus/evidence/task-6-clippy.log` and `.sisyphus/evidence/task-6-test.log` record final passing `just clippy` and `just test` runs with `exit status: 0`.
- Re-ran `just check-release-warnings` after synthetic mode so `.sisyphus/evidence/task-6-final-warning-gate.log` is restored to clean passing evidence.
