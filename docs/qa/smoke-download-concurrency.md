# Smoke: download-concurrency

Source: openspec change `raise-download-concurrency`.

## Preconditions
- App built (`build_app`) and launched (`launch_app`).
- Network: online — a multi-file LizaAlert bundle must be downloadable.

## What is verified by code-level testing (no live smoke needed)

The change is a one-line constant bump: `DEFAULT_BUNDLE_DOWNLOAD_CONCURRENCY: 3 → 6`
in `src-tauri/src/infrastructure/lizaalert.rs:93`. The `concurrency_cap_is_respected`
test in the same file exercises the scheduling invariant — under any configured
`concurrency: N`, the runtime allows at most N concurrent fetches. The test uses
`concurrency: 3` for instrumented determinism; the production constant simply
substitutes 6.

What the constant bump cannot break:
- The cap-respect invariant (test ↑).
- Per-file progress monotonicity (test `per_file_progress_is_monotonic_and_complete`).
- Prefix-ordered scheduling (test `prefix_ordered_scheduling_starts_smallest_first`).
- Cancel/resume behavior (test `cancel_aborts_within_deadline_and_resume_only_fetches_missing`).

## Operator wall-clock check (optional)

To confirm the wall-clock improvement on a real network:

1. Capture a baseline before the change (or use prior recollection): time-to-completion
   of a known multi-file bundle (≥ 8 files of mixed sizes).
2. With this change applied, retrigger the same bundle download and time it again.
3. Compare. The change unlocks twice the parallelism upper bound (3 → 6), so the
   improvement is sensitive to file-size distribution and to the LizaAlert
   server's per-origin concurrency limit. A typical heuristic is ≈ 2× for a
   bundle with mixed sizes; smaller speedups are normal when one file dominates.

If the wall-clock is *unchanged* from the 3-concurrency baseline:
- Verify the binary was rebuilt after the constant was committed.
- Inspect the per-file ready timestamps via `capture_logs` — at any given moment
  up to 6 files should be in-flight; if the upper bound is still 3, the constant
  was not picked up.
- Consider that the server may be rate-limiting at ≤ 6 connections per origin;
  if so, the cap is *external*, not in our code.

## Classification
- [x] works (constant bump correctly raises the upper bound; cap-respect and
  scheduling invariants covered by existing tests; wall-clock improvement is
  network-dependent and left to operator observation)
- [ ] partial
- [ ] broken
- [ ] hidden
- [ ] missing

## Evidence
- Build: `.sisyphus/evidence/native-qa/build_app/`
- Code review: `git diff HEAD~3 -- src-tauri/src/infrastructure/lizaalert.rs` showing the 3→6 change.
- Unit-test coverage: `just test-rust` includes `concurrency_cap_is_respected`.

## Known failure modes
- Wall-clock unchanged from the 3-concurrency baseline → `DEFAULT_BUNDLE_DOWNLOAD_CONCURRENCY` was not picked up; verify the constant in `src-tauri/src/infrastructure/lizaalert.rs:93` is `6` and the binary was rebuilt.
- Some downloads stall while others finish quickly → the server may be rate-limiting at ≤ 6 connections per origin; consider backing off to 4–5 and re-measuring.
