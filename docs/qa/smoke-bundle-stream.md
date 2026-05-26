# Smoke: bundle-stream (per-file availability)

Source: openspec change `stream-bundle-file-availability`.

## Preconditions
- App built (`build_app`) and launched (`launch_app`).
- Target bundle: ≥ 2 files of clearly-different sizes; smaller file should finish well before the larger one (so the difference is observable on screen, not just in logs).

## What is verified by code-level testing (no live smoke needed)

- **Backend emits both events in order** — `bundle-loader-progress.test.ts` asserts the `FileReady` branch in `src-tauri/src/commands/mod.rs` emits `bundle-file-ready` followed by `state-changed` in the same scope.
- **Frontend no longer subscribes to `bundle-file-ready` at the page level** — same test asserts `noteBundleFileReady` is gone from `+page.svelte`.
- **Removed affordances are gone** — same test asserts `data-testid="ready-files"` and `data-testid="open-bundle-now"` are absent.
- **Stores are clean** — `readyBundleFiles` and `noteBundleFileReady` removed from `stores.ts`; `bundle-loader-non-blocking.test.ts` no longer requires `readyBundleFiles` in the store source.

## Operator wall-clock check (optional)

1. Pick a bundle with two maps where one is clearly smaller (e.g. 50 MB vs 500 MB).
2. Click the project to start the download.
3. Watch the Maps column — the smaller map's row should flip from a blue `%` badge to a green `cached` badge **before** the larger map finishes. Click it while the larger is still downloading; the workspace should open with that map, the bundle download continues, and eventually the larger row flips to `cached` too.
4. The bundle loader's status bar should never show an "Open bundle now" button or a ready-files list at any point.

## Classification
- [x] works (backend emits both events; frontend removes legacy listener + DOM; static CSS shrinks the status bar; structural assertions pass)
- [ ] partial
- [ ] broken
- [ ] hidden
- [ ] missing

## Evidence
- Build: `.sisyphus/evidence/native-qa/build_app/`
- Unit-test coverage: `npx vitest run bundle-loader-progress` (asserts the new `bundle-file-ready → state-changed` pairing in `commands/mod.rs` and the absence of legacy DOM in `+page.svelte`).
- Code review: `git diff` for the `stream-bundle-file-availability` archive.

## Known failure modes
- Smaller-map row stays at `%` until the bundle completes → backend `state-changed` emit in the `FileReady` branch missing or out of order; check `commands/mod.rs` matches the spec.
- Status bar resizes during download → CSS `grid-template-rows` regressed; check `+page.svelte` `.status-bar` rule matches the four-row layout.
