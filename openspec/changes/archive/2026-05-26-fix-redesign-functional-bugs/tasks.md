## 1. Bug 1 — IPC error toast surface (dev only)

- [x] 1.1 Add a thin wrapper around `@tauri-apps/api/core::invoke` in `src/lib/api.ts` (or a new `src/lib/ipc.ts`) that, on rejection in `import.meta.env.DEV`, fires a sticky Sonner toast whose root carries `data-testid="ipc-error"`, title = IPC command name, description = `JSON.stringify(error)` with `String(error)` fallback for unstringifiable payloads
- [x] 1.2 Route every existing `invoke(...)` call in `src/lib/api.ts` through the wrapper; do NOT change any user-facing `toast.error(...)` call site at the consumer level (the dev toast stacks alongside)
- [x] 1.3 Reproduce the original cold-start JSON-error and the project-selection JSON-error with the wrapper active; capture the structured payload and file the precise field-mismatch as a follow-up engineering note (the wrapper is the surface, the underlying payload fix lands in the relevant flow's task — likely Bug 2 / Bug 3 covers it once the payload is visible)

## 2. Bug 2 — Viewport restore on cold-start

- [x] 2.1 In `src-tauri/src/application/mod.rs::restore_session` (≈ lines 1104-1185), after the restored `ActiveMapSelection` is validated and `local_path` confirmed to exist, call `self.register_active_map_layer(&selection)` (the same helper invoked by `open_selected_map` at `application/mod.rs:352-366` / `415-429`); on `Err`, report via `update_status(DiagnosticLevel::Error, …)` and clear `self.lizaalert.active_map`
- [x] 2.2 Confirm via cold-start smoke that `MapView.svelte`'s `applyActiveMap` effect runs with `meta.bounds` non-null for the restored OZI map and that `map.fitBounds(meta.bounds, ...)` lands the viewport at calibrated bounds, NOT at `{ center: [0,0], zoom: 0 }`
- [x] 2.3 Add a Rust unit test (or extend an existing one in `src-tauri/src/application/mod.rs` tests) that asserts `restore_session` calls `register_active_map_layer` when a valid persisted active map is present

## 3. Bug 3 — Library Maps tab consumes download progress + status bar mirrors bundle progress

- [x] 3.1 In `src/components/library/MapsTab.svelte`: import `downloadingMaps` and `downloadProgress` from `$lib/stores`; in the `{#each maps as m}` loop replace the leading-control slot with logic that renders an in-progress indicator (badge with percentage) when `$downloadingMaps.has(m.name)`, falls back to the `cached` badge when `m.downloaded`, and renders the existing neutral spacer otherwise
- [x] 3.2 In `src/components/WorkspaceShell.svelte`: import `bundleProgress` from `$lib/stores`; render `$bundleProgress?.message` (or an equivalent phase + completed/total summary) inside the existing `data-testid="workspace-status-bar"` element when the store is non-null; render the existing idle content otherwise
- [x] 3.3 Static-grep verify that no new `listen(...)` registration was added by tasks 3.1 / 3.2; the layout-owner invariant from `consolidate-state-event-flow` must hold

## 4. Bug 4 — Double-click Maps serialisation

- [x] 4.1 In `src/components/library/MapsTab.svelte`: import `busy` from `$lib/stores`; bind `disabled={$busy}` on the "Maps…" button at `MapsTab.svelte:48-51` AND guard the `openLoader` callback with `if (get(busy)) return;` (defence-in-depth in case a future call path bypasses the disabled attribute)
- [x] 4.2 Verify by manual smoke that clicking "Maps…" twice in <500ms while a `load_projects` is in flight sends exactly one IPC (instrument with the dev-toast wrapper from task 1 if needed) and that the UI does not freeze

## 5. Bug 5 — Cmd-K trigger restyle

- [x] 5.1 In `src/components/WorkspaceShell.svelte` at lines 101-109 (and the `.cmdk-*` CSS at line ~236): remove the `cmdk-label` "Search…" copy; replace with either an icon-only trigger, a "Command palette" label, or a `⌘K`-only chord. The CSS SHALL set `cursor: pointer` (not `text`), SHALL NOT mimic input focus (no inset border, no input-like ring), and SHALL match the shell's other button affordances
- [x] 5.2 Keep the `<kbd class="cmdk-glyph">⌘K</kbd>` chord rendered as a key-cap badge styled heavier than placeholder copy
- [x] 5.3 Confirm by manual smoke that clicking the trigger opens the `CommandPalette` dialog AND that focus lands in the dialog's internal `cmdk` `Command.Input` AND that typing characters there filters the palette — i.e. the user can search

## 6. Verification

- [x] 6.1 Manual: cold-start with a persisted active OZI map — confirm viewport lands at calibrated bounds (matches click-open viewport) and no IPC-error toast appears
- [x] 6.2 Manual: open the Library Maps tab while a bundle is downloading — confirm per-row progress indicator updates as files land and flips to `cached` when each map is fully written
- [x] 6.3 Manual: confirm workspace status bar shows the current `bundleProgress` text during a download and clears on completion
- [x] 6.4 Manual: double-click "Maps…" — confirm exactly one IPC fires (button visibly disabled, no 5s freeze)
- [x] 6.5 Manual: inspect the Cmd-K trigger — confirm no caret, no input-like ring, no "Search…" label; clicking opens the palette dialog with internal input focused; typing filters results
- [x] 6.6 Manual: induce an IPC failure in a dev build (e.g. by feeding an invalid `load_project` slug) — confirm a sticky `data-testid="ipc-error"` toast appears with the full JSON-stringified error payload
- [x] 6.7 `just ci` passes (clippy, check, lint, test)
- [x] 6.8 `openspec validate fix-redesign-functional-bugs --strict` passes
