## Context

After the redesign trilogy (`redesign-workspace-shell-layout` + `redesign-library-pane` + `redesign-inspector-pane`) shipped, a manual smoke run uncovered five bugs the visual review missed. Three are wiring regressions where new components don't consume stores the old code did; one is a backend-side viewport-restore gap; one is a UX-trap where a button looks like an input. Two of the five (the JSON-toast and the double-click freeze) also expose a deeper hole: this codebase has no in-dev surface for the failure mode "the frontend sent the wrong payload shape" or "two IPCs raced". Errors today surface as Sonner toasts whose `description: String(error)` reduces structured errors to `[object Object]` or to a one-line Tauri error string that hides the actual offending field.

This proposal does five targeted wiring fixes, plus one cross-cutting addition: a dev-only IPC-error toast whose `data-testid="ipc-error"` is grep-able from a smoke harness. The dev toast is additive — it does not replace the existing user-facing `toast.error("Failed to …")` calls; it stacks alongside them so smoke can both see the friendly message and the raw payload that caused it.

## Goals / Non-Goals

**Goals:**
- Restore viewport fit-to-bounds on cold-start session restore (parity with click-open).
- Show per-row download progress and a status-bar progress line in the workspace when a bundle is downloading.
- Make "Maps…" double-click a no-op (gated on `$busy`).
- Make the Cmd-K trigger visibly NOT-an-input.
- Surface IPC failures with the full error text in dev so the next regression is debuggable from a screenshot.
- Land all fixes with zero new `listen()` registrations (the `consolidate-state-event-flow` single-owner invariant holds).

**Non-Goals:**
- Production telemetry / Sentry-style aggregation of IPC errors.
- Restyling the bundle-loader Sheet, the Library Rail beyond the Maps tab row, or the Inspector Rail.
- Adding new Tauri IPC commands or new event names.
- Map preview thumbnails (still tracked under `map-preview-thumbnails`).
- Server-side download retries / resumability (out of scope; existing semantics stand).

## Decisions

### Decision 1: Dev-only IPC error toast carries `data-testid="ipc-error"` and the full structured payload

**Decision**: introduce a thin wrapper around `@tauri-apps/api/core::invoke` at `src/lib/api.ts` (or a new `src/lib/ipc.ts` if `api.ts` gets unwieldy — implementation choice for the applier) that, on rejection in `import.meta.env.DEV` builds, fires a Sonner toast with:

- title: the IPC command name (e.g. `open_selected_map`)
- description: the JSON-stringified error object, with a fallback to `String(error)` if stringify itself throws
- `data-testid="ipc-error"` set on the toast root via Sonner's `id` / class affordance
- `duration: Infinity` (sticky) so the user can read the full payload before it auto-dismisses

In production builds the wrapper SHALL pass the error through untouched; user-facing toasts at call sites continue to render the friendly message.

**Rationale**: the user's screenshot showed `Failed to load project: undefined` — the description string was empty because the error from the IPC reject was an object with `message` nested inside `data`, and `String({message:...})` returns `[object Object]`. The wrapper turns this into a smoke-testable artefact and removes the guess-work for the next regression. `data-testid` (not a class) because smoke harnesses already query by `data-testid` in the existing tests.

**Alternatives considered**:
- _Global `window.onerror` / `unhandledrejection` handler_: rejected — IPC rejects are not always uncaught (most are wrapped in `try / catch` with a user-facing toast), so a global handler would either miss them or duplicate the friendly toast.
- _Replace every `toast.error("Failed to …")` call site individually_: rejected — high diff churn, easy to miss a site, and the user-facing message is intentionally friendly. A wrapper is the lowest-blast-radius addition.

### Decision 2: Session-restore drives the active map through `register_active_map_layer`, same as a click-open

**Decision**: modify `Application::restore_session` in `src-tauri/src/application/mod.rs:1104-1185` so that after the restored `ActiveMapSelection` is validated and set on `self.lizaalert.active_map`, the function ALSO calls `self.register_active_map_layer(&selection)` (or the equivalent helper) just like `open_selected_map` does at `application/mod.rs:352-366` and `application/mod.rs:415-429`. Errors from the register step SHALL be reported via `update_status` with `DiagnosticLevel::Error` and the active map cleared, matching the existing failure mode of `open_selected_map`.

The frontend `MapView.svelte` effect at `MapView.svelte:686-758` is unchanged — once the backend has the active map registered, the existing `activeMapRef` slice fires and `fitBounds(meta.bounds)` runs against the OZI metadata. The user-visible difference is that on cold-start the canvas now lands inside the calibrated bounds instead of at top-left zoom 0.

**Rationale**: the click-open path works (user confirmed). The session-restore path stops one step short of where the click-open path runs — it sets the selection but never calls the layer-registration helper. The cleanest fix is to make session-restore call the same helper. This keeps the viewport-fit logic in one place (`MapView.svelte::applyActiveMap`) and avoids splitting the fit-bounds responsibility across the frontend and the session restorer.

**Alternatives considered**:
- _Frontend-only fix: have the route page issue a `fitBounds` call after redirecting to `/project`_: rejected — duplicates logic already in `MapView`, and the route page doesn't know the bounds without an extra IPC.
- _Emit a synthetic `bundle-progress` "indexing complete" event from `restore_session`_: rejected — abuses an event meant for download progress.

### Decision 3: Library Maps tab consumes `downloadingMaps` / `downloadProgress` and the workspace status bar consumes `bundleProgress`

**Decision**:

- `src/components/library/MapsTab.svelte` SHALL import `downloadingMaps` and `downloadProgress` from `$lib/stores`. For each map row, if `$downloadingMaps.has(m.name)`, the row SHALL render a progress badge (e.g. percentage from `$downloadProgress.get(m.name)`) in place of (or beside) the existing leading-control slot. When the map transitions from `downloading` to `downloaded: true` (via the existing `state-changed` -> `appState.refresh()` flow), the badge SHALL flip to `cached`.
- `src/components/WorkspaceShell.svelte` (or wherever the status bar lives) SHALL subscribe to `bundleProgress` and render the same one-line text that `BundleLoader.svelte` shows. The status bar is otherwise unchanged; this is a single read-and-render addition into the existing `data-testid="workspace-status-bar"` element.

**Rationale**: the data is already on the frontend — `+layout.svelte:56-58` writes `bundleProgress` and the `download-progress` events already fill `downloadProgress`. The Library Maps tab was authored after the trilogy and was written against `currentProject.maps[].downloaded` only, missing the streaming-availability wiring. The status bar was emptied during the workspace-shell redesign and never re-wired. Both are pure wiring fixes — no new stores, no new events.

**Alternatives considered**:
- _Single combined badge "downloading 47%"_: kept open as an implementation choice; the spec only mandates that progress is observable per row.
- _Move the per-row progress to a tooltip_: rejected — the user reported the BUG that progress is invisible; a tooltip is invisible by default and does not fix the complaint.

### Decision 4: "Maps…" double-click serialisation via `$busy` short-circuit, not via promise dedup

**Decision**: `MapsTab.svelte::openLoader` and any other `loadProjects()` trigger SHALL early-return when `get(busy) === true`. The button SHALL also visibly show busy state (e.g. `disabled={$busy}`) so the user sees that the click had no effect. The store `busy` is already derived from `appState.busy` at `src/lib/stores.ts:87`.

**Rationale**: the failure mode is the second `load_projects` IPC reaching the backend before the first finishes; the backend serialises them and the second waits ~5s. Short-circuiting on the frontend is one line, leans on existing state, and matches how the `BundleLoader`'s refresh button already gates itself (`disabled={$busy}` at `BundleLoader.svelte:148`). A promise-dedup layer in `api.ts` would also work but is broader than this bug warrants.

**Alternatives considered**:
- _`debounce(300)` on the click_: rejected — adds visible latency to the first click for a problem that only manifests on the second.
- _Backend-side idempotent `load_projects`_: out of scope; the backend's serialisation is correct, the frontend just shouldn't queue.

### Decision 5: Cmd-K trigger looks like a chord button, not an input

**Decision**: the trigger in `WorkspaceShell.svelte:101-109` SHALL be styled per these rules:

- NO `<input>` element. Stays as `<button type="button">`.
- NO blinking caret, NO `cursor: text`, NO box-shadow / outline that mimics input focus rings.
- The label inside the trigger SHALL NOT use placeholder ellipsis ("Search…") that reads as input-state. Acceptable labels: "Open command palette", "Command palette", or an icon + `⌘K` chord without verbose copy.
- The `⌘K` chord SHALL render as `<kbd>` (semantic) styled as a key-cap badge, visually heavier than a placeholder-style decoration.
- On hover/focus the trigger SHALL use button-style affordances (background tint, ring on focus) matching other buttons in the shell, NOT input-style affordances.
- The dialog opened by the trigger ALREADY contains a real `cmdk` `Command.Input` inside (`CommandPalette.svelte`); that internal input takes focus when the dialog opens. No change there.

**Rationale**: the user mistook a button for an input because it was styled like one. Removing the input-mimicking affordances is the fix; the underlying primitive (a button that opens a dialog) is correct.

**Alternatives considered**:
- _Make it a real `<input>` that synchronously feeds the palette's command store_: rejected — duplicates the `cmdk` input that lives inside the dialog, doubles the focus-management complexity, and breaks the "palette dialog is a single focus trap" pattern.
- _Hide the trigger entirely and rely on Cmd-K key chord_: rejected — discoverability regressed.

### Decision 6: All five fixes land in one change because they share the "redesign wiring slipped" theme

The five bugs are not independent enough to warrant five separate proposals — they share a single root context (the redesign trilogy missed wiring) and the IPC-error toast surfaces all of them simultaneously in dev. Splitting would multiply review overhead without splitting the implementation work; the tasks remain numbered per bug so the applier can land them in any order.
