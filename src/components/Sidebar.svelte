<script lang="ts">
  import {
    appState,
    status,
    activeMap,
    tracksPanelOpen,
    trackPointsPanelOpen,
    waypointsPanelOpen,
    addWaypointMode,
    drawingModeActive,
    activeTrackLayerId,
    activeWaypointLayerId,
    drawingTrackLayerId,
    drawingTrackId,
    drawingPointCount,
    drawingFinishRequested,
    drawingSegmentId,
    editModeActive,
    consoleOpen,
  } from "../lib/stores";
  import {
    importGpx,
    importPlt,
    saveProject,
    loadProjectFile,
    undo,
    redo,
    revealBundle,
    createEmptyTrack,
    getTrackDetail,
  } from "../lib/api";
  import ThemePicker from "./ThemePicker.svelte";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { openBundleLoader } from "../lib/windows";

  async function handleImportGpx() {
    const path = await open({ multiple: false, filters: [{ name: "GPX", extensions: ["gpx", "zip"] }] });
    if (path) await importGpx(path as string);
  }

  async function handleImportPlt() {
    const path = await open({ multiple: false, filters: [{ name: "PLT track", extensions: ["plt"] }] });
    if (path) await importPlt(path as string);
  }

  async function handleSave() {
    const path = await save({
      defaultPath: `${$appState?.project_name ?? "project"}.ozp`,
      filters: [{ name: "OZI Project", extensions: ["ozp"] }],
    });
    if (path) await saveProject(path);
  }

  async function handleOpen() {
    const path = await open({ multiple: false, filters: [{ name: "OZI Project", extensions: ["ozp"] }] });
    if (path) await loadProjectFile(path as string);
  }

  async function toggleTrackDrawingMode() {
    if ($drawingModeActive) {
      drawingFinishRequested.set(true);
      return;
    }

    const layerId = $activeTrackLayerId;
    if (layerId === null) return;

    try {
      editModeActive.set(false);
      addWaypointMode.set(false);
      const trackId = await createEmptyTrack(layerId, "New Track");
      const detail = await getTrackDetail(layerId, trackId);
      drawingTrackLayerId.set(layerId);
      drawingTrackId.set(trackId);
      drawingSegmentId.set(BigInt(detail.segments[0].id));
      drawingPointCount.set(0);
      drawingModeActive.set(true);
    } catch (error) {
      console.error("Failed to start track drawing mode", error);
    }
  }

  function handleActiveTrackLayerChange(event: Event) {
    const value = (event.currentTarget as HTMLSelectElement).value;
    activeTrackLayerId.set(BigInt(value));
  }

  function handleActiveWaypointLayerChange(event: Event) {
    const value = (event.currentTarget as HTMLSelectElement).value;
    activeWaypointLayerId.set(BigInt(value));
  }
</script>

<aside class="sidebar">
  <header class="sidebar-header">
    <span class="app-title">ozi-rs</span>
    <ThemePicker />
  </header>

  <div class="section">
    <div class="section-title">Project</div>
    <div class="btn-row">
      <button onclick={handleOpen}>Open</button>
      <button onclick={handleSave}>Save</button>
      <button onclick={undo} title="Undo">↩</button>
      <button onclick={redo} title="Redo">↪</button>
    </div>
  </div>

  <div class="section">
    <div class="section-title">Map</div>
    <button class="full primary" onclick={openBundleLoader}>Maps…</button>

    {#if $activeMap}
      <div class="active-map">
        <div class="active-map-label">Active</div>
        <div class="active-map-name">{$activeMap.project_name}</div>
        <div class="active-map-pkg">{$activeMap.package_name}</div>
        <button class="full secondary small" onclick={revealBundle}>Reveal in Finder</button>
      </div>
    {/if}
  </div>

  <div class="section">
    <div class="section-title">Tracks</div>
    {#if ($appState?.track_layers ?? []).length > 0}
      <label class="layer-picker">
        <span>Track layer</span>
        <select
          value={$activeTrackLayerId?.toString() ?? ""}
          onchange={handleActiveTrackLayerChange}
          disabled={$drawingModeActive}
        >
          {#each $appState?.track_layers ?? [] as layer (layer.id)}
            <option value={String(layer.id)}>{layer.name}</option>
          {/each}
        </select>
      </label>
    {/if}
    <div class="btn-row">
      <button onclick={handleImportGpx}>Import GPX</button>
      <button onclick={handleImportPlt}>Import PLT</button>
    </div>
    <button
      class="full"
      class:active={$drawingModeActive}
      onclick={toggleTrackDrawingMode}
      title={$drawingModeActive
        ? "Finish drawing mode and keep the new track."
        : "Create an empty track and click on the map to add points."}
    >
      {$drawingModeActive ? `Done (${$drawingPointCount} points)` : "Create Track"}
    </button>
    <button class="full" onclick={() => tracksPanelOpen.update((v) => !v)}>
      {$tracksPanelOpen ? "Hide" : "Show"} Tracks Panel
    </button>
    <button class="full" onclick={() => trackPointsPanelOpen.update((v) => !v)}>
      {$trackPointsPanelOpen ? "Hide" : "Show"} Points Panel
    </button>
  </div>

  <div class="section">
    <div class="section-title">Waypoints</div>
    {#if ($appState?.waypoint_layers ?? []).length > 0}
      <label class="layer-picker">
        <span>Waypoint layer</span>
        <select
          value={$activeWaypointLayerId?.toString() ?? ""}
          onchange={handleActiveWaypointLayerChange}
        >
          {#each $appState?.waypoint_layers ?? [] as layer (layer.id)}
            <option value={String(layer.id)}>{layer.name}</option>
          {/each}
        </select>
      </label>
    {/if}
    <button class="full" onclick={() => waypointsPanelOpen.update(v => !v)}>
      {$waypointsPanelOpen ? "Hide" : "Show"} Waypoints Panel
    </button>
    <button
      class="full"
      class:active={$addWaypointMode}
      disabled={$drawingModeActive}
      onclick={() => addWaypointMode.update(v => !v)}
      title="Click on the map to place a waypoint. Press Escape to cancel."
    >
      {$addWaypointMode ? "Cancel Add Waypoint" : "Add Waypoint"}
    </button>
  </div>

  <div
    class="status-bar"
    class:error={$status.toLowerCase().includes("error") || $status.toLowerCase().includes("failed")}
  >
    <span>{$status}</span>
    <button
      class="console-toggle"
      onclick={() => consoleOpen.update((v) => !v)}
      title="Toggle console (`)"
    >›_</button>
  </div>
</aside>

<style>
  .sidebar {
    width: 200px;
    height: 100%;
    background: var(--ctp-mantle);
    border-right: 1px solid var(--ctp-surface0);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    flex-shrink: 0;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 10px;
    background: var(--ctp-crust);
    border-bottom: 1px solid var(--ctp-surface0);
  }

  .app-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--ctp-mauve);
  }

  .section {
    padding: 8px;
    border-bottom: 1px solid var(--ctp-surface0);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .section-title {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--ctp-overlay1);
    margin-bottom: 2px;
  }

  .btn-row {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }

  button.full {
    width: 100%;
    text-align: left;
  }

  .layer-picker {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: 10px;
    color: var(--ctp-overlay1);
  }

  .layer-picker select {
    width: 100%;
    min-width: 0;
    font-size: 11px;
  }

  button.primary {
    background: var(--ctp-blue);
    color: var(--ctp-base);
    border-color: var(--ctp-blue);
  }

  button.primary:hover { filter: brightness(1.1); }

  button.active {
    background: var(--ctp-green);
    color: var(--ctp-base);
    border-color: var(--ctp-green);
  }

  button.active:hover { filter: brightness(1.1); }

  button.secondary {
    background: transparent;
    border-color: var(--ctp-surface2);
    color: var(--ctp-subtext1);
  }

  button.small {
    font-size: 11px;
    padding: 2px 6px;
  }

  .active-map {
    background: var(--ctp-surface0);
    border-radius: 4px;
    padding: 6px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .active-map-label {
    font-size: 10px;
    color: var(--ctp-overlay1);
  }

  .active-map-name {
    font-size: 11px;
    font-weight: 500;
    color: var(--ctp-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .active-map-pkg {
    font-size: 11px;
    color: var(--ctp-subtext0);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .status-bar {
    margin-top: auto;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 8px;
    font-size: 10px;
    color: var(--ctp-subtext0);
    background: var(--ctp-crust);
    border-top: 1px solid var(--ctp-surface0);
    min-height: 24px;
  }

  .status-bar.error { color: var(--ctp-red); }

  .status-bar span {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .console-toggle {
    background: none;
    border: none;
    color: var(--ctp-overlay1);
    font-family: monospace;
    font-size: 11px;
    padding: 0 4px;
    flex-shrink: 0;
  }

  .console-toggle:hover { color: var(--ctp-text); }
</style>
