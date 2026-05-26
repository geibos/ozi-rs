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
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import {
    activeDownloadId,
    activeMap,
    appState,
    bundleProgress,
    busy,
    currentDownload,
    currentProject,
    downloadProgress,
    downloadingMaps,
    projects,
    projectsLoading,
    resetBundleDownloadState,
    status,
    syncProjectsFromAppState,
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
  import { toast } from "svelte-sonner";

  let projectFilter = $state("");
  // `debouncedProjectFilter` is the value the `filtered` derivation reads.
  // Fast typing keeps re-arming the timer (see effect below); the list
  // re-filters at most once per 150ms quiet window. 150ms sits inside the
  // perceptual-instantaneity envelope while comfortably absorbing a normal
  // typing burst. Spec floor is 120ms (`consolidate-state-event-flow`).
  let debouncedProjectFilter = $state("");
  let selectedSlug = $state("");
  let transientStatus = $state<string | null>(null);

  $effect(() => {
    const s = $appState;
    syncProjectsFromAppState(s);
    if (s && !s.busy) {
      projectsLoading.set(false);
      bundleProgress.set(null);
    }
  });

  // Debounce the filter input. The teardown clears any pending timer so a
  // late flush after navigation can't whack a fresh store value.
  $effect(() => {
    const value = projectFilter;
    const handle = setTimeout(() => {
      debouncedProjectFilter = value;
    }, 150);
    return () => clearTimeout(handle);
  });

  onMount(() => {
    // The layout owns every Tauri event listener and the startup
    // `loadProjects()` call (single-source rule, `consolidate-state-event-flow`).
    // The only page-level startup concern is the cold-start auto-redirect
    // into /project when a project is already open (e.g., after reload).
    if (initialRedirectChecked) return;
    initialRedirectChecked = true;
    let cancelled = false;

    (async () => {
      await appState.refresh();
      if (cancelled) return;
      if (get(activeMap)) {
        goto(resolve("/project"));
      }
    })();

    return () => {
      cancelled = true;
    };
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
    // Guard against repeated clicks on the same already-selected project:
    // without this, a double-click would issue an unnecessary cancel + restart.
    if (selectedSlug === slug && $activeDownloadId !== null) return;

    selectedSlug = slug;

    // Snapshot the previous id BEFORE any state mutation. We deliberately
    // do NOT reset `activeDownloadId` to null between cancel and restart —
    // a concurrent click during the IPC round-trip would otherwise read
    // null, skip cancel, and let the backend see overlapping downloads.
    // The id stays pointing to the cancelled download (the backend tolerates
    // a second cancel) until `loadProject` returns the new id.
    const previousId = $activeDownloadId;
    const isSwitching = previousId !== null;

    if (isSwitching) {
      const project = $projects.find((p) => p.slug === slug);
      transientStatus = `Switching to ${project?.name ?? slug}…`;
      try {
        await cancelDownload(previousId);
      } catch {
        // cancel failures should not block the switch
      }
    }

    // Clear the per-bundle progress UI without touching activeDownloadId.
    downloadProgress.set(new Map());
    currentDownload.set(null);

    try {
      const id = await loadProject(slug);
      activeDownloadId.set(id || null);
    } catch {
      activeDownloadId.set(null);
    } finally {
      transientStatus = null;
    }
  }

  async function handleOpenMap(mapName: string) {
    try {
      await openSelectedMap(mapName);
    } catch (error) {
      toast.error("Failed to open map", { description: String(error) });
      return;
    }
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
    $bundleProgress?.total
      ? Math.round((($bundleProgress.completed ?? 0) / $bundleProgress.total) * 100)
      : null,
  );

  const currentFileLabel = $derived.by(() => {
    const c = $currentDownload;
    if (!c) return null;
    if (c.file_index == null || c.file_count == null) return null;
    return `${c.file_index + 1} / ${c.file_count} — ${c.package_name}`;
  });

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

  <div class="status-bar" class:busy={$busy}>
    <div class="status-line-slot">
      {#if $busy || transientStatus}
        <span class="spinner"></span>
      {/if}
      <span class="status-text">{transientStatus ?? $bundleProgress?.message ?? $status ?? ""}</span>
    </div>

    <div class="current-file-slot">
      {#if currentFileLabel}
        <span class="current-file" data-testid="current-file-label">
          Downloading {currentFileLabel}
        </span>
      {/if}
    </div>

    <div class="progress-slot">
      {#if $currentDownload && $currentDownload.total_bytes == null && $currentDownload.downloaded_bytes > 0}
        <div class="bundle-track" data-testid="indeterminate-bar">
          <div class="bundle-fill indeterminate-bar"></div>
        </div>
      {:else if bundlePercent != null}
        <div class="bundle-track">
          <div class="bundle-fill" style={`width: ${bundlePercent}%`}></div>
        </div>
      {:else}
        <div class="bundle-track bundle-track-placeholder"></div>
      {/if}
    </div>

    <div class="meta-slot">
      {#if $bundleProgress?.total != null}
        <span>{$bundleProgress.completed ?? 0}/{$bundleProgress.total}</span>
      {/if}
      {#if $bundleProgress?.downloaded_bytes != null}
        <span>
          {formatBytes($bundleProgress.downloaded_bytes)}
          {$bundleProgress.total_bytes ? `/ ${formatBytes($bundleProgress.total_bytes)}` : ""}
        </span>
      {/if}
    </div>

    <div class="status-actions">
      {#if $activeDownloadId && $busy}
        <button
          class="action-btn"
          data-testid="cancel-download"
          onclick={handleCancelDownload}>Cancel</button>
      {/if}
    </div>
  </div>
</div>

<style>
  .root {
    --bundle-status-bar-height: 80px;
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

  .refresh-btn:hover { color: hsl(var(--foreground)); }

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

  .list-item:hover { background: hsl(var(--secondary)); }
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

  .map-item:hover { background: hsl(var(--secondary)); }
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

  /* Status badges: green = cached, orange = downloadable. These are semantic
   * status colours, not theme tokens — kept as flat OKLCH values so they
   * stay legible in both light and dark mode and don't compete with the
   * Teal accent. */
  .badge.green  { background: oklch(0.62 0.14 145); color: hsl(var(--background)); }
  .badge.orange { background: oklch(0.72 0.16 60);  color: hsl(var(--background)); }
  .badge.blue   {
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
    0%   { transform: translateX(-100%); width: 40%; }
    100% { transform: translateX(350%);  width: 40%; }
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

  .status-bar {
    flex-shrink: 0;
    height: var(--bundle-status-bar-height);
    display: grid;
    grid-template-columns: 1fr auto;
    grid-template-areas:
      "status-line   actions"
      "current-file  actions"
      "progress      actions"
      "meta          actions";
    grid-template-rows: 20px 16px 12px 16px;
    column-gap: 8px;
    padding: 6px 10px;
    font-size: 11px;
    color: hsl(var(--muted-foreground));
    background: hsl(var(--card));
    border-top: 1px solid hsl(var(--secondary));
    overflow: hidden;
  }

  .status-line-slot {
    grid-area: status-line;
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }

  .current-file-slot {
    grid-area: current-file;
    display: flex;
    align-items: center;
    min-width: 0;
  }

  .progress-slot {
    grid-area: progress;
    display: flex;
    align-items: center;
    min-width: 0;
  }

  .meta-slot {
    grid-area: meta;
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 10px;
    color: hsl(var(--muted-foreground));
    font-variant-numeric: tabular-nums;
    overflow: hidden;
    white-space: nowrap;
  }

  .status-text {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .bundle-track {
    width: 100%;
    height: 4px;
    border-radius: 999px;
    overflow: hidden;
    background: hsl(var(--secondary));
  }

  .bundle-track-placeholder {
    opacity: 0.5;
  }

  .bundle-fill {
    height: 100%;
    /* Bundle-progress fill: a single-hue Teal-derived gradient so we honour
     * the "no secondary accent" guardrail. Goes from full Teal-500 to a
     * slightly lighter shade so the bar still reads as motion. */
    background: linear-gradient(
      90deg,
      hsl(var(--primary)),
      oklch(from hsl(var(--primary)) calc(l + 0.08) c h)
    );
    transition: width 0.2s ease;
  }

  @keyframes indeterminate-bar {
    0% { margin-left: -40%; width: 40%; }
    100% { margin-left: 100%; width: 40%; }
  }

  .bundle-fill.indeterminate-bar {
    animation: indeterminate-bar 1.2s ease-in-out infinite;
    background: hsl(var(--primary));
  }

  .current-file {
    font-size: 10px;
    color: hsl(var(--muted-foreground));
    font-variant-numeric: tabular-nums;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .status-actions {
    grid-area: actions;
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex-shrink: 0;
    align-self: start;
  }

  .action-btn {
    font-size: 11px;
    padding: 2px 8px;
    background: hsl(var(--border));
    color: hsl(var(--foreground));
    border: 1px solid hsl(var(--muted));
    border-radius: 3px;
    cursor: pointer;
  }

  .action-btn:hover {
    background: hsl(var(--muted));
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
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
