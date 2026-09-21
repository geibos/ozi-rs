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
    projects,
    projectsLoading,
    syncProjectsFromAppState,
  } from "../lib/stores";
  import { cancelDownload } from "../lib/api";
  import { locale, t } from "../lib/i18n";
  import { formatBytes } from "$lib/format-bytes";
  import BundleLoader from "../components/BundleLoader.svelte";

  let transientStatus = $state<string | null>(null);

  $effect(() => {
    const s = $appState;
    syncProjectsFromAppState(s);
    if (s && !s.busy) {
      projectsLoading.set(false);
      bundleProgress.set(null);
    }
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

  async function handleCancelDownload() {
    const id = $activeDownloadId;
    if (!id) return;
    await cancelDownload(id);
    activeDownloadId.set(null);
  }

  const bundlePercent = $derived(
    $bundleProgress?.total
      ? Math.round(
          (($bundleProgress.completed ?? 0) / $bundleProgress.total) * 100,
        )
      : null,
  );

  /**
   * What the bottom line says on the app's first screen.
   *
   * The backend's own status is free English text ("Load projects from
   * maps.lizaalert.ru"), and it was the first thing a Russian crew read on
   * launch. When the frontend knows the answer — the catalogue is loading, or
   * it holds N projects — it says so in the interface language. A transient
   * message, a running download's phase or a backend status that is not the
   * idle one still win, because those carry information this cannot derive.
   */
  const catalogueText = $derived.by(() => {
    if ($projectsLoading) return $t("catalogue.loading");
    if ($projects.length === 0) return $t("catalogue.empty");
    return $t("catalogue.loaded").replace("{count}", String($projects.length));
  });

  const statusText = $derived(
    transientStatus ?? $bundleProgress?.message ?? catalogueText,
  );

  const currentFileLabel = $derived.by(() => {
    const c = $currentDownload;
    if (!c) return null;
    if (c.file_index == null || c.file_count == null) return null;
    return `${c.file_index + 1} / ${c.file_count} — ${c.package_name}`;
  });
</script>

<div class="root">
  <BundleLoader />

  <div class="status-bar" class:busy={$busy}>
    <div class="status-line-slot">
      {#if $busy || transientStatus}
        <span class="spinner"></span>
      {/if}
      <span class="status-text" title={statusText}>{statusText}</span>
    </div>

    <div class="current-file-slot">
      {#if currentFileLabel}
        <span class="current-file" data-testid="current-file-label">
          {$t("download.currentFile")}
          {currentFileLabel}
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
          {formatBytes($bundleProgress.downloaded_bytes, $locale)}
          {$bundleProgress.total_bytes
            ? `/ ${formatBytes($bundleProgress.total_bytes, $locale)}`
            : ""}
        </span>
      {/if}
    </div>

    <div class="status-actions">
      {#if $activeDownloadId && $busy}
        <button
          class="action-btn"
          data-testid="cancel-download"
          onclick={handleCancelDownload}>{$t("download.cancel")}</button
        >
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

  .status-bar {
    flex-shrink: 0;
    height: var(--bundle-status-bar-height);
    display: grid;
    /* minmax(0, …): a bare 1fr track's min-size is `auto`, so a long
       nowrap status line ("Imported …") used to push the grid wider than
       the window instead of truncating. */
    grid-template-columns: minmax(0, 1fr) auto;
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
    background: linear-gradient(
      90deg,
      hsl(var(--primary)),
      oklch(from hsl(var(--primary)) calc(l + 0.08) c h)
    );
    transition: width 0.2s ease;
  }

  @keyframes indeterminate-bar {
    0% {
      margin-left: -40%;
      width: 40%;
    }
    100% {
      margin-left: 100%;
      width: 40%;
    }
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
