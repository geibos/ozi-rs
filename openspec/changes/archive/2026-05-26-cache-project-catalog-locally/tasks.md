## 1. Local catalog cache primitives

- [x] 1.1 Add a `CatalogCachePayload` type in `src/lib/types.ts` describing `{ items: LizaProjectSummaryDto[]; writtenAt: string }`
- [x] 1.2 Add `loadCatalogCache(): LizaProjectSummaryDto[] | null` in `src/lib/stores.ts` (or a new `src/lib/catalog-cache.ts` if the helper grows too large to inline): read `localStorage["liza:projects:v1"]`, JSON-parse, validate the shape, return the items on success or `null` on any failure
- [x] 1.3 Add `saveCatalogCache(items: LizaProjectSummaryDto[]): void` that JSON-stringifies `{ items, writtenAt: new Date().toISOString() }` and calls `localStorage.setItem`; wrap in `try/catch` and swallow `QuotaExceededError` with only a `console.warn`
- [x] 1.4 Add unit tests in `src/test/` covering: round-trip (save then load returns identical items), corrupt cache returns `null` (malformed JSON, wrong shape, missing fields), quota exceeded does not throw

## 2. Hydration before first paint

- [x] 2.1 In `src/lib/stores.ts`, change the `projectsStore` initializer from `writable<LizaProjectSummaryDto[]>([])` to `writable<LizaProjectSummaryDto[]>(loadCatalogCache() ?? [])` so the store is seeded before any component mounts
- [x] 2.2 Verify that `+page.svelte`'s `$projects` derived view reflects the cached items on the first paint (no further code change required — the existing `derived(projectsStore, ...)` already propagates the seeded value)
- [x] 2.3 Add a Vitest test (or component-test stub) asserting that the projects store contains the cached items immediately after module import, before any IPC

## 3. Merge semantics (upsert by slug, never delete)

- [x] 3.1 Replace the body of `appendProjectsChunk(chunk)` in `src/lib/stores.ts` with an upsert: for each entry in `chunk`, if a matching `slug` exists in the current list update its fields in place; otherwise append at the end
- [x] 3.2 Preserve list order: existing entries keep their positions; new entries arrive at the end in the order they appear within their chunk
- [x] 3.3 Add unit tests in `src/test/`: appending a chunk with all-new slugs grows the list; appending a chunk with a mix of new and known slugs updates the known ones in place and appends only the new ones; a refresh that omits a known slug does not remove it from the list

## 4. Cache write on refresh completion

- [x] 4.1 Add a top-level subscription in `src/lib/stores.ts` that watches `projectsStore` and, debounced by ~800 ms after the last change, calls `saveCatalogCache(projects)`. Rationale (verified during smoke, see `docs/qa/smoke-catalog-cache.md`): the originally-spec'd `appState.busy: true → false` transition is not observable from the layout because the backend emits `state-changed` only on completion (after `busy` has already cleared) and the layout's listener is registered *after* `loadProjects()` is invoked — both gaps belong to change C (`consolidate-state-event-flow`). A debounce off `projectsStore` itself is independent of that wiring rewrite.
- [x] 4.2 Ensure the write fires only when the refresh produced at least one chunk (avoid overwriting a good cache with an empty list if the refresh failed before any data arrived) — gate on `projectsStore` being non-empty
- [x] 4.3 Add Vitest tests that drive `projectsStore` through realistic chunk patterns under `vi.useFakeTimers()` and confirm: (a) a single write fires after the debounce window elapses with no further changes; (b) no write when the list stays empty; (c) successive chunks restart the timer so the write reflects the final list

## 5. Virtualized project list (deferred to a follow-up change)

The initial attempt to wrap the project list with `@tanstack/svelte-virtual`
broke list rendering entirely under Tauri's WebView. The cache-layer pieces
(groups 1–4) deliver the dominant user-visible win on their own — the list
appears instantly from the cache and survives navigation. Virtualization is
deferred to a dedicated future change so it can be debugged in the running
WebView without holding up the cache work. Tasks below are kept for history;
status reflects what landed.

- [x] 5.1 Add `@tanstack/svelte-virtual` to `package.json` and run `npm install`; update `THIRD_PARTY_LICENSES.md` per the `ui-shell` third-party-credit requirement (package stays installed for the follow-up change)
- [ ] 5.2 _Deferred_: virtualizer wrapper around the project-column list (broke rendering in WebView; revisit in a dedicated change with browser-side debugging)
- [x] 5.3 Keep the maps-column list (`+page.svelte:266`) unvirtualized; the per-project map count is small and per-row progress markup benefits from being mounted whole
- [x] 5.4 Filter `$derived` continues to operate on the full `$projects` array (no virtualization in place to subset it)
- [ ] 5.5 _Deferred_: keyboard tab order / scroll / focus restoration smoke for virtualized rows — N/A until virtualization lands

## 6. Non-interference with change C (`consolidate-state-event-flow`)

- [x] 6.1 Do NOT add, remove, or move any `listen()` registration in `+page.svelte` or `+layout.svelte` — that wiring is owned by change C
- [x] 6.2 Do NOT add a debounce around `projectFilter` — change C owns the debounce decision
- [x] 6.3 Do NOT remove the `loadProjects()` call from `+page.svelte:onMount` — change C removes that duplicate; this change is orthogonal
- [x] 6.4 Verify by `git diff` that this change's touch surface in `+page.svelte` and `+layout.svelte` is limited to the virtualizer wrapper, comments, and zero IPC-wiring lines

## 7. Verification

- [x] 7.1 Verified during agent smoke (see `docs/qa/smoke-catalog-cache.md`): cache write proven live (SQLite `liza:projects:v1` lands at ~7 s after launch with the full 1.6 MB / 12849-item JSON). First-paint cache hit on the second launch follows from the synchronous `projectsStore = writable(loadCatalogCache() ?? [])` seed at module load and is covered by unit test 2.3.
- [x] 7.2 Same module-scope `projectsStore` seed satisfies the round-trip case: the loader remount on Sidebar→`/` reads the same store and paints the cached entries on the first frame. Mechanism described in `docs/qa/smoke-catalog-cache.md`.
- [x] 7.3 Covered by unit tests `returns null for malformed JSON`, `returns null when the payload has the wrong shape`, `returns null when items is missing`, `returns null when writtenAt is missing` in `catalog-cache.test.ts` (the contract is that `loadCatalogCache()` returns null on every failure mode, and the store seeds to `[]`).
- [x] 7.4 Covered by unit test `refresh that omits a known slug does not remove it from the list` in `catalog-cache.test.ts` (the upsert merge contract is the same in-app as in the test).
- [ ] 7.5 _Deferred_ with task 5.2: DOM-bounded `.list-item` count assertion belongs to the virtualization change
- [x] 7.6 Synchronous seed (verified by test 2.3) means the filter input is mounted with a fully-populated `$projects` array — no async settle, the first keystroke runs against ready data.
- [x] 7.7 `just ci` passes (clippy, check, lint, test) including the new Vitest cases added in 1.4, 2.3, 3.3, and 4.3
- [x] 7.8 `openspec validate cache-project-catalog-locally --strict` passes
