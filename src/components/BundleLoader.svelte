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
   *
   * Interaction contract (owner feedback, 2026-07):
   *   - Row click = PREVIEW only (`previewProject`): fetches the bundle's
   *     map list without downloading anything.
   *   - The explicit "Open bundle (download)" button in the maps column
   *     starts the real download (`loadProject`).
   *   - The projects column is a manual virtual list — the catalog holds
   *     ~13k rows and a flat list of that many <button>s makes WebKit
   *     drop clicks and blank out on scroll.
   */
  import { get } from "svelte/store";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import {
    activeDownloadId,
    activeMap,
    catalogueError,
    bundleLoaderOpen,
    bundleLoaderPreselect,
    busy,
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
    previewProject,
    setBundlesRoot,
  } from "../lib/api";
  import { locale, t } from "../lib/i18n";
  import { open } from "@tauri-apps/plugin-dialog";
  import { toast } from "svelte-sonner";
  import { filterProjects } from "$lib/project-list";
  import { formatBytes, formatOptionalBytes } from "$lib/format-bytes";
  import { appendRecentFile } from "../lib/recentFiles";

  let {
    onCloseRequest,
  }: {
    onCloseRequest?: () => void;
  } = $props();

  let projectFilter = $state("");
  let debouncedProjectFilter = $state("");
  let selectedSlug = $state("");
  // Offline this is the only list that means anything: the catalogue is
  // thirteen thousand rows and a crew without a signal can open the handful
  // that are already on disk.
  let onlyCached = $state(false);
  /**
   * Top-level bundle entries the operator cleared.
   *
   * Nothing is skipped by default: what a crew can spare is their call, not
   * this code's. Print sheets and Android tile packs are usually most of the
   * weight and this app opens neither, which is what the hint says.
   */
  let skipped = $state<Record<string, true>>({});

  $effect(() => {
    const value = projectFilter;
    const handle = setTimeout(() => {
      debouncedProjectFilter = value;
    }, 150);
    return () => clearTimeout(handle);
  });

  const filtered = $derived(
    filterProjects($projects, debouncedProjectFilter, onlyCached),
  );

  // ── Manual virtual list over `filtered` ─────────────────────────────
  // Fixed row height (must match `.list-item.virtual-row` in the styles
  // below), spacer div of totalHeight, only the visible range ±OVERSCAN
  // rendered as absolutely-positioned rows. Scroll recompute is
  // rAF-throttled; the window resets to the top on every filter change.
  const ROW_HEIGHT = 28;
  const OVERSCAN = 15;

  let listEl = $state<HTMLDivElement | null>(null);
  let scrollTop = $state(0);
  let viewportHeight = $state(0);
  let scrollRaf = 0;

  function handleListScroll() {
    if (scrollRaf !== 0) return;
    scrollRaf = requestAnimationFrame(() => {
      scrollRaf = 0;
      scrollTop = listEl?.scrollTop ?? 0;
    });
  }

  $effect(() => {
    // Reset the scroll window whenever the filtered set changes — a stale
    // offset past the new (shorter) list would render nothing.
    void debouncedProjectFilter;
    void onlyCached;
    scrollTop = 0;
    if (listEl) listEl.scrollTop = 0;
  });

  const totalHeight = $derived(filtered.length * ROW_HEIGHT);
  const startIndex = $derived(
    Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN),
  );
  const endIndex = $derived(
    Math.min(
      filtered.length,
      Math.ceil((scrollTop + viewportHeight) / ROW_HEIGHT) + OVERSCAN,
    ),
  );
  const visibleRows = $derived(
    filtered.slice(startIndex, endIndex).map((project, i) => ({
      project,
      top: (startIndex + i) * ROW_HEIGHT,
    })),
  );

  async function handleRefresh() {
    projectsLoading.set(true);
    await loadProjects();
  }

  // ── Preview on row click ────────────────────────────────────────────
  // `previewProject` fetches the map list only; the pending hint clears
  // when the previewed project lands in `currentProject` (state-changed
  // round-trip) or after a 15 s timeout, whichever comes first.
  const PREVIEW_TIMEOUT_MS = 15_000;
  let previewPendingName = $state<string | null>(null);
  let previewTimer: ReturnType<typeof setTimeout> | null = null;

  function clearPreviewPending() {
    previewPendingName = null;
    if (previewTimer !== null) {
      clearTimeout(previewTimer);
      previewTimer = null;
    }
  }

  async function handleSelectProject(slug: string) {
    selectedSlug = slug;
    // A different bundle has different contents; carrying the previous
    // choice over would silently skip a folder of the new one.
    skipped = {};
    const name = $projects.find((p) => p.slug === slug)?.name ?? slug;
    previewPendingName = name;
    if (previewTimer !== null) clearTimeout(previewTimer);
    previewTimer = setTimeout(() => {
      previewTimer = null;
      previewPendingName = null;
    }, PREVIEW_TIMEOUT_MS);
    try {
      await previewProject(slug);
    } catch (error) {
      clearPreviewPending();
      toast.error($t("loader.previewFailed"), { description: String(error) });
    }
  }

  /**
   * Honour a project chosen elsewhere (the command palette's "Switch
   * project"): select it, preview it, and scroll it into view. Consumed once
   * — a stale slug must not re-select on the next open.
   */
  $effect(() => {
    const slug = $bundleLoaderPreselect;
    if (slug === null) return;
    const known = $projects.find((p) => p.slug === slug);
    if (!known) return;
    bundleLoaderPreselect.set(null);
    // Filtering to the name puts the row at the top of the list; scrolling a
    // virtualized 13k-row list to an index would fight the debounced filter.
    projectFilter = known.name;
    void handleSelectProject(slug);
  });

  $effect(() => {
    if (
      previewPendingName !== null &&
      $currentProject?.name === previewPendingName
    ) {
      clearPreviewPending();
    }
  });

  $effect(() => {
    return () => {
      if (scrollRaf !== 0) cancelAnimationFrame(scrollRaf);
      if (previewTimer !== null) clearTimeout(previewTimer);
    };
  });

  // ── Explicit bundle download ────────────────────────────────────────
  // Cancels any previous download before starting the new one (the
  // backend never sees overlapping downloads), resets the transient
  // progress stores, then records the new download_id.
  async function handleOpenBundle() {
    // Falling back to the open project's slug: the loader is also reached
    // from the workspace ("Открыть проект…"), where a project is already
    // previewed and no row has been clicked. Without the fallback the only
    // download button in the app did nothing at all on that path — silently,
    // because the handler returned before it called anything.
    const slug = selectedSlug || ($currentProject?.slug ?? "");
    if (!slug) return;
    const previousId = $activeDownloadId;
    if (previousId !== null) {
      try {
        await cancelDownload(previousId);
      } catch {
        // cancel failures should not block the restart
      }
    }
    resetBundleDownloadState(null);
    try {
      const id = await loadProject(slug, Object.keys(skipped));
      activeDownloadId.set(id || null);
    } catch (error) {
      activeDownloadId.set(null);
      // The backend refuses while the catalogue walk holds the busy flag.
      // Saying "wait" is the difference between a slow app and a dead button.
      const busyRefusal = String(error).includes("busy:");
      toast.error(
        busyRefusal
          ? $t("loader.openBundleBusy")
          : $t("loader.openBundleFailed"),
        busyRefusal ? undefined : { description: String(error) },
      );
    }
  }

  async function handleOpenMap(mapName: string) {
    try {
      const downloadId = await openSelectedMap(mapName);
      if (downloadId) {
        // The map was not on disk: show its progress and let it be cancelled,
        // the same as a whole-bundle download.
        resetBundleDownloadState(downloadId);
        return;
      }
    } catch (error) {
      toast.error($t("loader.openMapFailed"), { description: String(error) });
      return;
    }
    const am = get(activeMap);
    if (am) {
      // Append to the Cmd-K palette's Recent files list (silent on failure;
      // pure UX convenience per `redesign-inspector-pane`).
      appendRecentFile({
        projectSlug: am.project_name,
        mapPath: am.local_path,
        mapName: am.package_name,
        openedAt: Date.now(),
      });
      // The "open the loader" flag is set by callers that navigate to `/`
      // (Maps tab, command palette). `/` does not own the Sheet, so without
      // clearing it here the loader reopened over the map just opened.
      bundleLoaderOpen.set(false);
      if (onCloseRequest) {
        onCloseRequest();
      } else {
        goto(resolve("/project"));
      }
    }
  }

  async function handleOpenLocalBundle() {
    const dir = await open({ directory: true, multiple: false });
    if (!dir) return;
    resetBundleDownloadState(null);
    try {
      const id = await openLocalBundle(dir as string);
      if (id) activeDownloadId.set(id);
    } catch (error) {
      toast.error($t("loader.openLocalBundleFailed"), {
        description: String(error),
      });
    }
  }

  async function handleSetBundlesRoot() {
    const dir = await open({ directory: true, multiple: false });
    if (dir) await setBundlesRoot(dir as string);
  }
</script>

<div class="loader">
  <div class="col">
    <div class="col-header">
      <span>{$t("loader.projects")}</span>
      <button onclick={handleRefresh} disabled={$busy} class="refresh-btn">
        {$busy ? "…" : "↻"}
      </button>
    </div>

    <div class="filter-row">
      <input
        class="filter-input"
        type="search"
        placeholder={$t("loader.filterPlaceholder")}
        bind:value={projectFilter}
      />
    </div>

    <div class="filter-row filter-row-secondary">
      <button
        class="only-cached-btn"
        class:on={onlyCached}
        aria-pressed={onlyCached}
        title={onlyCached ? $t("loader.onlyCachedOn") : $t("loader.onlyCached")}
        onclick={() => (onlyCached = !onlyCached)}
        data-testid="only-cached"
      >
        {$t("loader.onlyCached")}
      </button>
      <span class="filter-count" data-testid="project-count">
        {$t("loader.filterCount")
          .replace("{shown}", String(filtered.length))
          .replace("{total}", String($projects.length))}
      </span>
    </div>

    {#if $projectsLoading}
      <div class="refresh-hint" data-testid="catalog-refreshing">
        <span class="spinner"></span>
        {$t("loader.refreshing")}
      </div>
    {:else if $catalogueError !== null}
      <!-- The cached list is the right answer offline, but a crew has to know
           that is what they are reading: a bundle made today would not be in
           it, and a missing row would otherwise read as "no such search". -->
      <div class="refresh-hint offline" data-testid="catalog-offline">
        {$t("loader.offline")}
      </div>
    {/if}

    <div
      class="list virtual-list"
      data-testid="project-virtual-list"
      bind:this={listEl}
      bind:clientHeight={viewportHeight}
      onscroll={handleListScroll}
    >
      {#if filtered.length === 0}
        <div class="empty">{$t("loader.noMatches")}</div>
      {:else}
        <div class="virtual-spacer" style={`height: ${totalHeight}px`}>
          {#each visibleRows as row (row.project.slug)}
            <button
              class="virtual-row list-item"
              class:active={selectedSlug === row.project.slug}
              style={`top: ${row.top}px`}
              onclick={() => handleSelectProject(row.project.slug)}
            >
              <span class="row-name">{row.project.name}</span>
              {#if row.project.cached}
                <span
                  class="row-cached"
                  title={$t("loader.cachedProject")}
                  data-testid="project-cached-badge"
                  >{$t("loader.cachedBadge")}</span
                >
              {/if}
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <div class="col-footer">
      <button onclick={handleOpenLocalBundle} class="footer-btn">
        {$t("loader.openLocalBundle")}
      </button>
      <button onclick={handleSetBundlesRoot} class="footer-btn muted">
        {$t("loader.setBundlesRoot")}
      </button>
    </div>
  </div>

  <div class="col">
    <div class="col-header">
      <span>{$t("loader.maps")}</span>
      {#if $currentProject}
        <span class="project-name">{$currentProject.name}</span>
      {/if}
    </div>

    {#if $currentProject && !previewPendingName && $currentProject.contents.length > 0}
      <div class="contents-box" data-testid="bundle-contents">
        <div class="contents-title">{$t("loader.contents")}</div>
        {#each $currentProject.contents as entry (entry.name)}
          <label class="contents-row">
            <input
              type="checkbox"
              checked={skipped[entry.name] !== true}
              onchange={(e) => {
                const next = { ...skipped };
                if (e.currentTarget.checked) {
                  delete next[entry.name];
                } else {
                  next[entry.name] = true;
                }
                skipped = next;
              }}
            />
            <span class="contents-name" title={entry.name}>{entry.name}</span>
            <span class="contents-size">
              {entry.is_dir
                ? $t("loader.folder")
                : (formatOptionalBytes(entry.size_bytes, $locale) ?? "")}
            </span>
          </label>
        {/each}
        <p class="contents-hint">{$t("loader.contentsHint")}</p>
      </div>
    {/if}

    {#if $currentProject && !previewPendingName}
      <div class="maps-actions">
        <button
          class="open-bundle-btn"
          data-testid="open-bundle"
          onclick={handleOpenBundle}
          disabled={$busy}
          title={$busy ? $t("loader.openBundleBusy") : undefined}
        >
          {$busy ? $t("loader.openBundleBusy") : $t("loader.openBundle")}
        </button>
      </div>
    {/if}

    <div class="list">
      {#if previewPendingName}
        <div class="empty pending" data-testid="maps-pending">
          <span class="spinner"></span>
          {$t("loader.loadingMaps")}
        </div>
      {:else if $currentProject}
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
                <span class="badge green">{$t("loader.cachedBadge")}</span>
              {:else}
                <span class="badge orange">↓</span>
              {/if}
            </div>

            {#if !isDownloading && m.size_bytes != null}
              <!-- Its own line: the column is ~240px and a map name fills it,
                   so a size beside the name was cut off at the edge. This is
                   the number that decides whether the download is worth doing
                   on a phone tether — fifteen megabytes or two hundred. -->
              <span class="map-size" data-testid="map-size">
                {formatOptionalBytes(m.size_bytes, $locale)}
              </span>
            {/if}

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
                  {formatBytes(prog.downloaded_bytes, $locale)}
                  {prog.total_bytes
                    ? `/ ${formatBytes(prog.total_bytes, $locale)}`
                    : ""}
                </span>
              </div>
            {/if}
          </button>
        {/each}
      {:else}
        <div class="empty">{$t("loader.selectProject")}</div>
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

  /* `flex-shrink: 0` plus `width: 100%` pushed the downloaded filter and the
     count out of a 280px rail entirely. The input takes what is left after
     them instead. */
  .filter-input {
    min-width: 0;
    flex: 1;
    font-size: 12px;
    padding: 4px 8px;
    background: hsl(var(--card));
    border: 1px solid hsl(var(--border));
    border-radius: 4px;
    color: hsl(var(--foreground));
  }

  .filter-row {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 6px 8px;
  }

  /* Second line: the downloaded filter and the match count. On one line with
     the input they had no room in a 280px rail. */
  .filter-row-secondary {
    margin-top: 0;
    justify-content: space-between;
  }

  .filter-count {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: hsl(var(--muted-foreground));
    flex-shrink: 0;
  }

  .refresh-hint.offline {
    color: hsl(var(--destructive));
  }

  .refresh-hint {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 0 10px 4px;
    font-size: 10px;
    color: hsl(var(--muted-foreground));
    flex-shrink: 0;
  }

  .list {
    flex: 1;
    overflow-y: auto;
    padding: 2px 0;
  }

  .virtual-list {
    position: relative;
    padding: 0;
  }

  .virtual-spacer {
    position: relative;
    width: 100%;
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

  /* Fixed-height absolutely-positioned virtual row. The 28px height must
     stay in sync with ROW_HEIGHT in the script block. */
  .list-item.virtual-row {
    position: absolute;
    left: 0;
    right: 0;
    height: 28px;
    padding: 0 12px;
    display: flex;
    align-items: center;
  }

  .row-name {
    min-width: 0;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* "downloaded" marker: the one thing that distinguishes rows offline, so
     it reads at a glance without competing with the name. */
  .row-cached {
    flex-shrink: 0;
    margin-left: 8px;
    padding: 0 5px;
    border-radius: 3px;
    background: hsl(var(--muted));
    color: hsl(var(--muted-foreground));
    font-size: 10px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .only-cached-btn {
    flex-shrink: 0;
    padding: 3px 7px;
    font-size: 11px;
    border: 1px solid hsl(var(--border));
    border-radius: 4px;
    background: none;
    color: hsl(var(--muted-foreground));
    cursor: pointer;
  }

  .only-cached-btn.on {
    background: hsl(var(--primary));
    border-color: hsl(var(--primary));
    color: hsl(var(--background));
  }

  .list-item:hover {
    background: hsl(var(--secondary));
  }
  .list-item.active {
    background: hsl(var(--border));
    color: hsl(var(--primary));
    font-weight: 500;
  }

  .contents-box {
    border-bottom: 1px solid hsl(var(--secondary));
    padding: 8px 10px;
  }

  .contents-title {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: hsl(var(--muted-foreground));
    margin-bottom: 4px;
  }

  .contents-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 2px 0;
    font-size: 12px;
    cursor: pointer;
  }

  .contents-name {
    min-width: 0;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .contents-size {
    flex-shrink: 0;
    font-size: 11px;
    color: hsl(var(--muted-foreground));
    font-variant-numeric: tabular-nums;
  }

  .contents-hint {
    margin-top: 4px;
    font-size: 11px;
    line-height: 1.35;
    color: hsl(var(--muted-foreground));
  }

  .maps-actions {
    padding: 8px 10px;
    border-bottom: 1px solid hsl(var(--secondary));
    flex-shrink: 0;
  }

  .open-bundle-btn {
    width: 100%;
    font-size: 12px;
    font-weight: 600;
    padding: 6px 10px;
    background: hsl(var(--primary));
    color: hsl(var(--background));
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .open-bundle-btn:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .open-bundle-btn:disabled {
    background: hsl(var(--secondary));
    color: hsl(var(--muted-foreground));
    cursor: default;
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

  .map-size {
    font-size: 11px;
    color: hsl(var(--muted-foreground));
    font-variant-numeric: tabular-nums;
  }

  /* `min-width: 0` is what makes the ellipsis work: a flex item defaults to
     min-width auto and refuses to shrink below its content, so a long map
     name pushed the size and the downloaded badge out of the column. */
  .map-name {
    flex: 1;
    min-width: 0;
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

  .empty.pending {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
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
