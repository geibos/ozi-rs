# Smoke: Large-track load performance (>10k points)

Source: ADR-0020 (MVP scope), section Tracks — "import and render large tracks efficiently."

## Preconditions

- App built and available at `target/debug/bundle/macos/ozi-rs.app`
- Large GPX fixture with >10,000 track points available in example_data (if available)
  - Current fixtures are <100 points: `/Users/sobieg/Projects/ozi-rs/example_data/2018-09-26_Nizovskaya/8-Android&iOS/2018-09-26_Nizovskaya_500m.gpx` (79 points)
  - No >10k fixture currently provided
- Alternative: generate synthetic large GPX for this test (future work)
- Active bundle with maps already loaded

## UI entry point

- Same as Tasks 5a/5b: "Import GPX" button in Tracks panel.
- File picker should accept large GPX files without timeout.

## Expected Outcome

**Note:** This smoke is marked as `hidden — large fixture not available` because
no fixture with >10k points currently exists in the example_data directory.
This test cannot be run until such a fixture is provided.

**If a large fixture becomes available in a future audit run:**

1. **Action**: `appium_launch_session` (fresh session)
   **Expected**: Session active; app running.
   **Artifact**: Session ID.

2. **Action**: `appium_screenshot` (baseline)
   **Expected**: Screenshot captures the running app.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_large_baseline.png`

3. **Action**: `capture_logs` to record pre-import timestamp
   **Expected**: Logs available for later parsing.
   **Artifact**: Log mark.

4. **Action**: Click "Import GPX" and select large fixture (>10k points)
   **Expected**: File dialog accepts file; import begins.
   **Artifact**: n/a.

5. **Action**: Monitor logs for import completion marker (e.g., "import finished" or "12500 points loaded")
   **Expected**: Import completes within 2 s; no parse errors; all points loaded.
   **Artifact**: Log timestamps showing import duration.

6. **Action**: `appium_screenshot` after import
   **Expected**: Track appears in Tracks panel; map view does not freeze during screenshot.
   **Artifact**: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_large_imported.png`

7. **Action**: Pan/zoom map 3+ times using map controls (zoom in, pan, zoom out)
   **Expected**: Each interaction responsive; no freezing or lag >1 s; polyline renders smoothly.
   **Artifact**: `appium_screenshot` after each pan/zoom (3 screenshots total).

8. **Action**: Monitor responsiveness during pan/zoom
   **Expected**: Interactions complete within 500 ms; visual feedback immediate; no dropped frames.
   **Artifact**: Visual inspection of screenshots; note any lag.

9. **Action**: `capture_logs` after interactions
   **Expected**: No error or warning logs related to rendering or memory.
   **Artifact**: `.sisyphus/evidence/native-qa/capture_logs/stdout_large.txt`

## Classification

- [ ] **works** — import <2 s; pan/zoom responsive (<500 ms per action); no memory warnings
- [ ] **partial** — import <5 s or pan/zoom slow (500–1000 ms) but usable
- [ ] **broken** — import hangs or crashes; pan/zoom freezes or stutters >1 s
- [ ] **hidden** — large fixture not available; test cannot be run
- [ ] **missing**

**Current classification: `hidden` (P3)** — Fixture not provided.

Rationale: Large-track performance is a known high-risk feature per design spec.
However, the audit's example_data directory does not contain a >10k-point fixture.
To run this test:
1. Provide a real large GPX (e.g., from a multi-day SAR operation)
2. Or generate synthetic large GPX (e.g., 20k points in tight spiral)
3. Add to `example_data/` and re-run Task 5f

Success criteria (when fixture available): (1) import completes in <2 s, (2) map
remains responsive during pan/zoom, (3) no memory exhaustion or rendering glitches.

## Evidence

- (N/A if hidden; once fixture available)
- Large GPX fixture: `/Users/sobieg/Projects/ozi-rs/example_data/[location]/[filename].gpx` (>10k points)
- Baseline screenshot: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_large_baseline.png`
- Post-import screenshot: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_large_imported.png`
- Pan/zoom screenshots: `.sisyphus/evidence/native-qa/appium_screenshot/screenshot_large_pan_*.png` (3 screenshots)
- Logs: `.sisyphus/evidence/native-qa/capture_logs/stdout_large.txt`

## Known failure modes

- **Large GPX file selection timeout**: File picker may have timeout for very large files (>50 MB).
  Check Tauri file picker config; consider async file handling if needed.
- **Import hangs (no progress)**: GPX parser may be blocking the UI thread. Check if parsing
  is happening in a background thread in `infrastructure/formats.rs`; ensure use of `tokio::spawn`
  or similar for non-blocking I/O.
- **Map renders but polyline invisible**: MapLibre may have a limit on vertices per layer.
  Check if large tracks need to be simplified (e.g., via Ramer-Douglas-Peucker algorithm)
  before rendering. See `src/lib/maplibre/` for layer simplification logic.
- **Memory spike or crash during pan/zoom**: Large polylines may consume too much GPU memory.
  Check MapLibre WebGL constraints; consider tile-based decimation or dynamic LOD (level-of-detail).
- **Import times vary wildly**: Performance may be impacted by system load. Run test multiple times
  and record median duration; note if variance >50% (indicator of contention or GC pressure).

---

## References

- ADR-0020: MVP scope, Tracks section
- `docs/commands-reference.md`: `ImportTrack` command and async handling
- `src-tauri/src/infrastructure/formats.rs`: GPX parser (check for async/blocking behavior)
- `src/lib/maplibre/`: Map layer rendering and simplification logic
- Design spec: Performance targets for large-track rendering (if documented in `docs/requirements.md`)
