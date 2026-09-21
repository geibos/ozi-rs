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
    bundleLoaderView,
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
    cancelProjectListing,
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

  // Where the operator was when the loader last closed. The Sheet unmounts
  // this component, so anything held only here is gone by the time they come
  // back — see `bundleLoaderView`.
  const restoredView = get(bundleLoaderView);

  let projectFilter = $state(restoredView.filter);
  let debouncedProjectFilter = $state(restoredView.filter);
  let selectedSlug = $state(restoredView.selectedSlug);
  // Offline this is the only list that means anything: the catalogue is
  // thirteen thousand rows and a crew without a signal can open the handful
  // that are already on disk.
  let onlyCached = $state(restoredView.onlyCached);
  /**
   * Top-level bundle entries the operator cleared.
   *
   * Nothing is skipped by default: what a crew can spare is their call, not
   * this code's. Print sheets and Android tile packs are usually most of the
   * weight and this app opens neither, which is what the hint says.
   */
  let skipped = $state<Record<string, true>>({ ...restoredView.skipped });

  // Hand the position back on every change, so whenever the Sheet takes the
  // component away there is nothing left to lose.
  $effect(() => {
    bundleLoaderView.set({
      filter: projectFilter,
      onlyCached,
      selectedSlug,
      skipped: { ...skipped },
      scrollTop,
    });
  });

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
  let scrollTop = $state(restoredView.scrollTop);
  let viewportHeight = $state(0);
  let scrollRaf = 0;

  function handleListScroll() {
    if (scrollRaf !== 0) return;
    scrollRaf = requestAnimationFrame(() => {
      scrollRaf = 0;
      scrollTop = listEl?.scrollTop ?? 0;
    });
  }

  // The reset below must not fire on the component's first run: the filter it
  // reacts to is the restored one, and resetting for it would throw away the
  // position this component exists to bring back.
  let scrollResetArmed = false;

  $effect(() => {
    // Reset the scroll window whenever the filtered set changes — a stale
    // offset past the new (shorter) list would render nothing.
    void debouncedProjectFilter;
    void onlyCached;
    if (!scrollResetArmed) {
      scrollResetArmed = true;
      if (listEl) listEl.scrollTop = scrollTop;
      return;
    }
    scrollTop = 0;
    if (listEl) listEl.scrollTop = 0;
  });

  /**
   * Keyboard position in the catalogue, as an index into `filtered`.
   *
   * The list is virtualized, so only the rows on screen exist in the DOM:
   * there is no tab order over thirteen thousand of them and never can be.
   * Moving DOM focus row by row would also fight the windowing. So the list
   * itself holds focus and points at the current row with
   * `aria-activedescendant`, which is what a virtualized listbox is for, and
   * the index is kept against the filtered array rather than against what
   * happens to be rendered.
   *
   * -1 is "nowhere yet": arrowing down from the search box starts at the top.
   */
  let keyboardIndex = $state(-1);

  const activeRowId = $derived(
    keyboardIndex >= 0 && keyboardIndex < filtered.length
      ? `project-row-${filtered[keyboardIndex].slug}`
      : undefined,
  );

  function scrollIndexIntoView(index: number) {
    if (!listEl) return;
    const top = index * ROW_HEIGHT;
    const bottom = top + ROW_HEIGHT;
    if (top < listEl.scrollTop) listEl.scrollTop = top;
    else if (bottom > listEl.scrollTop + listEl.clientHeight) {
      listEl.scrollTop = bottom - listEl.clientHeight;
    }
  }

  function moveKeyboardIndex(to: number) {
    if (filtered.length === 0) return;
    const clamped = Math.max(0, Math.min(filtered.length - 1, to));
    keyboardIndex = clamped;
    scrollIndexIntoView(clamped);
  }

  function handleListKeys(event: KeyboardEvent) {
    const page = Math.max(1, Math.floor(viewportHeight / ROW_HEIGHT) - 1);
    const from = keyboardIndex < 0 ? -1 : keyboardIndex;
    switch (event.key) {
      case "ArrowDown":
        moveKeyboardIndex(from + 1);
        break;
      case "ArrowUp":
        moveKeyboardIndex(from <= 0 ? 0 : from - 1);
        break;
      case "PageDown":
        moveKeyboardIndex(from + page);
        break;
      case "PageUp":
        moveKeyboardIndex(from - page);
        break;
      case "Home":
        moveKeyboardIndex(0);
        break;
      case "End":
        moveKeyboardIndex(filtered.length - 1);
        break;
      case "Enter":
      case " ":
        if (keyboardIndex >= 0 && keyboardIndex < filtered.length) {
          void handleSelectProject(filtered[keyboardIndex].slug);
        } else {
          return;
        }
        break;
      default:
        return;
    }
    // Only reached when a key above was handled: the arrows would otherwise
    // scroll the list underneath the position they just moved.
    event.preventDefault();
  }

  /** Down from the search box walks into the list: type, arrow, Enter. */
  function handleSearchKeys(event: KeyboardEvent) {
    if (event.key !== "ArrowDown" && event.key !== "Enter") return;
    if (filtered.length === 0) return;
    event.preventDefault();
    listEl?.focus();
    if (keyboardIndex < 0) moveKeyboardIndex(0);
    if (event.key === "Enter") {
      void handleSelectProject(filtered[Math.max(0, keyboardIndex)].slug);
    }
  }

  // A narrowed list makes the old position meaningless.
  $effect(() => {
    void debouncedProjectFilter;
    void onlyCached;
    keyboardIndex = -1;
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

  /**
   * Stop waiting for the catalogue.
   *
   * The walk is a thousand pages at worst and holds the application busy for
   * all of it, so the download button stays disabled for as long as it runs.
   * What has already arrived stays in the list; it is just not all of it, and
   * the status line says so.
   */
  async function handleStopRefresh() {
    try {
      await cancelProjectListing();
    } catch (error) {
      toast.error($t("loader.stopRefreshFailed"), {
        description: String(error),
      });
    }
  }

  async function handleRefresh() {
    projectsLoading.set(true);
    await loadProjects();
  }

  // ── Preview on row click ────────────────────────────────────────────
  // `previewProject` fetches the map list only; the pending hint clears when
  // the previewed project lands in `currentProject` (state-changed round-trip).
  //
  // The marker is the slug, not the display name: a name is not identity, and
  // in a catalogue of thirteen thousand entries two searches can carry the
  // same one.
  //
  // The timer no longer ends the wait. It used to clear the spinner after
  // fifteen seconds while the request was still running, which said "done"
  // about something that had not happened; now it only admits the wait is
  // running long. What bounds the request is the HTTP read timeout in the
  // backend, which reports a real failure through `previewProject`.
  const PREVIEW_SLOW_MS = 15_000;
  let previewPendingSlug = $state<string | null>(null);
  let previewIsSlow = $state(false);
  let previewTimer: ReturnType<typeof setTimeout> | null = null;

  function clearPreviewPending() {
    previewPendingSlug = null;
    previewIsSlow = false;
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
    previewPendingSlug = slug;
    previewIsSlow = false;
    if (previewTimer !== null) clearTimeout(previewTimer);
    previewTimer = setTimeout(() => {
      previewTimer = null;
      previewIsSlow = true;
    }, PREVIEW_SLOW_MS);
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
      previewPendingSlug !== null &&
      $currentProject?.slug === previewPendingSlug
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
        onkeydown={handleSearchKeys}
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
        <!-- How far it has got, because that is what the decision to stop is
             made on: a crew with the search they came for does not need the
             other twelve thousand. -->
        {$projects.length === 0
          ? $t("loader.refreshing")
          : $t("loader.refreshingCount").replace(
              "{count}",
              String($projects.length),
            )}
        <button
          class="stop-refresh-btn"
          onclick={handleStopRefresh}
          data-testid="stop-refresh"
        >
          {$t("loader.stopRefresh")}
        </button>
      </div>
    {:else if $catalogueError !== null}
      <!-- The cached list is the right answer offline, but a crew has to know
           that is what they are reading: a bundle made today would not be in
           it, and a missing row would otherwise read as "no such search". -->
      <div class="refresh-hint offline" data-testid="catalog-offline">
        {$t("loader.offline")}
      </div>
    {/if}

    <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
    <div
      class="list virtual-list"
      data-testid="project-virtual-list"
      role="listbox"
      tabindex="0"
      aria-label={$t("loader.projectListLabel")}
      aria-activedescendant={activeRowId}
      bind:this={listEl}
      bind:clientHeight={viewportHeight}
      onscroll={handleListScroll}
      onkeydown={handleListKeys}
    >
      {#if filtered.length === 0}
        <div class="empty">{$t("loader.noMatches")}</div>
      {:else}
        <div class="virtual-spacer" style={`height: ${totalHeight}px`}>
          {#each visibleRows as row (row.project.slug)}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <div
              class="virtual-row list-item"
              id={`project-row-${row.project.slug}`}
              role="option"
              aria-selected={selectedSlug === row.project.slug}
              class:active={selectedSlug === row.project.slug}
              class:keyed={activeRowId === `project-row-${row.project.slug}`}
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
            </div>
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

    {#if $currentProject && !previewPendingSlug && $currentProject.contents.length > 0}
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

    {#if $currentProject && !previewPendingSlug}
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
      {#if previewPendingSlug}
        <div class="empty pending" data-testid="maps-pending">
          <span class="spinner"></span>
          {previewIsSlow
            ? $t("loader.mapsStillLoading")
            : $t("loader.loadingMaps")}
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

  /* The keyboard position is not DOM focus — the list holds that — so it
     needs a mark of its own, distinct from the selected row. */
  .list-item.virtual-row.keyed {
    outline: 2px solid var(--ring);
    outline-offset: -2px;
  }

  .virtual-list:focus-visible {
    outline: 2px solid var(--ring);
    outline-offset: -2px;
  }

  .stop-refresh-btn {
    margin-left: auto;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: transparent;
    padding: 0.05rem 0.4rem;
    color: inherit;
    font-size: inherit;
    cursor: pointer;
  }

  .stop-refresh-btn:hover {
    background: var(--muted);
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
