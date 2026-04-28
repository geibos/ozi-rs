# Task 1 `just run-release` Warning Inventory

Source checkbox: `- [ ] 1. Capture and classify `just run-release` warning output`

## Capture

- Raw log: `.sisyphus/evidence/task-1-run-release-raw.log`
- Capture phases present:
  - `===== git status before capture =====`
  - `===== build: npm run tauri build -- --no-bundle =====`
  - `===== build exit: 0 =====`
  - `===== startup: ./src-tauri/target/release/ozi-rs =====`
  - `===== startup termination: TERM after 15 seconds, KILL fallback after 2 seconds =====`
  - `===== startup capture complete =====`
- Startup process cleanup: `pgrep -fl '(^|/)ozi-rs$' || true` produced no output after capture, so no `ozi-rs` process remained.

## Pre-existing dirty-file context

`git status --short` was run before capture. The repository already had unrelated dirty/untracked files before this task, including modified source/docs/notepads and untracked `.sisyphus/` evidence/plan files. This task did not revert or modify those files.

Pre-existing modified files recorded in the raw log: `.sisyphus/notepads/phases-7-10-features/decisions.md`, `.sisyphus/notepads/phases-7-10-features/issues.md`, `.sisyphus/notepads/phases-7-10-features/learnings.md`, `AGENTS.md`, `docs/adr/adr-0005-snapshot-undo-redo.md`, `docs/testing-strategy.md`, `src-tauri/src/commands/tiles.rs`, `src-tauri/src/domain/project.rs`, `src-tauri/src/infrastructure/export/plt.rs`, `src-tauri/src/infrastructure/import/mod.rs`, `src-tauri/src/infrastructure/import/ozi_georeference.rs`, `src-tauri/src/infrastructure/import/plt.rs`, `src/components/SimplifyPanel.svelte`, `src/components/ThemePicker.svelte`, `src/lib/windows.ts`, `src/main.ts`, and `src/views/BundleLoaderView.svelte`.

Pre-existing untracked paths recorded in the raw log included `.opencode/`, `.playwright-mcp/`, `.sisyphus/boulder.json`, `.sisyphus/drafts/`, several prior `.sisyphus/evidence/task-*` files, `.sisyphus/notepads/doc-audit-fixes/`, `.sisyphus/notepads/phases-7-10-features/problems.md`, `.sisyphus/notepads/run-release-warnings/`, `.sisyphus/plans/`, `CLAUDE.md`, `docs/adr/adr-0017-delta-based-undo.md`, `docs/adr/adr-0018-waypoint-symbols-as-strings.md`, `docs/superpowers/`, `src-tauri/.sisyphus/`, and `src/lib/theme.ts`.

## Required scanner command

Case-insensitive warning-like scanner used:

```bash
grep -Ein 'warning|warn|deprecated|security|csp|bundle|error' .sisyphus/evidence/task-1-run-release-raw.log
```

The scanner found 10 matching lines. Each scanner match is represented exactly once in the table below.

## Scanner-match inventory

| Raw log line | exact text | phase | emitting tool/module | classification | suspected root cause | fix owner task | verification command |
|---:|---|---|---|---|---|---|---|
| 18 | ` M src/views/BundleLoaderView.svelte` | git status before capture | Git working-tree status | exact no-fix rationale: dirty-file context only, not emitted release/build warning | Path contains `Bundle`, matching scanner pattern incidentally while recording pre-existing dirty files | No fix: evidence context, preserve dirty-file record | `grep -Ein 'warning\|warn\|deprecated\|security\|csp\|bundle\|error' .sisyphus/evidence/task-1-run-release-raw.log` |
| 33 | `?? .sisyphus/evidence/task-4-error-variants.txt` | git status before capture | Git working-tree status | exact no-fix rationale: dirty-file context only, not emitted release/build warning | Path contains `error`, matching scanner pattern incidentally while recording pre-existing untracked files | No fix: evidence context, preserve dirty-file record | same scanner command |
| 36 | `?? .sisyphus/notepads/run-release-warnings/` | git status before capture | Git working-tree status | exact no-fix rationale: dirty-file context only, not emitted release/build warning | Path contains `warnings`, matching scanner pattern incidentally while recording pre-existing untracked directory | No fix: evidence context, preserve dirty-file record | same scanner command |
| 44 | `===== build: npm run tauri build -- --no-bundle =====` | build boundary | Capture script | exact no-fix rationale: required phase boundary, not warning output | Boundary contains `--no-bundle`, matching scanner pattern incidentally | No fix: required evidence boundary | same scanner command |
| 47 | `> tauri build --no-bundle` | build | npm script / Tauri CLI command echo | exact no-fix rationale: command echo, not warning output | Command contains `--no-bundle`, matching scanner pattern incidentally | No fix: recipe-preserving command echo | same scanner command |
| 124 | `▲ [WARNING] Expected identifier but found whitespace [css-syntax-error]` | build | esbuild CSS minifier via Vite | frontend/Vite/Svelte | `src/app.css:90` uses `scrollbar-color: var(--ctp-surface2) transparent;`; esbuild's CSS minifier rejects this value form during production CSS minification | Task 2 | `npm run build` and scanner over the build log |
| 131 | `▲ [WARNING] Unexpected "var(" [css-syntax-error]` | build | esbuild CSS minifier via Vite | frontend/Vite/Svelte | Same `src/app.css:90` declaration as line 124; second parser diagnostic for the `var(` token in the same property | Task 2 | `npm run build` and scanner over the build log |
| 141 | `dist/assets/BundleLoaderView-Bu4UASdg.css    4.85 kB │ gzip:   1.24 kB` | build | Vite asset size report | exact no-fix rationale: normal emitted asset filename/size line, not a warning | Generated chunk name includes `BundleLoaderView`, matching scanner pattern incidentally | No fix: build artifact name only | same scanner command |
| 147 | `dist/assets/BundleLoaderView-Bo20i7N-.js     6.84 kB │ gzip:   2.66 kB` | build | Vite asset size report | exact no-fix rationale: normal emitted asset filename/size line, not a warning | Generated chunk name includes `BundleLoaderView`, matching scanner pattern incidentally | No fix: build artifact name only | same scanner command |
| 157 | `- Adjust chunk size limit for this warning via build.chunkSizeWarningLimit.` | build | Vite/Rollup chunk-size advisory | frontend/Vite/Svelte | Context lines 152-156 report `App-BwVPn97T.js` at 841.95 kB after minification, above Vite's 500 kB chunk-size warning threshold | Task 2 | `npm run build` and scanner over the build log |

## Supplemental captured frontend diagnostics not matched by the mandated regex

The Svelte plugin emitted warning diagnostics that do not contain the scanner terms on the primary message line, but they are build warnings and should be considered by Task 2 alongside the scanner matches.

| Raw log line | exact text | phase | emitting tool/module | classification | suspected root cause | fix owner task | verification command |
|---:|---|---|---|---|---|---|---|
| 57 | `8:55:10 AM [vite-plugin-svelte] src/components/TracksPanel.svelte:199:18 Avoid using autofocus` | build | vite-plugin-svelte / Svelte compiler | frontend/Vite/Svelte | Edit input uses `autofocus`, producing Svelte `a11y_autofocus` warning | Task 2 | `npm run build` |
| 65 | `8:55:10 AM [vite-plugin-svelte] src/components/Console.svelte:4:6 `scrollEl` is updated, but is not declared with `$state(...)`. Changing its value will not correctly trigger updates` | build | vite-plugin-svelte / Svelte compiler | frontend/Vite/Svelte | Svelte 5 reactive state warning: mutable `scrollEl` binding is declared as a plain `let` instead of `$state(...)` | Task 2 | `npm run build` |
| 73 | `8:55:10 AM [vite-plugin-svelte] src/components/MapView.svelte:830:2 Unused CSS selector ".track-point-marker"` | build | vite-plugin-svelte / Svelte compiler | frontend/Vite/Svelte | Component CSS selector is not matched in Svelte's static analysis, likely because related marker DOM is created outside the component template or selector is stale | Task 2 | `npm run build` |
| 81 | `8:55:10 AM [vite-plugin-svelte] src/components/MapView.svelte:840:2 Unused CSS selector ".track-point-marker.selected"` | build | vite-plugin-svelte / Svelte compiler | frontend/Vite/Svelte | Same static-analysis/stale-selector root cause as line 73 for the selected state selector | Task 2 | `npm run build` |
| 89 | `8:55:10 AM [vite-plugin-svelte] src/components/MapView.svelte:845:2 Unused CSS selector ".track-point-marker:active"` | build | vite-plugin-svelte / Svelte compiler | frontend/Vite/Svelte | Same static-analysis/stale-selector root cause as line 73 for the active-state selector | Task 2 | `npm run build` |
| 97 | `8:55:10 AM [vite-plugin-svelte] src/components/WaypointsPanel.svelte:93:16 Avoid using autofocus` | build | vite-plugin-svelte / Svelte compiler | frontend/Vite/Svelte | Edit input uses `autofocus`, producing Svelte `a11y_autofocus` warning | Task 2 | `npm run build` |
| 105 | `8:55:10 AM [vite-plugin-svelte] src/components/SymbolPicker.svelte:77:4 Visible, non-interactive elements with a click event must be accompanied by a keyboard event handler. Consider whether an interactive element such as `<button type="button">` or `<a>` might be more appropriate` | build | vite-plugin-svelte / Svelte compiler | frontend/Vite/Svelte | Popover wrapper is a `div` with `onclick` but no keyboard handler, producing `a11y_click_events_have_key_events` | Task 2 | `npm run build` |
| 113 | `8:55:10 AM [vite-plugin-svelte] src/components/SymbolPicker.svelte:77:4 `<div>` with a click handler must have an ARIA role` | build | vite-plugin-svelte / Svelte compiler | frontend/Vite/Svelte | Same popover wrapper `div` with click handler lacks an ARIA role, producing `a11y_no_static_element_interactions` | Task 2 | `npm run build` |

## Classification summary

- Frontend/Vite/Svelte assigned to Task 2: 11 actionable build warning rows (2 esbuild CSS scanner rows, 1 Vite chunk-size scanner row, 8 supplemental Svelte plugin diagnostics).
- Tauri config/platform assigned to Task 3: none captured.
- Rust app crate / local path dependency / registry dependency assigned to Task 4: none captured.
- Release startup runtime output assigned to Task 5: no warning/error-like startup scanner matches captured. Startup emitted two `INFO` lines from `ozi_rs_lib::application`, which did not match the required scanner terms.
- Exact no-fix rationale rows: 7 scanner matches were command/status/artifact lines rather than warnings.
