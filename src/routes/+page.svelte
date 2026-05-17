<script lang="ts" module>
  // Module-level cold-start guard: the auto-redirect from `/` to `/project`
  // fires only the first time `/` mounts in a session. Subsequent visits to
  // `/` (e.g. via the workspace "Maps…" button) are intentional and must not
  // bounce the user back to `/project`.
  let initialRedirectChecked = false;
</script>

<script lang="ts">
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { listen } from "@tauri-apps/api/event";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import {
    activeDownloadId,
    activeMap,
    appState,
    appendProjectsChunk,
    busy,
    currentProject,
    downloadProgress,
    downloadingMaps,
    noteBundleFileReady,
    projects,
    projectsLoading,
    readyBundleFiles,
    resetBundleDownloadState,
    status,
    syncProjectsFromAppState,
    updateDownloadProgress,
  } from "../lib/stores";
  import {
    cancelDownload,
    loadProjects,
    loadProject,
    openSelectedMap,
    openLocalBundle,
    setBundlesRoot,
  } from "../lib/api";
  import { open } from "@tauri-apps/plugin-dialog";
  import type {
    BundleFileReadyPayload,
    BundleProgressPayload,
    DownloadProgressPayload,
    LizaProjectSummaryDto,
  } from "../lib/types";

  let projectFilter = $state("");
  let selectedSlug = $state("");
  let bundleProgress = $state<BundleProgressPayload | null>(null);
  let currentDownload = $state<DownloadProgressPayload | null>(null);

  $effect(() => {
    const s = $appState;
    syncProjectsFromAppState(s);
    if (s && !s.busy) {
      projectsLoading.set(false);
      bundleProgress = null;
    }
  });

  onMount(() => {
    let cancelled = false;
    const cleanups: Array<() => void> = [];

    (async () => {
      if (!initialRedirectChecked) {
        initialRedirectChecked = true;
        await appState.refresh();
        if (cancelled) return;
        if (get(activeMap)) {
          goto(resolve("/project"));
          return;
        }
      }

      const subscribers: Array<Promise<() => void>> = [
        listen<DownloadProgressPayload>("download-progress", (e) => {
          updateDownloadProgress(e.payload);
          currentDownload = e.payload;
        }),
        listen<BundleProgressPayload>("bundle-progress", (e) => {
          bundleProgress = e.payload;
        }),
        listen<BundleFileReadyPayload>("bundle-file-ready", (e) =>
          noteBundleFileReady(e.payload),
        ),
        listen<LizaProjectSummaryDto[]>("projects-chunk", (e) =>
          appendProjectsChunk(e.payload),
        ),
      ];

      const fns = await Promise.all(subscribers);
      if (cancelled) {
        fns.forEach((fn) => fn());
        return;
      }
      cleanups.push(...fns);

      loadProjects().catch(() => {});
    })();

    return () => {
      cancelled = true;
      cleanups.forEach((fn) => fn());
    };
  });

  const filtered = $derived(
    $projects.filter((p) =>
      p.name.toLowerCase().includes(projectFilter.toLowerCase()),
    ),
  );

  async function handleRefresh() {
    projectsLoading.set(true);
    await loadProjects();
  }

  function handleSelectProject(slug: string) {
    selectedSlug = slug;
    resetBundleDownloadState(null);
    currentDownload = null;
    loadProject(slug)
      .then((id) => {
        if (id) activeDownloadId.set(id);
      })
      .catch(() => {
        /* errors surface via diagnostics */
      });
  }

  async function handleOpenMap(mapName: string) {
    await openSelectedMap(mapName);
    if (get(activeMap)) goto(resolve("/project"));
  }

  async function handleOpenLocalBundle() {
    const dir = await open({ directory: true, multiple: false });
    if (dir) {
      resetBundleDownloadState(null);
      const id = await openLocalBundle(dir as string);
      if (id) activeDownloadId.set(id);
    }
  }

  async function handleCancelDownload() {
    const id = $activeDownloadId;
    if (!id) return;
    await cancelDownload(id);
    activeDownloadId.set(null);
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
      : null,
  );

  const currentFileLabel = $derived.by(() => {
    if (!currentDownload) return null;
    if (
      currentDownload.file_index == null ||
      currentDownload.file_count == null
    )
      return null;
    return `${currentDownload.file_index + 1} / ${currentDownload.file_count} — ${currentDownload.package_name}`;
  });

  // A bundle becomes openable as soon as at least one file is ready.
  const canOpenPartial = $derived($readyBundleFiles.length > 0 && $busy);

  async function handleOpenPartial() {
    await appState.refresh();
  }
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
      {#if currentFileLabel}
        <span class="current-file" data-testid="current-file-label">
          Downloading {currentFileLabel}
        </span>
      {/if}
      {#if currentDownload && currentDownload.total_bytes == null && currentDownload.downloaded_bytes > 0}
        <div class="bundle-track" data-testid="indeterminate-bar">
          <div class="bundle-fill indeterminate-bar"></div>
        </div>
      {/if}
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
      {#if $readyBundleFiles.length > 0}
        <div class="ready-files" data-testid="ready-files">
          {#each $readyBundleFiles as f (f.package_name)}
            <div class="ready-row">
              <span class="ready-tick">✓</span>
              <span class="ready-name">{f.package_name}</span>
            </div>
          {/each}
        </div>
      {/if}
    </div>
    <div class="status-actions">
      {#if canOpenPartial}
        <button class="action-btn" data-testid="open-bundle-now" onclick={handleOpenPartial}>
          Open bundle now
        </button>
      {/if}
      {#if $activeDownloadId && $busy}
        <button
          class="action-btn"
          data-testid="cancel-download"
          onclick={handleCancelDownload}>Cancel</button>
      {/if}
    </div>
  </div>
{/if}

<style>
  .root {
    flex: 1;
    min-width: 0;
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
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
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
    z-index: 10;
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

  @keyframes indeterminate-bar {
    0% { margin-left: -40%; width: 40%; }
    100% { margin-left: 100%; width: 40%; }
  }

  .bundle-fill.indeterminate-bar {
    animation: indeterminate-bar 1.2s ease-in-out infinite;
    background: var(--ctp-blue);
  }

  .current-file {
    font-size: 10px;
    color: var(--ctp-subtext1);
    font-variant-numeric: tabular-nums;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .ready-files {
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-height: 80px;
    overflow-y: auto;
    margin-top: 2px;
  }

  .ready-row {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 10px;
    color: var(--ctp-subtext0);
  }

  .ready-tick {
    color: var(--ctp-green);
    font-weight: 700;
  }

  .ready-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .status-actions {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex-shrink: 0;
  }

  .action-btn {
    font-size: 11px;
    padding: 2px 8px;
    background: var(--ctp-surface1);
    color: var(--ctp-text);
    border: 1px solid var(--ctp-surface2);
    border-radius: 3px;
    cursor: pointer;
  }

  .action-btn:hover {
    background: var(--ctp-surface2);
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
