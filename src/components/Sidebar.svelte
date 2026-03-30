<script lang="ts">
  import {
    appState,
    busy,
    status,
    projects,
    currentProject,
    activeMap,
    tracksPanelOpen,
    consoleOpen,
  } from "../lib/stores";
  import {
    loadProjects,
    loadProject,
    openSelectedMap,
    openLocalBundle,
    importGpx,
    importPlt,
    setBundlesRoot,
    saveProject,
    loadProjectFile,
    undo,
    redo,
    revealBundle,
  } from "../lib/api";
  import ThemePicker from "./ThemePicker.svelte";
  import { open, save } from "@tauri-apps/plugin-dialog";

  let selectedSlug = $state<string>("");

  async function handleLoadProjects() {
    await loadProjects();
  }

  async function handleSelectProject() {
    if (selectedSlug) await loadProject(selectedSlug);
  }

  async function handleOpenMap(mapName: string) {
    await openSelectedMap(mapName);
  }

  async function handleOpenBundle() {
    const dir = await open({ directory: true, multiple: false });
    if (dir) await openLocalBundle(dir as string);
  }

  async function handleImportGpx() {
    const path = await open({
      multiple: false,
      filters: [{ name: "GPX", extensions: ["gpx", "zip"] }],
    });
    if (path) await importGpx(path as string);
  }

  async function handleImportPlt() {
    const path = await open({
      multiple: false,
      filters: [{ name: "PLT track", extensions: ["plt"] }],
    });
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
    const path = await open({
      multiple: false,
      filters: [{ name: "OZI Project", extensions: ["ozp"] }],
    });
    if (path) await loadProjectFile(path as string);
  }

  async function handleSetBundlesRoot() {
    const dir = await open({ directory: true, multiple: false });
    if (dir) await setBundlesRoot(dir as string);
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
      <button onclick={undo}>↩</button>
      <button onclick={redo}>↪</button>
    </div>
  </div>

  <div class="section">
    <div class="section-title">LizaAlert Maps</div>
    <button class="full" onclick={handleLoadProjects} disabled={$busy}>
      {$busy ? "Loading…" : "Refresh Projects"}
    </button>

    {#if $projects.length > 0}
      <select bind:value={selectedSlug} class="full">
        <option value="">— select project —</option>
        {#each $projects as p}
          <option value={p.slug}>{p.name}</option>
        {/each}
      </select>
      <button class="full" onclick={handleSelectProject} disabled={!selectedSlug || $busy}>
        Open Project
      </button>
    {/if}

    {#if $currentProject}
      <div class="map-list">
        {#each $currentProject.maps as m}
          <button
            class="map-item"
            onclick={() => handleOpenMap(m.name)}
            disabled={$busy}
          >
            <span class="map-name">{m.name}</span>
            {#if m.downloaded}
              <span class="badge cached">cached</span>
            {:else}
              <span class="badge remote">↓</span>
            {/if}
          </button>
        {/each}
      </div>
    {/if}

    <button class="full" onclick={handleOpenBundle}>Open Local Bundle…</button>
    <button class="full secondary" onclick={handleSetBundlesRoot}>Set Bundles Root…</button>

    {#if $activeMap}
      <div class="active-map">
        <div class="active-map-label">Active Map</div>
        <div class="active-map-name">{$activeMap.project_name} / {$activeMap.package_name}</div>
        <button class="full secondary small" onclick={revealBundle}>Reveal in Finder</button>
      </div>
    {/if}
  </div>

  <div class="section">
    <div class="section-title">Tracks</div>
    <div class="btn-row">
      <button onclick={handleImportGpx}>Import GPX</button>
      <button onclick={handleImportPlt}>Import PLT</button>
    </div>
    <button class="full" onclick={() => tracksPanelOpen.update((v) => !v)}>
      {$tracksPanelOpen ? "Hide" : "Show"} Tracks Panel
    </button>
  </div>

  <div class="status-bar" class:error={$status.toLowerCase().includes("error") || $status.toLowerCase().includes("failed")}>
    <span>{$status}</span>
    <button
      class="console-toggle"
      onclick={() => consoleOpen.update((v) => !v)}
      title="Toggle console (backtick)"
    >›_</button>
  </div>
</aside>

<style>
  .sidebar {
    width: 240px;
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

  button.secondary {
    background: transparent;
    border-color: var(--ctp-surface2);
    color: var(--ctp-subtext1);
  }

  button.small {
    font-size: 11px;
    padding: 2px 6px;
  }

  select.full {
    width: 100%;
  }

  .map-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-height: 160px;
    overflow-y: auto;
  }

  .map-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 6px;
    text-align: left;
    font-size: 11px;
    padding: 3px 6px;
  }

  .map-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .badge {
    font-size: 10px;
    padding: 1px 4px;
    border-radius: 3px;
    flex-shrink: 0;
  }

  .badge.cached {
    background: var(--ctp-green);
    color: var(--ctp-base);
  }

  .badge.remote {
    background: var(--ctp-peach);
    color: var(--ctp-base);
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
    color: var(--ctp-text);
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

  .status-bar.error {
    color: var(--ctp-red);
  }

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

  .console-toggle:hover {
    color: var(--ctp-text);
  }
</style>
