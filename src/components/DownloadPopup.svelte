<script lang="ts">
  /**
   * Floating per-file bundle-download progress panel (owner feedback: the
   * single aggregate bar never filled — see the note below).
   *
   * Mounted once in `+layout.svelte` so it stays visible on BOTH routes:
   * downloads keep running while the user works on `/project`.
   *
   * Why the old aggregate bar never filled: the backend emits
   * `bundle-progress` with `completed/total` only twice per bundle —
   * `0/N` when the download phase starts and `N/N` after every file has
   * finished (`lizaalert.rs`), so a bar bound to that ratio sits at 0 %
   * for the whole download and then jumps to 100 %. The per-file
   * `download-progress` stream, meanwhile, was rendered in the maps
   * column only behind `$downloadingMaps.has(name)` — a set the bundle
   * path never populates (it belongs to the single-map download path).
   * This panel renders straight from the `downloadProgress` store, which
   * updates immutably (a new Map per event), so every byte-level event
   * repaints its file's bar.
   *
   * Visibility: an active download id. Both download paths emit
   * `download-finished` with their id and the layout clears it, so the panel
   * lasts exactly as long as the download it belongs to — a single-map
   * download never touches the busy flag, and a catalogue refresh sets it
   * with nothing downloading.
   */
  import {
    activeDownloadId,
    bundleProgress,
    currentDownload,
    downloadProgress,
  } from "../lib/stores";
  import { cancelDownload } from "../lib/api";
  import { locale, t } from "../lib/i18n";
  import { formatBytes } from "$lib/format-bytes";

  const rows = $derived(
    [...$downloadProgress.values()]
      .filter(
        (p) =>
          $activeDownloadId === null || p.download_id === $activeDownloadId,
      )
      .sort((a, b) => (a.file_index ?? 0) - (b.file_index ?? 0)),
  );

  const fileTotal = $derived(
    $currentDownload?.file_count ?? $bundleProgress?.total ?? null,
  );

  // Files fully on disk: rows whose byte counter reached its known total.
  // `bundle-progress.completed` only ever reports 0 or N (see above), but
  // taking the max folds in resume-skipped files that never stream any
  // `download-progress` events.
  const filesDone = $derived.by(() => {
    const finished = rows.filter(
      (p) => p.total_bytes != null && p.downloaded_bytes >= p.total_bytes,
    ).length;
    return Math.max(finished, $bundleProgress?.completed ?? 0);
  });

  async function handleCancel() {
    const id = $activeDownloadId;
    if (!id) return;
    try {
      await cancelDownload(id);
    } finally {
      activeDownloadId.set(null);
    }
  }
</script>

<!-- The panel belongs to the download, not to the global busy flag: a
     single-map download never sets busy, and a catalogue refresh sets it
     without any download running. `download-finished` clears the id. -->
{#if $activeDownloadId !== null}
  <div class="download-popup" data-testid="download-popup">
    <div class="popup-header">
      <span class="popup-title">{$t("download.title")}</span>
      {#if fileTotal != null}
        <span class="popup-count" data-testid="popup-file-count">
          {$t("download.files")}: {filesDone}/{fileTotal}
        </span>
      {/if}
      <button
        class="popup-cancel"
        data-testid="popup-cancel-download"
        onclick={handleCancel}
      >
        {$t("download.cancel")}
      </button>
    </div>

    <div class="popup-rows">
      {#each rows as p (p.package_name)}
        {@const pct = p.total_bytes
          ? Math.min(
              100,
              Math.round((p.downloaded_bytes / p.total_bytes) * 100),
            )
          : null}
        <div class="popup-row" data-testid="popup-file-row">
          <div class="popup-row-top">
            <span class="popup-file" title={p.package_name}>
              {p.package_name}
            </span>
            <span class="popup-size">
              {formatBytes(p.downloaded_bytes, $locale)}{p.total_bytes
                ? ` / ${formatBytes(p.total_bytes, $locale)}`
                : ""}
            </span>
          </div>
          <div class="popup-track">
            <div
              class="popup-fill"
              class:indeterminate={pct == null}
              style={pct != null ? `width: ${pct}%` : ""}
            ></div>
          </div>
        </div>
      {:else}
        <div class="popup-empty">{$t("download.starting")}</div>
      {/each}
    </div>
  </div>
{/if}

<style>
  .download-popup {
    position: fixed;
    right: 12px;
    bottom: 12px;
    z-index: 40;
    width: 320px;
    max-height: 40vh;
    display: flex;
    flex-direction: column;
    background: hsl(var(--card));
    border: 1px solid hsl(var(--border));
    border-radius: 8px;
    box-shadow: 0 8px 24px hsl(var(--background) / 0.6);
    overflow: hidden;
  }

  .popup-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    border-bottom: 1px solid hsl(var(--secondary));
    flex-shrink: 0;
  }

  .popup-title {
    flex: 1;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: hsl(var(--foreground));
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .popup-count {
    font-size: 10px;
    color: hsl(var(--muted-foreground));
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }

  .popup-cancel {
    font-size: 11px;
    padding: 2px 8px;
    background: hsl(var(--border));
    color: hsl(var(--foreground));
    border: 1px solid hsl(var(--muted));
    border-radius: 3px;
    cursor: pointer;
    flex-shrink: 0;
  }

  .popup-cancel:hover {
    background: hsl(var(--muted));
  }

  .popup-rows {
    overflow-y: auto;
    padding: 6px 10px 8px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .popup-row {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .popup-row-top {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .popup-file {
    flex: 1;
    font-size: 11px;
    color: hsl(var(--foreground));
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .popup-size {
    font-size: 10px;
    color: hsl(var(--muted-foreground));
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .popup-track {
    height: 3px;
    background: hsl(var(--border));
    border-radius: 2px;
    overflow: hidden;
  }

  .popup-fill {
    height: 100%;
    background: hsl(var(--primary));
    border-radius: 2px;
    transition: width 0.25s ease;
  }

  @keyframes popup-indeterminate {
    0% {
      transform: translateX(-100%);
      width: 40%;
    }
    100% {
      transform: translateX(350%);
      width: 40%;
    }
  }

  .popup-fill.indeterminate {
    width: 40% !important;
    animation: popup-indeterminate 1.2s ease-in-out infinite;
  }

  .popup-empty {
    font-size: 11px;
    color: hsl(var(--muted-foreground));
    text-align: center;
    padding: 8px 0;
  }
</style>
