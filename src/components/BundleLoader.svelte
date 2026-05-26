<script lang="ts">
  /**
   * Bundle loader component — the catalog projects column + per-project
   * maps column. Used in two surfaces:
   *
   *   1. Cold-start `/` route — full-window when no active map is loaded.
   *   2. Workspace `/project` route — mounted inside a right-side `Sheet`
   *      so the user can switch maps without leaving the workspace.
   *
   * The component owns the filter input + selection state; it does NOT
   * own the status bar (the cold-start surface and the workspace status
   * bar each have their own). The Tauri event listeners that populate
   * `appState`, `bundleProgress`, `downloadProgress`, and the catalog
   * stream live in `+layout.svelte` (single-owner rule).
   */
  import { get } from "svelte/store";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import {
    activeDownloadId,
    activeMap,
    busy,
    currentDownload,
    currentProject,
    downloadProgress,
    downloadingMaps,
    projects,
    projectsLoading,
    resetBundleDownloadState,
  } from "../lib/stores";
  import {
    cancelDownload,
    loadProjects,
    loadProject,
    openLocalBundle,
    openSelectedMap,
    setBundlesRoot,
  } from "../lib/api";
  import { open } from "@tauri-apps/plugin-dialog";
  import { toast } from "svelte-sonner";

  let {
    onCloseRequest,
  }: {
    onCloseRequest?: () => void;
  } = $props();

  let projectFilter = $state("");
  let debouncedProjectFilter = $state("");
  let selectedSlug = $state("");

  $effect(() => {
    const value = projectFilter;
    const handle = setTimeout(() => {
      debouncedProjectFilter = value;
    }, 150);
    return () => clearTimeout(handle);
  });

  const filtered = $derived(
    $projects.filter((p) =>
      p.name.toLowerCase().includes(debouncedProjectFilter.toLowerCase()),
    ),
  );

  async function handleRefresh() {
    projectsLoading.set(true);
    await loadProjects();
  }

  async function handleSelectProject(slug: string) {
    if (selectedSlug === slug && $activeDownloadId !== null) return;
    selectedSlug = slug;
    const previousId = $activeDownloadId;
    const isSwitching = previousId !== null;

    if (isSwitching) {
      try {
        await cancelDownload(previousId);
      } catch {
        // cancel failures should not block the switch
      }
    }

    downloadProgress.set(new Map());
    currentDownload.set(null);

    try {
      const id = await loadProject(slug);
      activeDownloadId.set(id || null);
    } catch {
      activeDownloadId.set(null);
    }
  }

  async function handleOpenMap(mapName: string) {
    try {
      await openSelectedMap(mapName);
    } catch (error) {
      toast.error("Failed to open map", { description: String(error) });
      return;
    }
    if (get(activeMap)) {
      if (onCloseRequest) {
        onCloseRequest();
      } else {
        goto(resolve("/project"));
      }
    }
  }

  async function handleOpenLocalBundle() {
    const dir = await open({ directory: true, multiple: false });
    if (dir) {
      resetBundleDownloadState(null);
      const id = await openLocalBundle(dir as string);
      if (id) activeDownloadId.set(id);
    }
  }

  async function handleSetBundlesRoot() {
    const dir = await open({ directory: true, multiple: false });
    if (dir) await setBundlesRoot(dir as string);
  }

  function formatBytes(bytes: number): string {
    if (bytes >= 1024 * 1024)
      return `${(bytes / 1024 / 1024).toFixed(1)} MiB`;
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(0)} KiB`;
    return `${bytes} B`;
  }
</script>

<div class="loader">
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
          onclick={() => handleSelectProject(p.slug)}>{p.name}</button
        >
      {:else}
        <div class="empty">No matches</div>
      {/each}
    </div>

    <div class="col-footer">
      <button onclick={handleOpenLocalBundle} class="footer-btn">
        Open local bundle…
      </button>
      <button onclick={handleSetBundlesRoot} class="footer-btn muted">
        Set bundles root…
      </button>
    </div>
  </div>

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
                  {prog.total_bytes
                    ? `/ ${formatBytes(prog.total_bytes)}`
                    : ""}
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

<style>
  .loader {
    display: flex;
    flex: 1;
    min-height: 0;
    height: 100%;
    background: hsl(var(--background));
    gap: 1px;
    background-color: hsl(var(--secondary));
  }

  .col {
    flex: 1;
    display: flex;
    flex-direction: column;
    background: hsl(var(--background));
    min-width: 0;
  }

  .col-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 10px 6px;
    border-bottom: 1px solid hsl(var(--secondary));
    font-size: 11px;
    font-weight: 600;
    color: hsl(var(--muted-foreground));
    text-transform: uppercase;
    letter-spacing: 0.05em;
    flex-shrink: 0;
  }

  .project-name {
    font-weight: 400;
    text-transform: none;
    letter-spacing: 0;
    font-size: 11px;
    color: hsl(var(--muted-foreground));
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 140px;
  }

  .refresh-btn {
    background: none;
    border: none;
    color: hsl(var(--muted-foreground));
    font-size: 14px;
    padding: 0 2px;
    cursor: pointer;
    line-height: 1;
  }

  .refresh-btn:hover {
    color: hsl(var(--foreground));
  }

  .filter-input {
    width: 100%;
    font-size: 12px;
    padding: 4px 8px;
    background: hsl(var(--card));
    border: 1px solid hsl(var(--border));
    border-radius: 4px;
    color: hsl(var(--foreground));
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
    color: hsl(var(--muted-foreground));
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
    color: hsl(var(--foreground));
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .list-item:hover {
    background: hsl(var(--secondary));
  }
  .list-item.active {
    background: hsl(var(--border));
    color: hsl(var(--primary));
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

  .map-item:hover {
    background: hsl(var(--secondary));
  }
  .map-item.is-downloading {
    opacity: 0.75;
    cursor: default;
  }

  .map-row {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .map-name {
    flex: 1;
    font-size: 12px;
    color: hsl(var(--foreground));
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

  .badge.green {
    background: oklch(0.62 0.14 145);
    color: hsl(var(--background));
  }
  .badge.orange {
    background: oklch(0.72 0.16 60);
    color: hsl(var(--background));
  }
  .badge.blue {
    background: hsl(var(--primary));
    color: hsl(var(--background));
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
    background: hsl(var(--border));
    border-radius: 2px;
    overflow: hidden;
  }

  .prog-fill {
    height: 100%;
    background: hsl(var(--primary));
    border-radius: 2px;
    transition: width 0.25s ease;
  }

  @keyframes indeterminate {
    0% {
      transform: translateX(-100%);
      width: 40%;
    }
    100% {
      transform: translateX(350%);
      width: 40%;
    }
  }

  .prog-fill.indeterminate {
    width: 40% !important;
    animation: indeterminate 1.2s ease-in-out infinite;
  }

  .prog-label {
    font-size: 10px;
    color: hsl(var(--muted-foreground));
    white-space: nowrap;
    font-variant-numeric: tabular-nums;
  }

  .col-footer {
    padding: 8px;
    border-top: 1px solid hsl(var(--secondary));
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
    border-color: hsl(var(--muted));
    color: hsl(var(--muted-foreground));
  }

  .empty {
    padding: 16px 12px;
    font-size: 12px;
    color: hsl(var(--muted-foreground));
    text-align: center;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .spinner {
    display: inline-block;
    width: 10px;
    height: 10px;
    border: 2px solid hsl(var(--muted));
    border-top-color: hsl(var(--primary));
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    flex-shrink: 0;
  }
</style>
