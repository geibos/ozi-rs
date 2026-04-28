<script lang="ts">
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { listen } from "@tauri-apps/api/event";
  import {
    appState,
    appendProjectsChunk,
    busy,
    currentProject,
    downloadProgress,
    downloadingMaps,
    projects,
    projectsLoading,
    status,
    syncProjectsFromAppState,
    updateDownloadProgress,
  } from "../lib/stores";
  import {
    loadProjects,
    loadProject,
    openSelectedMap,
    openLocalBundle,
    setBundlesRoot,
  } from "../lib/api";
  import { open } from "@tauri-apps/plugin-dialog";
  import type {
    BundleProgressPayload,
    DownloadProgressPayload,
    LizaProjectSummaryDto,
  } from "../lib/types";

  let projectFilter = $state("");
  let selectedSlug = $state("");
  let bundleProgress = $state<BundleProgressPayload | null>(null);
  let refreshTimer: number | null = null;

  async function refreshState() {
    await appState.refresh();
    const latest = get(appState);
    syncProjectsFromAppState(latest);
    if (latest && !latest.busy) {
      projectsLoading.set(false);
      bundleProgress = null;
    }
  }

  function scheduleRefresh() {
    if (refreshTimer !== null) return;
    refreshTimer = window.setTimeout(async () => {
      refreshTimer = null;
      await refreshState();
    }, 120);
  }

  onMount(async () => {
    // Hide instead of close so the window can be re-shown instantly
    const { getCurrentWebviewWindow } = await import("@tauri-apps/api/webviewWindow");
    const currentWindow = getCurrentWebviewWindow();
    const unlistenClose = await currentWindow.onCloseRequested(async (event) => {
      event.preventDefault();
      await currentWindow.hide();
    });

    // Set up listeners BEFORE any async work so we don't miss events
    const unlisten = await listen<void>("state-changed", scheduleRefresh);
    const unlistenProgress = await listen<DownloadProgressPayload>(
      "download-progress",
      (e) => updateDownloadProgress(e.payload)
    );
    const unlistenBundleProgress = await listen<BundleProgressPayload>(
      "bundle-progress",
      (e) => {
        bundleProgress = e.payload;
      }
    );
    const unlistenProjectsChunk = await listen<LizaProjectSummaryDto[]>(
      "projects-chunk",
      (e) => appendProjectsChunk(e.payload)
    );

    // Show cached state immediately, then trigger a fresh load
    await refreshState();
    loadProjects().catch(() => {});

    return () => {
      if (refreshTimer !== null) {
        window.clearTimeout(refreshTimer);
      }
      unlisten();
      unlistenProgress();
      unlistenBundleProgress();
      unlistenProjectsChunk();
      unlistenClose();
    };
  });

  const filtered = $derived(
    $projects.filter((p) =>
      p.name.toLowerCase().includes(projectFilter.toLowerCase())
    )
  );

  async function handleRefresh() {
    projectsLoading.set(true);
    await loadProjects();
  }

  async function handleSelectProject(slug: string) {
    selectedSlug = slug;
    await loadProject(slug);
  }

  async function handleOpenMap(mapName: string) {
    await openSelectedMap(mapName);
  }

  async function handleOpenLocalBundle() {
    const dir = await open({ directory: true, multiple: false });
    if (dir) await openLocalBundle(dir as string);
  }

  async function handleSetBundlesRoot() {
    const dir = await open({ directory: true, multiple: false });
    if (dir) await setBundlesRoot(dir as string);
  }

  function formatBytes(bytes: number): string {
    if (bytes >= 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MiB`;
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(0)} KiB`;
    return `${bytes} B`;
  }

  const bundlePercent = $derived(
    bundleProgress?.total
      ? Math.round(((bundleProgress.completed ?? 0) / bundleProgress.total) * 100)
      : null
  );
</script>

<div class="root">
<div class="loader">
  <!-- Projects column -->
  <div class="col">
    <div class="col-header">
      <span>Projects</span>
      <button onclick={handleRefresh} disabled={$busy} class="refresh-btn">
        {$busy ? "…" : "↻"}
      </button>
    </div>

    <div class="filter-row">
      <input
        class="filter-input"
        type="search"
        placeholder="Filter…"
        bind:value={projectFilter}
      />
      <span class="filter-count">
        ({$projects.length})
        {#if $projectsLoading}
          <span class="spinner"></span>
        {/if}
      </span>
    </div>

    <div class="list">
      {#each filtered as p (p.slug)}
        <button
          class="list-item"
          class:active={selectedSlug === p.slug}
          onclick={() => handleSelectProject(p.slug)}
          disabled={$busy}
        >{p.name}</button>
      {:else}
        <div class="empty">No matches</div>
      {/each}
    </div>

    <div class="col-footer">
      <button onclick={handleOpenLocalBundle} class="footer-btn">Open local bundle…</button>
      <button onclick={handleSetBundlesRoot} class="footer-btn muted">Set bundles root…</button>
    </div>
  </div>

  <!-- Maps column -->
  <div class="col">
    <div class="col-header">
      <span>Maps</span>
      {#if $currentProject}
        <span class="project-name">{$currentProject.name}</span>
      {/if}
    </div>

    <div class="list">
      {#if $currentProject}
        {#each $currentProject.maps as m (m.name)}
          {@const isDownloading = $downloadingMaps.has(m.name)}
          {@const prog = $downloadProgress.get(m.name)}
          {@const pct = prog?.total_bytes
            ? Math.round((prog.downloaded_bytes / prog.total_bytes) * 100)
            : null}
          <button
            class="map-item"
            class:is-downloading={isDownloading}
            onclick={() => handleOpenMap(m.name)}
            disabled={isDownloading}
          >
            <div class="map-row">
              <span class="map-name">{m.name}</span>
              {#if isDownloading}
                <span class="badge blue">
                  {pct != null ? `${pct}%` : "…"}
                </span>
              {:else if m.downloaded}
                <span class="badge green">cached</span>
              {:else}
                <span class="badge orange">↓</span>
              {/if}
            </div>

            {#if isDownloading && prog}
              <div class="prog-row">
                <div class="prog-track">
                  <div
                    class="prog-fill"
                    class:indeterminate={pct == null}
                    style={pct != null ? `width: ${pct}%` : ""}
                  ></div>
                </div>
                <span class="prog-label">
                  {formatBytes(prog.downloaded_bytes)}
                  {prog.total_bytes ? `/ ${formatBytes(prog.total_bytes)}` : ""}
                </span>
              </div>
            {/if}
          </button>
        {/each}
      {:else}
        <div class="empty">Select a project on the left</div>
      {/if}
    </div>
  </div>
</div>
</div>

{#if $busy || $status}
  <div class="status-bar" class:busy={$busy}>
    {#if $busy}
      <span class="spinner"></span>
    {/if}
    <div class="status-main">
      <span class="status-text">{bundleProgress?.message ?? $status}</span>
      {#if bundleProgress}
        <div class="bundle-meta">
          {#if bundleProgress.total != null}
            <span>{bundleProgress.completed ?? 0}/{bundleProgress.total}</span>
          {/if}
          {#if bundleProgress.downloaded_bytes != null}
            <span>
              {formatBytes(bundleProgress.downloaded_bytes)}
              {bundleProgress.total_bytes ? `/ ${formatBytes(bundleProgress.total_bytes)}` : ""}
            </span>
          {/if}
        </div>
        {#if bundlePercent != null}
          <div class="bundle-track">
            <div class="bundle-fill" style={`width: ${bundlePercent}%`}></div>
          </div>
        {/if}
      {/if}
    </div>
  </div>
{/if}

<style>
  .root {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .loader {
    display: flex;
    flex: 1;
    min-height: 0;
    background: var(--ctp-base);
    gap: 1px;
    background-color: var(--ctp-surface0);
  }

  .col {
    flex: 1;
    display: flex;
    flex-direction: column;
    background: var(--ctp-base);
    min-width: 0;
  }

  .col-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 10px 6px;
    border-bottom: 1px solid var(--ctp-surface0);
    font-size: 11px;
    font-weight: 600;
    color: var(--ctp-subtext1);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    flex-shrink: 0;
  }

  .project-name {
    font-weight: 400;
    text-transform: none;
    letter-spacing: 0;
    font-size: 11px;
    color: var(--ctp-overlay1);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 140px;
  }

  .refresh-btn {
    background: none;
    border: none;
    color: var(--ctp-overlay1);
    font-size: 14px;
    padding: 0 2px;
    cursor: pointer;
    line-height: 1;
  }

  .refresh-btn:hover { color: var(--ctp-text); }

  .filter-input {
    width: 100%;
    font-size: 12px;
    padding: 4px 8px;
    background: var(--ctp-mantle);
    border: 1px solid var(--ctp-surface1);
    border-radius: 4px;
    color: var(--ctp-text);
    flex-shrink: 0;
  }

  .filter-row {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 6px 8px;
  }

  .filter-count {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--ctp-overlay1);
    flex-shrink: 0;
  }

  .list {
    flex: 1;
    overflow-y: auto;
    padding: 2px 0;
  }

  .list-item {
    display: block;
    width: 100%;
    text-align: left;
    padding: 6px 12px;
    font-size: 12px;
    background: none;
    border: none;
    border-radius: 0;
    color: var(--ctp-text);
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .list-item:hover { background: var(--ctp-surface0); }
  .list-item.active {
    background: var(--ctp-surface1);
    color: var(--ctp-blue);
    font-weight: 500;
  }

  .map-item {
    display: flex;
    flex-direction: column;
    width: 100%;
    text-align: left;
    padding: 6px 12px;
    background: none;
    border: none;
    cursor: pointer;
    gap: 5px;
  }

  .map-item:hover { background: var(--ctp-surface0); }
  .map-item.is-downloading { opacity: 0.75; cursor: default; }

  .map-row {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .map-name {
    flex: 1;
    font-size: 12px;
    color: var(--ctp-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .badge {
    flex-shrink: 0;
    font-size: 10px;
    padding: 1px 5px;
    border-radius: 3px;
  }

  .badge.green  { background: var(--ctp-green);  color: var(--ctp-base); }
  .badge.orange { background: var(--ctp-peach);  color: var(--ctp-base); }
  .badge.blue   {
    background: var(--ctp-blue);
    color: var(--ctp-base);
    font-variant-numeric: tabular-nums;
    min-width: 32px;
    text-align: center;
  }

  .prog-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .prog-track {
    flex: 1;
    height: 3px;
    background: var(--ctp-surface1);
    border-radius: 2px;
    overflow: hidden;
  }

  .prog-fill {
    height: 100%;
    background: var(--ctp-blue);
    border-radius: 2px;
    transition: width 0.25s ease;
  }

  @keyframes indeterminate {
    0%   { transform: translateX(-100%); width: 40%; }
    100% { transform: translateX(350%);  width: 40%; }
  }

  .prog-fill.indeterminate {
    width: 40% !important;
    animation: indeterminate 1.2s ease-in-out infinite;
  }

  .prog-label {
    font-size: 10px;
    color: var(--ctp-overlay1);
    white-space: nowrap;
    font-variant-numeric: tabular-nums;
  }

  .col-footer {
    padding: 8px;
    border-top: 1px solid var(--ctp-surface0);
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex-shrink: 0;
  }

  .footer-btn {
    width: 100%;
    font-size: 12px;
    text-align: left;
    padding: 4px 8px;
  }

  .footer-btn.muted {
    background: transparent;
    border-color: var(--ctp-surface2);
    color: var(--ctp-subtext1);
  }

  .empty {
    padding: 16px 12px;
    font-size: 12px;
    color: var(--ctp-overlay1);
    text-align: center;
  }

  .status-bar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    font-size: 11px;
    color: var(--ctp-subtext0);
    background: var(--ctp-crust);
    border-top: 1px solid var(--ctp-surface0);
    min-height: 24px;
    overflow: hidden;
  }

  .status-main {
    display: flex;
    flex: 1;
    min-width: 0;
    flex-direction: column;
    gap: 4px;
  }

  .status-text {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .bundle-meta {
    display: flex;
    gap: 10px;
    font-size: 10px;
    color: var(--ctp-overlay1);
    font-variant-numeric: tabular-nums;
  }

  .bundle-track {
    width: 100%;
    height: 4px;
    border-radius: 999px;
    overflow: hidden;
    background: var(--ctp-surface0);
  }

  .bundle-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--ctp-blue), var(--ctp-teal));
    transition: width 0.2s ease;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .spinner {
    display: inline-block;
    width: 10px;
    height: 10px;
    border: 2px solid var(--ctp-surface2);
    border-top-color: var(--ctp-blue);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    flex-shrink: 0;
  }
</style>
