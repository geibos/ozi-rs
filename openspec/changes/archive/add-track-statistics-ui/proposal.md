## Why

The `track-display` capability already mandates that per-track statistics (distance, duration, point count) surface in the track row UI, and the Rust domain (`src-tauri/src/domain/track.rs:291-326`) exposes `total_distance_km`, `total_duration`, and `point_count`. The Svelte `TracksPanel` does not yet render these values, so SAR coordinators cannot judge a track's coverage from the list and must drill into details for every track. Surfacing the numbers in the row closes that gap and brings the implementation in line with the spec.

## What Changes

- Extend `TrackSummaryDto` in `src-tauri/src/commands/mod.rs` with `distance_km: f64`, `duration_seconds: Option<u64>`, and `point_count: u32`, populated through the existing `Track::total_distance_km`, `Track::total_duration`, and `Track::point_count` methods inside `get_app_state` (no new domain logic).
- Mirror the new fields in `src/lib/types.ts` so the TypeScript `TrackSummary` type stays in sync with the Rust DTO.
- Update `src/components/TracksPanel.svelte` to render the statistics compactly alongside the track name (for example `12.3 km · 1h 24m · 156 pts`), omitting the duration segment when `duration_seconds` is `None`.

## Impact

- Affected capability: `track-display`.
- Affected code: `src-tauri/src/commands/mod.rs` (DTO + `get_app_state` mapping), `src/lib/types.ts` (DTO mirror), `src/components/TracksPanel.svelte` (row rendering), plus a frontend test covering the new display.
- No new commands, no schema migrations: existing project files load unchanged because statistics are derived on read.
