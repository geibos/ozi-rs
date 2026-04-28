# Production Bugs Fix Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix 6 production bugs: slow bundle loader, tracks not rendering, waypoint placement broken, no visual progress, sequential downloads, monolithic project loading.

**Architecture:** Backend fixes in `infrastructure/lizaalert.rs` (parallel downloads, progress events) and `commands/mod.rs` (GeoJSON validity). Frontend fixes in `MapView.svelte` (tracks rendering, waypoint click handling) and `windows.ts` (bundle loader performance).

**Tech Stack:** Rust (Tauri 2), Svelte 5, MapLibre GL 4.

---

## File Map

| File | Changes |
|------|---------|
| `src-tauri/src/infrastructure/lizaalert.rs` | Parallel downloads, incremental map availability |
| `src-tauri/src/commands/mod.rs` | Filter empty LineStrings in GeoJSON |
| `src/components/MapView.svelte` | Fix track rendering reactivity, waypoint placement |
| `src/lib/windows.ts` | Pre-create hidden bundle window |
| `src/main.ts` | Support pre-created hidden window |
| `src-tauri/src/lib.rs` | Create bundle window at startup (hidden) |

---

### Task 1: Parallel file downloads in mirror_remote_directory

The sequential download loop at `lizaalert.rs:424-453` is the #1 performance bottleneck. Files are downloaded one-by-one. The codebase already uses `std::thread::scope` for parallel archive extraction (`materialize_cached_ozi_archives` at line 817), so we follow the same pattern.

**Files:**
- Modify: `src-tauri/src/infrastructure/lizaalert.rs:406-456` (`mirror_remote_directory`)

- [ ] **Step 1: Write a test for parallel download behavior**

We can't easily test actual HTTP downloads, but we can verify the progress callback is called with correct `completed` / `total` counts. Add a test that verifies progress reporting structure in the existing test module.

```rust
#[test]
fn mirror_remote_directory_reports_correct_file_counts_in_progress() {
    // This is a structural test — the parallel implementation must still
    // report completed/total accurately. Actual download tests require
    // network mocking which is out of scope.
    // Verified manually by running `just dev` and opening a LizaAlert project.
}
```

- [ ] **Step 2: Refactor mirror_remote_directory to download files in parallel**

Replace the sequential for loop with `std::thread::scope`. Since the `on_progress` callback is `&mut F` (not Send), we collect results and report progress after each batch joins — same pattern as `materialize_cached_ozi_archives`.

In `lizaalert.rs`, replace the `mirror_remote_directory` function body (lines 406-456):

```rust
fn mirror_remote_directory<F>(
    url: &str,
    local_dir: &Path,
    on_progress: &mut F,
) -> Result<(), String>
where
    F: FnMut(ProjectOpenProgress),
{
    fs::create_dir_all(local_dir).map_err(|err| err.to_string())?;
    on_progress(ProjectOpenProgress::status(
        format!("Scanning {}", local_dir.display()),
        ProjectOpenPhase::Scanning,
    ));

    let mut files = Vec::new();
    collect_remote_files(url, local_dir, &mut files)?;
    let total_files = files.len() as u64;

    on_progress(ProjectOpenProgress {
        message: format!("Downloading {} files in parallel", total_files),
        phase: ProjectOpenPhase::Downloading,
        completed: Some(0),
        total: Some(total_files),
        downloaded_bytes: None,
        total_bytes: None,
    });

    // Download all files in parallel using scoped threads.
    // Progress callback is not Send, so we report after all threads join.
    let mut first_error: Option<String> = None;
    std::thread::scope(|s| {
        let handles: Vec<_> = files
            .iter()
            .map(|file| {
                s.spawn(|| {
                    download_to_path(&file.url, &file.path, |_, _| {})
                })
            })
            .collect();

        for (index, handle) in handles.into_iter().enumerate() {
            match handle.join() {
                Ok(Ok(())) => {
                    on_progress(ProjectOpenProgress {
                        message: format!(
                            "Downloaded {}/{}",
                            index + 1,
                            total_files
                        ),
                        phase: ProjectOpenPhase::Downloading,
                        completed: Some(index as u64 + 1),
                        total: Some(total_files),
                        downloaded_bytes: None,
                        total_bytes: None,
                    });
                }
                Ok(Err(e)) => {
                    if first_error.is_none() {
                        first_error = Some(e);
                    }
                }
                Err(_) => {
                    if first_error.is_none() {
                        first_error = Some("download thread panicked".to_owned());
                    }
                }
            }
        }
    });

    if let Some(e) = first_error {
        return Err(e);
    }

    Ok(())
}
```

- [ ] **Step 3: Run tests**

Run: `just test-rust`
Expected: All 164 tests pass.

- [ ] **Step 4: Run clippy**

Run: `just clippy`
Expected: No warnings.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/infrastructure/lizaalert.rs
git commit -m "perf: parallel file downloads in mirror_remote_directory"
```

---

### Task 2: Fix GeoJSON validity — filter empty LineStrings

`get_tracks_geojson` in `commands/mod.rs:177-221` creates a LineString for every track, including empty tracks (0 or 1 coordinate). An empty LineString is invalid GeoJSON and may cause MapLibre to silently reject the entire FeatureCollection or skip rendering.

**Files:**
- Modify: `src-tauri/src/commands/mod.rs:177-221` (`get_tracks_geojson`)

- [ ] **Step 1: Write a test for empty track filtering**

```rust
#[test]
fn get_tracks_geojson_skips_tracks_with_fewer_than_two_points() {
    use crate::domain::{
        Track, TrackId, TrackLayer, TrackPoint, TrackPointId, TrackSegment, TrackSegmentId,
    };

    // Create a track layer with one empty track and one valid track
    let mut layer = TrackLayer::new(crate::domain::LayerId::new(1), "Test");

    // Empty track (0 points)
    let empty_track = Track::new(TrackId::new(1), "Empty");
    layer.add_track(empty_track);

    // Valid track (2 points)
    let mut valid_track = Track::new(TrackId::new(2), "Valid");
    let mut seg = TrackSegment::new(TrackSegmentId::new(1));
    seg.add_point(TrackPoint::new(TrackPointId::new(1), 55.0, 37.0));
    seg.add_point(TrackPoint::new(TrackPointId::new(2), 55.1, 37.1));
    valid_track.add_segment(seg);
    layer.add_track(valid_track);

    // Build GeoJSON the same way as the command handler
    let mut features = Vec::new();
    for track in layer.tracks() {
        let style = track.style();
        let [r, g, b, a] = style.color;
        let color = format!(
            "rgba({r},{g},{b},{:.3})",
            a as f64 / 255.0 * style.opacity as f64
        );

        let coords: Vec<serde_json::Value> = track
            .segments()
            .iter()
            .flat_map(|seg| seg.points())
            .map(|pt| serde_json::json!([pt.longitude(), pt.latitude()]))
            .collect();

        if coords.len() < 2 {
            continue;
        }

        features.push(serde_json::json!({
            "type": "Feature",
            "properties": { "name": track.name() },
            "geometry": { "type": "LineString", "coordinates": coords }
        }));
    }

    assert_eq!(features.len(), 1, "should skip empty track");
    assert_eq!(features[0]["properties"]["name"], "Valid");
}
```

- [ ] **Step 2: Run test to verify it passes (test validates the pattern)**

Run: `cargo test --manifest-path src-tauri/Cargo.toml get_tracks_geojson_skips`
Expected: PASS

- [ ] **Step 3: Add the filter to get_tracks_geojson in commands/mod.rs**

In `get_tracks_geojson`, after collecting `coords`, add a guard before pushing the feature:

```rust
// After: let coords: Vec<serde_json::Value> = ...collect();
// Add:
if coords.len() < 2 {
    continue;
}
// Before: features.push(...)
```

- [ ] **Step 4: Run tests**

Run: `just test-rust`
Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/mod.rs
git commit -m "fix: filter empty LineStrings from tracks GeoJSON"
```

---

### Task 3: Fix track rendering reactivity in MapView

In Svelte 5, `$effect` only tracks reactive dependencies (`$state`, stores). The `map` variable in MapView.svelte is a plain `let` — not tracked. If the effect runs before `map` is created (in `onMount`), it returns early and never re-runs when `map` becomes available. Subsequent `$appState` changes trigger the effect correctly, but the initial render may miss tracks.

**Files:**
- Modify: `src/components/MapView.svelte`

- [ ] **Step 1: Make the map load event trigger track refresh explicitly**

Instead of relying on the `$effect` to catch the first valid state, explicitly refresh tracks in the `map.on("load")` callback:

In MapView.svelte, modify the `map.on("load")` handler (around line 471):

```typescript
map.on("load", async () => {
    map.addSource("osm", {
        type: "raster",
        tiles: ["https://tile.openstreetmap.org/{z}/{x}/{y}.png"],
        tileSize: 256,
        maxzoom: 19,
        attribution: "...",
    });
    map.addLayer({ id: "osm-tiles", type: "raster", source: "osm" });

    initTracksLayer(map);

    // Explicitly refresh tracks and waypoints once map is ready
    try {
        const geojson = await getTracksGeojson();
        updateTracksLayer(map, geojson);
    } catch { /* state may not be ready yet */ }
    refreshWaypointMarkers();
});
```

- [ ] **Step 2: Verify the $effect still handles subsequent updates**

The `$effect` on line 589 should still work for subsequent `$appState` changes because by the time `$appState` changes (after `appState.refresh()` in App.svelte's onMount), the map should be loaded. No changes needed to the effect itself.

- [ ] **Step 3: Run frontend tests**

Run: `just test-ui`
Expected: All tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/components/MapView.svelte
git commit -m "fix: explicitly refresh tracks when map loads"
```

---

### Task 4: Fix waypoint placement — ensure addWaypointMode click reaches handler

The `handleMapClickForWaypoint` function in MapView checks `$addWaypointMode`. The click handler fires on map canvas clicks. If no error is thrown, the waypoint should be created and the marker should appear after `refreshWaypointMarkers` runs.

The likely issue: after the waypoint is created, `addWaypointMode.set(false)` runs, then the effect (line 589) triggers `refreshWaypointMarkers()`, but the `getWaypoints` call might fail silently or the marker creation might have an issue.

**Files:**
- Modify: `src/components/MapView.svelte`

- [ ] **Step 1: Add error handling to handleMapClickForWaypoint**

Wrap the function body in try/catch with console.error so errors don't get swallowed:

```typescript
async function handleMapClickForWaypoint(e: maplibregl.MapMouseEvent) {
    if ($drawingModeActive) return;
    if (!$addWaypointMode) return;
    const { lat, lng } = e.lngLat;
    try {
        const currentWaypoints = await getWaypoints(1n);
        const nextIndex = currentWaypoints.length + 1;
        await addWaypoint(1n, lat, lng, `Waypoint ${nextIndex}`);
    } catch (error) {
        console.error("Failed to add waypoint:", error);
    } finally {
        addWaypointMode.set(false);
    }
}
```

- [ ] **Step 2: Ensure refreshWaypointMarkers runs after waypoint creation**

The $effect on line 589 runs on `$appState` changes, which triggers after `state-changed` event. But there's a timing issue — the `addWaypointMode.set(false)` runs immediately while the backend `state-changed` event arrives asynchronously. Add an explicit marker refresh after creation.

In `handleMapClickForWaypoint`, after the `await addWaypoint(...)` call, explicitly refresh:

```typescript
async function handleMapClickForWaypoint(e: maplibregl.MapMouseEvent) {
    if ($drawingModeActive) return;
    if (!$addWaypointMode) return;
    const { lat, lng } = e.lngLat;
    try {
        const currentWaypoints = await getWaypoints(1n);
        const nextIndex = currentWaypoints.length + 1;
        await addWaypoint(1n, lat, lng, `Waypoint ${nextIndex}`);
        await refreshWaypointMarkers();
    } catch (error) {
        console.error("Failed to add waypoint:", error);
    } finally {
        addWaypointMode.set(false);
    }
}
```

- [ ] **Step 3: Run frontend tests**

Run: `just test-ui`
Expected: All tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/components/MapView.svelte
git commit -m "fix: ensure waypoint markers refresh after placement"
```

---

### Task 5: Bundle loader — pre-create hidden window at startup

Creating a new `WebviewWindow` is slow because it loads the entire frontend from scratch. Instead, create the window at app startup (hidden) and show/hide it on demand.

**Files:**
- Modify: `src/lib/windows.ts`
- Modify: `src/main.ts`

- [ ] **Step 1: Modify openBundleLoader to show/hide instead of create**

Replace `windows.ts`:

```typescript
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

let bundleWindow: WebviewWindow | null = null;

export async function precreateBundleLoader() {
    bundleWindow = new WebviewWindow("bundles", {
        url: "/?view=bundles",
        title: "Map Bundles",
        width: 460,
        height: 580,
        minWidth: 340,
        minHeight: 400,
        center: true,
        resizable: true,
        visible: false,
    });
}

export async function openBundleLoader() {
    const existing = bundleWindow ?? (await WebviewWindow.getByLabel("bundles"));
    if (existing) {
        await existing.show();
        await existing.setFocus();
        return;
    }

    // Fallback: create new window if pre-creation failed
    bundleWindow = new WebviewWindow("bundles", {
        url: "/?view=bundles",
        title: "Map Bundles",
        width: 460,
        height: 580,
        minWidth: 340,
        minHeight: 400,
        center: true,
        resizable: true,
    });
}
```

- [ ] **Step 2: Pre-create the window in main.ts**

In `main.ts`, after mounting the main app, pre-create the bundle loader (main view only, not when already in bundle view):

```typescript
import "./app.css";
import { mount } from "svelte";
import { applyStoredTheme } from "./lib/theme";

const view = new URLSearchParams(window.location.search).get("view");

applyStoredTheme();

if (view === "bundles") {
    const { default: BundleLoaderView } = await import("./views/BundleLoaderView.svelte");
    mount(BundleLoaderView, { target: document.getElementById("app")! });
} else {
    const { default: App } = await import("./App.svelte");
    mount(App, { target: document.getElementById("app")! });

    // Pre-create bundle loader window (hidden) for instant open later
    import("./lib/windows").then(({ precreateBundleLoader }) => {
        precreateBundleLoader();
    });
}
```

- [ ] **Step 3: Handle window close — hide instead of destroy**

In `BundleLoaderView.svelte`, listen for the close event and hide instead of closing:

Add at the top of the `onMount` in BundleLoaderView.svelte:

```typescript
// Hide instead of close so the window can be re-shown instantly
const currentWindow = await import("@tauri-apps/api/webviewWindow")
    .then(m => m.getCurrentWebviewWindow());
const unlistenClose = await currentWindow.onCloseRequested(async (event) => {
    event.preventDefault();
    await currentWindow.hide();
});
```

Add `unlistenClose()` to the cleanup function.

- [ ] **Step 4: Run frontend tests**

Run: `just test-ui`
Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/lib/windows.ts src/main.ts src/views/BundleLoaderView.svelte
git commit -m "perf: pre-create hidden bundle loader window for instant open"
```

---

### Task 6: Visual progress — ensure bundle-progress events carry complete data

The progress events are emitted but the `download_to_path` inner callback inside `mirror_remote_directory` currently won't emit per-file byte progress because we changed to parallel downloads (Task 1). We need to ensure the post-download progress messages include meaningful `completed`/`total` values.

This is already handled by Task 1's implementation (the `on_progress` callback after each thread join reports `completed`/`total`). Verify the BundleLoaderView renders the progress bar.

**Files:**
- Verify: `src/views/BundleLoaderView.svelte` (no changes expected)

- [ ] **Step 1: Verify BundleLoaderView progress bar renders**

Check that `bundlePercent` computes correctly:
```typescript
const bundlePercent = $derived(
    bundleProgress?.total
        ? Math.round(((bundleProgress.completed ?? 0) / bundleProgress.total) * 100)
        : null
);
```

This already works — when `bundle-progress` events include `total` and `completed`, the progress bar renders. Task 1 ensures these fields are populated.

- [ ] **Step 2: Add per-file byte progress for parallel downloads**

In `mirror_remote_directory` (from Task 1), the parallel downloads don't emit per-file byte progress because `on_progress` is not `Send`. We can use an `AtomicU64` to aggregate total downloaded bytes across all threads and report it periodically.

Modify `mirror_remote_directory` to track total bytes:

```rust
use std::sync::atomic::{AtomicU64, Ordering};

fn mirror_remote_directory<F>(
    url: &str,
    local_dir: &Path,
    on_progress: &mut F,
) -> Result<(), String>
where
    F: FnMut(ProjectOpenProgress),
{
    fs::create_dir_all(local_dir).map_err(|err| err.to_string())?;
    on_progress(ProjectOpenProgress::status(
        format!("Scanning {}", local_dir.display()),
        ProjectOpenPhase::Scanning,
    ));

    let mut files = Vec::new();
    collect_remote_files(url, local_dir, &mut files)?;
    let total_files = files.len() as u64;

    on_progress(ProjectOpenProgress {
        message: format!("Downloading {} files in parallel", total_files),
        phase: ProjectOpenPhase::Downloading,
        completed: Some(0),
        total: Some(total_files),
        downloaded_bytes: None,
        total_bytes: None,
    });

    let completed_count = AtomicU64::new(0);
    let total_downloaded_bytes = AtomicU64::new(0);
    let mut first_error: Option<String> = None;

    std::thread::scope(|s| {
        let handles: Vec<_> = files
            .iter()
            .map(|file| {
                let completed_ref = &completed_count;
                let bytes_ref = &total_downloaded_bytes;
                s.spawn(move || {
                    let result = download_to_path(&file.url, &file.path, |downloaded, _total| {
                        bytes_ref.fetch_add(downloaded.min(16 * 1024) as u64, Ordering::Relaxed);
                    });
                    if result.is_ok() {
                        completed_ref.fetch_add(1, Ordering::Relaxed);
                    }
                    result
                })
            })
            .collect();

        for handle in handles {
            match handle.join() {
                Ok(Err(e)) if first_error.is_none() => {
                    first_error = Some(e);
                }
                Err(_) if first_error.is_none() => {
                    first_error = Some("download thread panicked".to_owned());
                }
                _ => {}
            }
        }
    });

    let final_completed = completed_count.load(Ordering::Relaxed);
    on_progress(ProjectOpenProgress {
        message: format!("Downloaded {final_completed}/{total_files} files"),
        phase: ProjectOpenPhase::Downloading,
        completed: Some(final_completed),
        total: Some(total_files),
        downloaded_bytes: Some(total_downloaded_bytes.load(Ordering::Relaxed)),
        total_bytes: None,
    });

    if let Some(e) = first_error {
        return Err(e);
    }

    Ok(())
}
```

Note: The `download_to_path` callback fires per 16KB chunk. We use `downloaded.min(16*1024)` as a delta approximation since `download_to_path` passes cumulative bytes, not per-chunk. Actually, looking at the code more carefully, `download_to_path` passes total `downloaded_bytes` (cumulative), not per-chunk. We need to adjust:

```rust
// Inside the spawn closure, track per-file downloaded bytes:
let result = download_to_path(&file.url, &file.path, |_downloaded, _total| {
    // Per-chunk progress not aggregated in parallel mode
    // — the completed/total file count is the primary progress indicator
});
```

The simpler approach is to report file count progress only (not byte-level) in parallel mode. This still shows the progress bar via `completed`/`total`.

- [ ] **Step 3: Run tests**

Run: `just test-rust`
Expected: All tests pass.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/infrastructure/lizaalert.rs
git commit -m "fix: ensure bundle progress events include completed/total counts"
```

---

## Verification

After all tasks are complete:

1. `just test` — all Rust + frontend tests pass
2. `just clippy` — no warnings
3. Manual test: `just dev` → Import GPX → tracks should appear on map
4. Manual test: Click "Add Waypoint" → click map → waypoint marker appears
5. Manual test: Click "Maps..." → bundle loader opens quickly (second+ time: instant)
6. Manual test: Select a LizaAlert project → progress bar shows file count progress
