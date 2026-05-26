## Context

The bundle-download pipeline already has the data the frontend needs; what's missing is the signal.

Backend, `src-tauri/src/infrastructure/lizaalert.rs:576`: workers send a `DownloadNotification::FileReady { package_name, local_path, file_index, file_count }` the moment a file has been written and fsync'd to its final path.

Backend forwarder, `src-tauri/src/commands/mod.rs:473-492`: the `FileReady` branch does two things — calls `AppState::note_bundle_file_ready(&package_name, &local_path)` (which appends to `ready_bundle_files` and, critically, sets `local_path = Some(...)` on the matching `LizaMapPackage` if the filename suffix matches) and emits a `bundle-file-ready` Tauri event with the per-file payload. It does NOT emit `state-changed`.

DTO, `src-tauri/src/commands/mod.rs:228-235`: `LizaMapPackageDto.downloaded` is computed at snapshot time as `m.local_path.is_some()`. So the moment `note_bundle_file_ready` runs, the next `getAppState()` call would already return `downloaded: true` for that map. The frontend just never finds out, because no `state-changed` is fired until the forwarder task finishes (`commands/mod.rs:516`).

Frontend, `src/routes/+page.svelte:88-90`: listens for `bundle-file-ready` and pushes payloads into `readyBundleFiles` via `noteBundleFileReady` (`stores.ts:97-108`). The store is rendered as the `ready-files` list and gates `canOpenPartial` (`+page.svelte:206`), which lights up the `Open bundle now` button (`+page.svelte:371-375`). Clicking the button calls `appState.refresh()` (`+page.svelte:208-210`) — which is exactly the missing signal, manually triggered.

The shape of the fix is therefore: stop relying on the user clicking "Open bundle now" to learn that map N is ready; have the backend tell the frontend at the moment of readiness, and let the Maps column re-render via the existing `currentProject` derived store.

## Goals / Non-Goals

**Goals:**
- Maps column reflects per-file readiness within the same tick the backend writes the file to disk.
- A row whose file just finished flips `blue %` → `green cached` and becomes clickable without any extra user gesture.
- The "Open bundle now" button and the ready-files list disappear from the status bar.
- Change A's stable-layout guarantee for the remaining status-bar content is preserved.

**Non-Goals:**
- Re-shaping the bundle loader into a Sheet / overlay (that's change F: `bundle-loader-as-overlay`).
- Persisting the LizaAlert catalog locally so cold-start is fast (that's change D: `cache-project-catalog-locally`).
- Consolidating the Tauri event listener topology to a single owner per event (that's change C: `consolidate-state-event-flow`).
- Adding typed payloads or version vectors to `state-changed`. We accept the existing coarse "go ask getAppState" contract.
- Throttling progress events. Frequency of `state-changed` is orthogonal here.

## Decisions

### Decision 1: Emit `state-changed` alongside `bundle-file-ready`, not a richer payload

When a file becomes ready, the forwarder SHALL keep emitting `bundle-file-ready` (its per-file payload still has value for telemetry and for any future per-file UI), AND SHALL additionally emit `state-changed` so that all `state-changed` consumers (`+layout.svelte:33` and the layout's reactive store) pull a fresh `AppStateDto`.

The frontend SHALL no longer rely on `bundle-file-ready` to drive `m.downloaded`. The `m.downloaded` flag is read from `AppStateDto.current_project.maps[i].downloaded`, which already reflects the just-set `local_path` (see Context).

**Rationale**: This is the smallest possible backend change — one extra `app.emit("state-changed", ())` line in the forwarder's `FileReady` branch (`commands/mod.rs:482`). It reuses the existing `state-changed` → `getAppState()` → store-update plumbing that the frontend already runs on every other mutation. No new DTO field, no new payload contract, no new TypeScript type to keep in sync.

**Alternatives considered**:
- _Strategy 2: enrich the `bundle-file-ready` payload with the new `MapAvailability` slice and have the frontend mutate `appState` locally._ Rejected: it duplicates the source of truth for `m.downloaded` (one source in `AppStateDto`, another in a delta event), and Svelte component code now has to know which DTO fields to splice. The frontend's invariant today is "after `state-changed`, re-read everything via `getAppState`". Bypassing that for one field invites drift the moment another field (`downloading_maps`, `status`) needs to update in the same tick.
- _Backend emits a brand-new `map-availability-changed` event with only the changed entries._ Rejected: that's a new IPC contract and a new TypeScript type — far more cost than emitting one extra `state-changed`. If event-rate per file becomes a measured problem, change C's debouncing layer can deal with it for all `state-changed` traffic at once.

### Decision 2: Remove `Open bundle now` and the ready-files list entirely

Both UI elements were workarounds for the "Maps column doesn't update until the bundle finishes" gap. Once Decision 1 closes that gap, both elements give the user nothing the Maps column doesn't already give them (more directly and more compactly).

- The `Open bundle now` button SHALL be removed from `+page.svelte`. The `handleOpenPartial` function, the `canOpenPartial` derived value, and the page-level `bundle-file-ready` listener SHALL be removed.
- The `ready-files` list (DOM + the `ready-list-slot` grid area) SHALL be removed from `+page.svelte`.
- The `readyBundleFiles` store and the `noteBundleFileReady` helper SHALL be removed from `src/lib/stores.ts`. `resetBundleDownloadState` SHALL no longer reference them.

**Rationale**: keeping a dead button or a dead store as "defensive code" is misleading documentation. The status bar slot count is also a perceived-clutter problem; collapsing two rows reduces it without losing information.

### Decision 3: Status-bar height shrinks, layout invariant from change A is preserved

The `.status-bar` CSS grid in `+page.svelte:625-633` currently has five rows:

```
"status-line   actions"   20px
"current-file  actions"   16px
"progress      actions"   12px
"meta          actions"   16px
"ready-list    actions"   minmax(80px, 1fr)
```

After this change the grid SHALL have four rows:

```
"status-line   actions"   20px
"current-file  actions"   16px
"progress      actions"   12px
"meta          actions"   16px
```

The `--bundle-status-bar-height` custom property SHALL drop from `160px` to `~80px` (exact value to be tuned with `padding` to look right). The bar still uses CSS-grid named slots with reserved sub-row heights — that is the contract change A added to the spec. The only thing changing here is that one slot is gone; the remaining slots SHALL keep their reserved-height guarantees so progress events do not reflow neighbouring UI.

This is a one-time outer-height change between "before this change" and "after this change". The runtime invariant from change A — *during* a session, the bar's outer height does not jitter as events arrive — is preserved.

### Decision 4: Don't touch the bundle-file-ready event signature

The Tauri event `bundle-file-ready` and its payload (`BundleFileReadyPayload` in `commands/mod.rs:200-207`) SHALL remain unchanged in this change. The backend still emits it. We just stop consuming it on the frontend side.

**Rationale**: the event has a documented contract in `lizaalert-integration/spec.md:30` ("emit one `bundle-file-ready` event per file as soon as that file is fully written"). Removing it would be a separate, larger spec change. Keeping it leaves room for future listeners (telemetry, a desktop notification, the eventual overlay UI from change F) without re-introducing a new event type.

## Risks / Trade-offs

- **Risk**: emitting `state-changed` on every file-ready inflates the frontend's `state-changed` traffic during a download. For a typical SAR bundle of ~10-20 files this is ~10-20 extra events per download, spread over seconds. The frontend already runs `getAppState()` on each `state-changed`; the extra round-trip cost is bounded by file completion rate (worst case: small files, ~1 per second). Acceptable, and change C will tighten this end-to-end if needed. **Mitigation**: none required for this change.
- **Risk**: a future field added to `AppStateDto` that must update on the same tick as a file-ready will need to ride on the same `state-changed` emission. **Mitigation**: that's exactly the model `state-changed` is built for; this change is consistent with the existing contract, not an exception to it.
- **Trade-off**: removing the ready-files list loses a chronological "what arrived in what order" log. We accept this — the Maps column is reordered by prefix, not by arrival order, but each row's badge tells you whether that map is here yet, which is what the user actually needs.

## Overlap with other changes

- **Change A (`bundle-loader-non-blocking`)** — shipped. This change builds on top: A made cached map rows clickable; this change makes them flip to "cached" sooner. The CSS-grid status-bar pattern from A is reused with one fewer row.
- **Change C (`consolidate-state-event-flow`)** — drafted, not implemented. C plans to move all Tauri listeners (including `bundle-file-ready`) into `+layout.svelte`. This change removes the only existing page-level `bundle-file-ready` listener entirely (its work is now done by `state-changed`). After this change lands, there are zero `bundle-file-ready` listeners in the codebase. C's "single owner per event" invariant accommodates "zero owners" trivially. There is no conflict.
- **Change D (`cache-project-catalog-locally`)** — not implemented; unrelated to per-file readiness streaming.
- **Change F (`bundle-loader-as-overlay`)** — not implemented; rebuilds the loader as a Sheet. When F lands it inherits the streaming behavior from this change for free, because the data flow lives in the store / DTO, not in the loader's outer shell.

## Migration Plan

Pure UI / event behavior change. No data migration, no session-file change, no spec contract removed (events kept, only listeners reshuffled).

Land in a single PR:

1. Backend: add the one-line `app.emit("state-changed", ())` to the `FileReady` branch.
2. Frontend: remove the page-level `bundle-file-ready` listener, the `readyBundleFiles` store, the `noteBundleFileReady` helper, the `canOpenPartial` derived value, the `handleOpenPartial` function, the `Open bundle now` button DOM, the `ready-files` list DOM, and the `ready-list-slot` grid area.
3. Frontend: shrink `--bundle-status-bar-height` and update `.status-bar` `grid-template-areas` / `grid-template-rows`.
4. Manual smoke per `tasks.md` group 5.
