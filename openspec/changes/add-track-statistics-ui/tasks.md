## 1. Validate delta

- [x] 1.1 Re-read `openspec/specs/track-display/spec.md` and confirm the modified requirement text matches the spec verbatim before adding the new scenarios.
- [x] 1.2 Run `openspec validate add-track-statistics-ui --strict` and ensure it passes before implementation begins.

## 2. Extend TrackSummaryDto

- [x] 2.1 Add `distance_km: f64`, `duration_seconds: Option<u64>`, and `point_count: u32` to `TrackSummaryDto` in `src-tauri/src/commands/mod.rs`.
- [x] 2.2 Populate the new fields in `get_app_state` (and any other DTO factory) using `Track::total_distance_km`, `Track::total_duration`, and `Track::point_count`; convert `Duration` to whole seconds and represent the missing-timestamps case as `None`.

## 3. Sync types.ts

- [x] 3.1 Mirror the three new fields on `TrackSummary` in `src/lib/types.ts`, matching the Rust DTO casing produced by serde.
- [x] 3.2 Update any other TypeScript consumers that destructure `TrackSummary` so the new optional `duration_seconds` is handled.

## 4. Render in TracksPanel

- [x] 4.1 In `src/components/TracksPanel.svelte`, render the statistics next to the track name as `"<distance> km · <duration> · <points> pts"`, formatting distance to one decimal place and duration as `<h>h <m>m` (or `<m>m` when under an hour).
- [x] 4.2 Hide the duration segment (and its surrounding separator) when `duration_seconds` is `null`/`undefined`.

## 5. Frontend test for stats display

- [x] 5.1 Add a TracksPanel test that mounts a track with distance/duration/point count and asserts the row text contains the formatted statistics.
- [x] 5.2 Add a second case proving the duration segment is omitted when `duration_seconds` is missing.

## 6. QA via just test/just clippy

- [x] 6.1 Run `just clippy` and resolve any new warnings on touched files.
- [x] 6.2 Run `just test` and confirm Rust and frontend suites stay green.
