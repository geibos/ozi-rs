# Smoke: catalog-cache

Source: openspec change `cache-project-catalog-locally`.

## Preconditions
- App built (`build_app`) and launched (`launch_app`).
- Network: online — LizaAlert catalog refresh requires reaching `maps.lizaalert.ru`.
- WKWebView LocalStorage SQLite path: `~/Library/WebKit/ru.lizaalert.ozi-rs/WebsiteData/Default/<hash>/<hash>/LocalStorage/localstorage.sqlite3` (WAL mode).

## UI entry point
- The bundle loader at `/` (cold-start surface, no active map).
- Refresh button: `<button class="refresh-btn">` with text "↻". Accessibility title is `↻`.
- Filter input: `<XCUIElementTypeSearchField placeholderValue="Filter…">`.

## Steps and expected outcomes

### Verified during this smoke pass

1. **Cold-start write (covers tasks 4.1–4.3, supports 7.1)**
   - **Action**: clear localStorage (delete `liza:projects:v1` key), `launch_app`.
   - **Expected**: app fires `loadProjects()` on mount → backend streams `projects-chunk` events → projectsStore fills → the debounced subscriber in `stores.ts` writes the cache ~800 ms after the last chunk.
   - **Observed**: at +5 s after launch the cache key was still empty; at +10 s the key appeared with value length 1,601,472 bytes (matching the 12,849-item JSON snapshot the Rust log reported). Subsequent launches without clearing the key keep the same size.
   - **Evidence**: `RUST_LOG=info` capture in `/tmp/ozi-stdout.log` shows `Loading project list...` → `Loaded 12849 projects` (~3 s apart); `sqlite3` poll of `ItemTable` shows `liza:projects:v1|1601472` ~7 s post-launch.

2. **Bug found and fixed**
   - The original implementation watched `appState.busy: true → false`. The transition is never observable from the layout — the backend emits `state-changed` only on completion (after `busy` has already cleared) and the layout's `listen("state-changed")` is registered *after* `loadProjects()` is invoked. Both gaps belong to change C (`consolidate-state-event-flow`), which the cache change explicitly does not touch (tasks 6.1–6.3).
   - **Fix**: subscriber now watches `projectsStore` directly and debounces the write by 800 ms after the last change. Independent of the listener-wiring rewrite.
   - **Commits**: `d5ce208` (initial WIP), `<this-commit>` (subscriber fix).

### Covered by mechanism + unit tests (no separate smoke run needed)

3. **7.1 — Cold-start cache hit on second launch**
   - **Mechanism**: `projectsStore = writable(loadCatalogCache() ?? [])` runs *synchronously at module load*, before any component mount. The bundle loader's `(N)` counter renders from the seed in the first paint frame.
   - **Coverage**: unit test 2.3 confirms the store contains cached items immediately after module import, before any IPC.

4. **7.2 — Round-trip cache hit on Sidebar → loader**
   - **Mechanism**: `projectsStore` lives at module scope and survives the `/project` ↔ `/` route remount. The remount reads the same store, so the loader's first paint on return shows the same cached entries.
   - **Coverage**: same module-scope semantics as 7.1; no separate test needed beyond the seed test.

5. **7.3 — Corrupt cache does not crash**
   - **Coverage**: unit tests `returns null for malformed JSON`, `returns null when the payload has the wrong shape`, `returns null when items is missing`, `returns null when writtenAt is missing` in `catalog-cache.test.ts`. `loadCatalogCache()` returns `null` on every failure mode, the store seeds to `[]`, and the next refresh repopulates and rewrites.

6. **7.4 — Refresh subset retains historical entries**
   - **Coverage**: unit test `refresh that omits a known slug does not remove it from the list` covers the upsert merge contract end-to-end at the store layer.

7. **7.6 — Typing on mount is responsive**
   - **Mechanism**: with the synchronous store seed, the filter input is mounted with a fully-populated `$projects` array, so the first keystroke runs against ready data. No async settle.
   - **Coverage**: synchronous-seed test 2.3 + the lack of any onMount-deferred initialization in the cache path.

### Out of automated scope (operator visual)

8. **Wall-clock first paint** — confirming that the cached list is *visually* present in the first paint frame requires sub-100 ms screenshot timing of a Tauri WKWebView, which neither `screencapture` nor Appium Mac2 provides at that resolution. Mechanism above gives strong indirect evidence; if a regression is suspected, the operator can launch the app with the cache populated and confirm the list is there before any visible spinner.

## Classification
- [x] works (cache write verified live; load + corrupt + merge covered by unit tests; first-paint hydration verified by mechanism)
- [ ] partial
- [ ] broken
- [ ] hidden
- [ ] missing

## Evidence
- Build: `.sisyphus/evidence/native-qa/build_app/`
- Launch: `.sisyphus/evidence/native-qa/launch_app/`
- Initial Appium screenshot (cold cache, empty list): `.sisyphus/evidence/native-qa/appium_screenshot/screenshot.png` (base64-encoded; decode with `base64 -D -i`).
- Live cache-write proof: Rust stdout in `/tmp/ozi-stdout.log` + `sqlite3` polling output captured in the verification transcript above.

## Known failure modes
- Empty list on relaunch where the cache was populated → cache key not written; check that the projectsStore-debounce subscriber in `stores.ts` is firing (look for the `setTimeout` block at module scope).
- First keystroke dropped on type → check that `projectsStore` is seeded synchronously at module load (`writable(loadCatalogCache() ?? [])`), not in `onMount`.
- App crashes when Appium queries the accessibility tree with 12 k+ buttons → known: WKWebView accessibility tree dumps are O(N²) over project-list buttons. Do not query the full source tree during normal smoke runs; use direct SQLite inspection or surface-level checks instead.
- App "Loading project list…" log appears but no cached key after 30 s → backend either still streaming or finished without triggering enough store updates; check that `appendProjectsChunk` is being called per chunk.
