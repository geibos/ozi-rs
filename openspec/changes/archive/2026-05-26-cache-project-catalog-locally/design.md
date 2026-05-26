## Context

The LizaAlert project catalog is fetched by `load_projects` (`src/lib/api.ts:18`) and streamed back as `projects-chunk` events whose payload is `LizaProjectSummaryDto[]` (`src/lib/types.ts:8`). Today's flow:

1. `+page.svelte:onMount` registers a `projects-chunk` listener, then calls `loadProjects()` (`+page.svelte:91-103`).
2. Each chunk lands in `appendProjectsChunk` (`stores.ts:58`) which dedupes by `slug` and appends new entries to `projectsStore`.
3. The list re-derives `filtered` (`+page.svelte:112`) and re-renders the `{#each $projects.filter(...)}` block.
4. On unmount (e.g. user navigates to `/project`), the JS module persists; on a fresh app start the store starts empty again.

Three relevant constants:
- `LizaProjectSummaryDto` is a tiny shape — `{ slug: string; name: string }`. The catalog size is the dominant cost driver, not per-entry size.
- The catalog is dominated by historical SAR missions that essentially never change. New missions trickle in week by week.
- `localStorage` is available in the Tauri WebviewWindow without permission tweaks; quotas are platform-dependent but ≥5 MB on every supported OS.

This change is one of four addressing bundle-loader friction. The other three:
- **A — `bundle-loader-non-blocking`** (shipped): removed blanket UI lock and stabilized the status-bar layout.
- **B — `auto-create-default-layers`** (drafted): backend invariant that an open project has a default track and waypoint layer.
- **C — `consolidate-state-event-flow`** (drafted): single-listener topology, MapView slice re-renders, **and a 150 ms debounce on the filter input plus a single-shot `loadProjects()` from the layout at startup**.

The C overlap is intentional and deliberate: C debounces the filter to fix per-keystroke stutter under load. This change attacks a different layer of the same symptom — even with C in place, if the catalog is empty until the IPC finishes, the user is still blocked. This change makes the catalog non-empty from the first paint; C makes the per-keystroke filter pass cheap. They compose; neither replaces the other. See "Decisions" §1 for the exact handoff.

## Goals / Non-Goals

**Goals:**
- Cached catalog is hydrated synchronously from `localStorage` before any IPC fires.
- Re-entering the bundle loader from `/project` shows the previous catalog instantly.
- Background refresh merges new entries and updates existing ones, but never drops known historical entries on its own.
- List virtualization bounds the DOM cost of the bundle catalog to the viewport.

**Non-Goals:**
- Listener consolidation, debounced filter input, single-shot `loadProjects()` at startup — owned by change C (`consolidate-state-event-flow`). This change MAY touch `+page.svelte:onMount` for the cache-hydration step, but SHALL NOT remove or move any `listen()` call (C owns that wiring). The cache write/read helpers SHALL be independent of how `loadProjects()` is initiated so C's refactor can land cleanly afterward.
- Backend changes to `load_projects` or the `projects-chunk` shape. The wire format stays exactly as documented in `lizaalert-integration/spec.md`.
- A "Refresh now" UX redesign of the bundle loader. The existing refresh button (`+page.svelte:118`) keeps its meaning: trigger an immediate `loadProjects()`. The sheet / overlay rework is a separate change (F).
- Cache invalidation by TTL. The user explicitly said historical entries do not change — we keep them indefinitely. The user-facing refresh button is the sole "discard freshness" control needed.
- Per-entry diffing on the backend side. Refresh remains an "append-on-new-slug, replace-on-known-slug" merge in the frontend.

## Decisions

### Decision 1: localStorage with a versioned key, stale-while-revalidate

The cache SHALL live in `localStorage` under the key `liza:projects:v1`. The stored value SHALL be a JSON string of:

```text
{
  items: LizaProjectSummaryDto[],
  writtenAt: string  // ISO-8601 UTC
}
```

The hydration helper (`loadCatalogCache()` in `stores.ts`) SHALL:
- read the key, JSON-parse, and validate `items` is a non-empty array of `{ slug: string; name: string }`;
- on any parse / shape failure, silently return `null` (no toast, no error) so the UI behaves as if no cache existed;
- never throw.

The write helper (`saveCatalogCache(items)`) SHALL:
- be called once per refresh, after the refresh has settled (specifically on the `$appState.busy → false` transition), not per chunk;
- JSON-stringify and `localStorage.setItem`;
- swallow `QuotaExceededError` silently (log to the dev console only — the cache is best-effort).

The key suffix `v1` exists so that a future schema change can introduce `v2` while leaving stale `v1` entries to be garbage-collected by the browser.

**Rationale**: localStorage is synchronous and works inside `$effect` / module initialization without async ceremony. The catalog is small (few hundred entries × tiny DTO = tens of KB), well within the 5 MB quota on every supported platform. IndexedDB would force every read into a `Promise`, which defeats the "cache renders before the first paint" goal.

**Alternatives considered**:
- _IndexedDB via a wrapper (idb-keyval)_: better suited to large blobs, but every read becomes async. We'd need a layout-level prefetch and a "cache still loading" placeholder state. Not worth it for tens of KB.
- _Tauri-side disk cache (e.g. `app_data_dir/liza-projects.json`)_: requires a new IPC command, a new permission, and an async fetch on mount. Strictly worse than localStorage for this dataset.
- _In-memory only (just survive route changes within a session)_: doesn't address the cold-start case where the user opens the app and immediately needs to pick a bundle.

### Decision 2: Cache hydration is synchronous, before the first paint

`projectsStore`'s initializer (in `stores.ts`) SHALL eagerly call `loadCatalogCache()` and seed itself with the cached items, if any. This happens at module initialization, before any component mounts. By the time `+page.svelte` renders, `$projects` already contains the cached list.

If no cache exists (first-ever launch), `projectsStore` initializes to `[]` and the loader paints in its current "empty until chunks arrive" state.

**Rationale**: This is the smallest possible change to the existing reactive graph. `+page.svelte`'s `{#each filtered}` block needs no awareness that the data came from cache vs. IPC — it consumes the same store either way.

**Alternatives considered**:
- _Hydrate inside `+page.svelte:onMount`_: would paint once empty, then re-paint with the cache. Two paints for no reason; introduces the very flicker we are trying to remove.
- _Hydrate from a Svelte `load()` function_: prerender precludes runtime store access from route-level loaders (per `ui-shell` spec). Cannot use this path.

### Decision 3: Merge semantics — append-or-replace by slug, never delete

`appendProjectsChunk` (`stores.ts:58`) today filters out chunks whose slug already exists. This change replaces that with an "upsert by slug" merge:

- For each entry in the incoming chunk:
  - if `slug` already exists in `projectsStore` → replace the existing entry's `name` (and any future fields) in place;
  - else → append to the end.
- Order within the chunk is preserved; pre-existing entries that the chunk does not reference keep their position.

After all chunks of a refresh have arrived (signaled by `$appState.busy → false`), the cache SHALL be written.

**Rationale**: Historical missions never disappear from the user's perspective, even if the LizaAlert catalog stops serving them. A `name` correction (typo fix, callsign update on the server) SHALL flow through to the user. The cache is the conservative side of "memory of what we've ever seen"; the IPC is the authoritative side of "what's available right now". The merge keeps the union.

**Trade-off**: A slug that LizaAlert legitimately removes (e.g. a test mission deleted on the server) will linger in the user's local list forever, until they manually clear localStorage. Acceptable — the user explicitly chose "historical entries stay reachable" over "server is the source of truth". A future change MAY add a Settings affordance to clear the cache.

**Alternative considered**: _Diff against the previous refresh and prune entries missing from two consecutive full refreshes_. Adds complexity, requires storing the previous refresh separately, and contradicts the user's stated preference. Rejected.

### Decision 4: List virtualization via `@tanstack/svelte-virtual`

The bundle loader project list SHALL use `@tanstack/svelte-virtual` (the Svelte 5 port of TanStack Virtual) to render only the viewport-visible window. Fixed item height (single line, no images, no wrap) makes this a trivial fixed-size virtualizer — estimated item count is small enough that even an unvirtualized `{#each}` would work for most users, but the JIT-friendly path is needed once the catalog crosses ~300 entries and the Svelte reactive tree starts dominating filter-time.

Concretely:
- Add `@tanstack/svelte-virtual` to `package.json`.
- Wrap the existing `.list` container in `+page.svelte` (the project-column scroll body, `+page.svelte:239`) with the virtualizer. The list items keep their existing markup and click handler.
- Maps column (right side) is left alone — the per-project map count is in single digits.

**Rationale**: TanStack Virtual is the de-facto Svelte choice; it has a stable Svelte 5 release line, it's already in scope for similar lists in `TrackPointsPanel` (followup), and it's small enough that we are not pulling in a heavyweight grid library for a one-column scroll. `@neodrag/svelte` is a drag library, not virtualization — including it would have been a misread of its scope.

**Alternative considered**:
- _Hand-rolled windowing with `IntersectionObserver` and a sentinel_: works, but every panel that later needs virtualization re-implements it. One library, applied consistently.
- _No virtualization, rely on browser-native `content-visibility: auto`_: cheap to try but does not help with the JS-side cost of the filter `$derived` traversing a large list and the Svelte reactive subscription bookkeeping per row. Virtualization addresses both.

### Decision 5: Cache write is fire-and-forget; refresh remains the active path

When `$appState.busy` transitions from `true` to `false` and the current refresh produced at least one chunk, the system SHALL call `saveCatalogCache($projects)`. The write SHALL be wrapped in a try/catch that swallows `QuotaExceededError` and logs to the dev console without surfacing a toast.

The refresh itself remains the active code path — the user-visible "Projects" count updates as chunks arrive, just as today. The cache is a side-effect of refresh completion, not its primary path.

**Rationale**: Writing the cache on every chunk is wasteful (the JSON serialization is ~tens of KB and we'd do it 5+ times for a typical refresh). Writing on `busy → false` produces a single coherent snapshot per refresh. If the user kills the app mid-refresh, the previous good snapshot is preserved.

## Risks / Trade-offs

- **Risk**: Cached entries linger after they are removed server-side. **Mitigation**: documented in Decision 3 as deliberate. A future Settings entry can offer "Clear catalog cache". The cache is local-only and at worst causes a `slug not found` error if the user picks a deleted entry — the backend's existing error handling covers that case.
- **Risk**: `localStorage` quota is exceeded on a tiny percentage of platforms (typically because the user's localStorage is full of unrelated app data). **Mitigation**: write helper swallows `QuotaExceededError`; the next session simply behaves as if no cache exists.
- **Risk**: The schema of `LizaProjectSummaryDto` is extended later (e.g. add `last_modified`) and old cache reads return entries missing the new field. **Mitigation**: cache key is versioned (`liza:projects:v1`). The shape validator in `loadCatalogCache()` checks only the fields it knows about; older caches are tolerated until the key suffix is bumped. Stale entries get refreshed on the first background refresh.
- **Risk**: The virtualized list's keyboard / accessibility behavior differs from the native `{#each}`. **Mitigation**: TanStack Virtual preserves the underlying `<button>` semantics; the wrapper only controls which subset of items is materialized. Manual smoke check for tab-order and focus-restore on filter clear (see tasks).
- **Trade-off**: Two paint frames for refresh-after-cache (cache items, then merged refresh items) instead of one. Acceptable — the first frame is the user-perceived "I can pick something now" moment, which is the entire point of this change.
- **Trade-off**: Cache and live state diverge while the user has the loader open during a refresh. Acceptable — the divergence window is the refresh duration (typically a few seconds), and the merge converges them.

## Open Questions

- Q: Should the loader expose a "Last refreshed at" hint sourced from the cache's `writtenAt`? **A (tentative)**: not in this change. The status bar is already crowded after change A; adding a timestamp without a redesign would steal real estate. Defer to the bundle-loader visual refresh change (F).
- Q: Does the cache need to survive across app version updates? **A**: yes, by default — `localStorage` survives upgrades, and `v1` versioning lets us invalidate explicitly if the DTO changes. No special migration code is needed for the v1 → v1 transition (the schema is stable).

## Migration Plan

No migration. The first session after this change ships writes the cache; every subsequent session reads it. Users with no prior cache pay the same cost as today (refresh on every mount) until their first session completes a refresh successfully.
