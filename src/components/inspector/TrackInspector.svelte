<script lang="ts">
  /**
   * Track Inspector — context-sensitive surface for `$selectedTrack`.
   *
   * Layout (top-down, locked by the inspector-pane spec):
   *   1. Header  — colour swatch + name + visibility toggle
   *   2. Stats card — distance / duration / point count / start time
   *   3. Segments / points table (`<TrackSegmentsTable />`)
   *   4. Elevation-chart placeholder (no IPC, no charting library)
   *   5. Actions row — Export GPX / Export PLT / Set line width / Simplify / Delete
   *
   * All actions dispatch through the existing `ProjectCommand`-shaped
   * endpoints in `src/lib/api.ts`. No direct store mutation. No new IPC.
   */
  import ArrowUpDownIcon from "@lucide/svelte/icons/arrow-up-down";
  import CalendarClockIcon from "@lucide/svelte/icons/calendar-clock";
  import CropIcon from "@lucide/svelte/icons/crop";
  import DownloadIcon from "@lucide/svelte/icons/download";
  import EyeIcon from "@lucide/svelte/icons/eye";
  import EyeOffIcon from "@lucide/svelte/icons/eye-off";
  import FileOutputIcon from "@lucide/svelte/icons/file-output";
  import LineChartIcon from "@lucide/svelte/icons/line-chart";
  import LocateIcon from "@lucide/svelte/icons/locate";
  import SlidersHorizontalIcon from "@lucide/svelte/icons/sliders-horizontal";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import WavesIcon from "@lucide/svelte/icons/waves";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import {
    appState,
    mapViewportBounds,
    requestTrackFocus,
    selectedTrack,
    simplifyState,
    tracksGeometryVersion,
  } from "$lib/stores";
  import {
    cropTrackToExtent,
    cropTrackToTime,
    deleteTrack,
    exportGpx,
    exportTrackPlt,
    getTrackDetail,
    getTrackExportDefaultPath,
    setTrackLineWidth,
    sortTrackPoints,
    toggleTrackVisible,
  } from "$lib/api";
  import { t } from "$lib/i18n";
  import { confirm as confirmDialog, open } from "@tauri-apps/plugin-dialog";
  import { toast } from "svelte-sonner";
  import {
    formatDistanceKm,
    formatDurationSeconds,
    formatPointCount,
  } from "$lib/track-stats";
  import type { TrackDetail, TrackSummary } from "$lib/types";
  import TrackSegmentsTable from "./TrackSegmentsTable.svelte";

  let trackDetail: TrackDetail | null = $state(null);
  let detailKey: string | null = $state(null);
  let lineWidthDraft = $state(3);

  const summary: TrackSummary | null = $derived.by(() => {
    const sel = $selectedTrack;
    const state = $appState;
    if (!sel || !state) return null;
    return (
      state.tracks.find(
        (t) =>
          BigInt(t.layer_id) === sel.layerId &&
          BigInt(t.track_id) === sel.trackId,
      ) ?? null
    );
  });

  $effect(() => {
    if (summary) lineWidthDraft = summary.line_width;
  });

  // Pull TrackDetail purely for the "start time" cell of the stats card.
  // The segments table component loads its own copy via `loadTrackDetail`
  // — the duplicated call is acceptable here because both calls are
  // cached by the backend and segment-table data is cached locally too.
  $effect(() => {
    const sel = $selectedTrack;
    if (!sel || !$appState) {
      trackDetail = null;
      detailKey = null;
      return;
    }
    // $tracksGeometryVersion is part of the key so cleanup actions
    // (sort / crop / split / join) invalidate the cached detail.
    const key = `${sel.layerId}:${sel.trackId}:${$tracksGeometryVersion}`;
    if (key === detailKey) return;
    void loadDetail(sel.layerId, sel.trackId, key);
  });

  async function loadDetail(layerId: bigint, trackId: bigint, key: string) {
    try {
      trackDetail = await getTrackDetail(layerId, trackId);
      detailKey = key;
    } catch (error) {
      console.error("TrackInspector: getTrackDetail failed", error);
      trackDetail = null;
    }
  }

  function firstTimestamp(detail: TrackDetail | null): string | null {
    if (!detail) return null;
    for (const seg of detail.segments) {
      for (const pt of seg.points) {
        if (pt.timestamp) return pt.timestamp;
      }
    }
    return null;
  }

  async function handleToggleVisible() {
    const sel = $selectedTrack;
    if (!sel) return;
    try {
      await toggleTrackVisible(sel.layerId, sel.trackId);
    } catch (error) {
      toast.error("Failed to toggle visibility", {
        description: String(error),
      });
    }
  }

  async function handleExportGpx() {
    const sel = $selectedTrack;
    const s = summary;
    if (!sel || !s) return;
    try {
      const defaultPath = await getTrackExportDefaultPath(s.name, "gpx");
      const path = await open({
        save: true,
        defaultPath: defaultPath ?? `${s.name}.gpx`,
        filters: [{ name: "GPX", extensions: ["gpx"] }],
      } as Parameters<typeof open>[0]);
      if (path) await exportGpx(sel.layerId, path as string);
    } catch (error) {
      toast.error("Failed to export GPX", { description: String(error) });
    }
  }

  async function handleExportPlt() {
    const sel = $selectedTrack;
    const s = summary;
    if (!sel || !s) return;
    try {
      const defaultPath = await getTrackExportDefaultPath(s.name, "plt");
      const path = await open({
        save: true,
        defaultPath: defaultPath ?? `${s.name}.plt`,
        filters: [{ name: "PLT", extensions: ["plt"] }],
      } as Parameters<typeof open>[0]);
      if (path) await exportTrackPlt(sel.layerId, sel.trackId, path as string);
    } catch (error) {
      toast.error("Failed to export PLT", { description: String(error) });
    }
  }

  async function handleLineWidthChange(event: Event) {
    const sel = $selectedTrack;
    if (!sel) return;
    const width = Number((event.currentTarget as HTMLInputElement).value);
    lineWidthDraft = width;
    try {
      await setTrackLineWidth(sel.layerId, sel.trackId, width);
    } catch (error) {
      toast.error("Failed to set line width", { description: String(error) });
    }
  }

  function handleSimplify() {
    const sel = $selectedTrack;
    if (!sel) return;
    simplifyState.set({
      active: true,
      layerId: sel.layerId,
      trackId: sel.trackId,
      tolerance: 10,
      preview: null,
    });
  }

  /** "Show on map" — MapView consumes the request and fits the bounds. */
  function handleShowOnMap() {
    const sel = $selectedTrack;
    if (!sel) return;
    requestTrackFocus(sel.layerId, sel.trackId);
  }

  // ── CJ-4 track cleanup: sort by time / crop to view / crop by time ──

  let cropTimeOpen = $state(false);
  let cropFromDraft = $state("");
  let cropToDraft = $state("");

  function pointsRemovedToast(removed: number) {
    toast.success(
      $t("trackInspector.pointsRemoved").replace("{count}", String(removed)),
    );
  }

  async function handleSortByTime() {
    const sel = $selectedTrack;
    if (!sel) return;
    try {
      await sortTrackPoints(sel.layerId, sel.trackId);
      tracksGeometryVersion.update((v) => v + 1);
      toast.success($t("trackInspector.sortDone"));
    } catch (error) {
      toast.error($t("trackInspector.sortFailed"), {
        description: String(error),
      });
    }
  }

  async function handleCropToView() {
    const sel = $selectedTrack;
    const bounds = $mapViewportBounds;
    if (!sel || !bounds) return;
    const confirmed = await confirmDialog(
      $t("trackInspector.cropToViewConfirm"),
      {
        title: $t("trackInspector.cropTitle"),
        kind: "warning",
        okLabel: $t("trackInspector.crop"),
        cancelLabel: $t("trackInspector.cancel"),
      },
    );
    if (!confirmed) return;
    try {
      const removed = await cropTrackToExtent(sel.layerId, sel.trackId, {
        min_lat: bounds.minLat,
        min_lon: bounds.minLon,
        max_lat: bounds.maxLat,
        max_lon: bounds.maxLon,
      });
      tracksGeometryVersion.update((v) => v + 1);
      pointsRemovedToast(removed);
    } catch (error) {
      toast.error($t("trackInspector.cropFailed"), {
        description: String(error),
      });
    }
  }

  /** Format an epoch-ms instant as a `datetime-local` value (local time). */
  function toDatetimeLocal(ms: number): string {
    const d = new Date(ms);
    const pad = (n: number) => String(n).padStart(2, "0");
    return (
      `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}` +
      `T${pad(d.getHours())}:${pad(d.getMinutes())}`
    );
  }

  /**
   * `datetime-local` values carry no offset, so `Date.parse` reads them as
   * local time — exactly what the input shows. Empty input → open bound.
   */
  function draftToIsoUtc(draft: string): string | null {
    if (!draft) return null;
    const ms = Date.parse(draft);
    return Number.isNaN(ms) ? null : new Date(ms).toISOString();
  }

  function openCropByTime() {
    let first: number | null = null;
    let last: number | null = null;
    for (const seg of trackDetail?.segments ?? []) {
      for (const pt of seg.points) {
        if (!pt.timestamp) continue;
        const ms = Date.parse(pt.timestamp);
        if (Number.isNaN(ms)) continue;
        if (first === null || ms < first) first = ms;
        if (last === null || ms > last) last = ms;
      }
    }
    cropFromDraft = first !== null ? toDatetimeLocal(first) : "";
    cropToDraft = last !== null ? toDatetimeLocal(last) : "";
    cropTimeOpen = true;
  }

  async function handleCropByTime() {
    const sel = $selectedTrack;
    if (!sel) return;
    const from = draftToIsoUtc(cropFromDraft);
    const to = draftToIsoUtc(cropToDraft);
    cropTimeOpen = false;
    try {
      const removed = await cropTrackToTime(sel.layerId, sel.trackId, from, to);
      tracksGeometryVersion.update((v) => v + 1);
      pointsRemovedToast(removed);
    } catch (error) {
      toast.error($t("trackInspector.cropFailed"), {
        description: String(error),
      });
    }
  }

  async function handleDelete() {
    const sel = $selectedTrack;
    if (!sel) return;
    try {
      await deleteTrack(sel.layerId, sel.trackId);
      selectedTrack.set(null);
    } catch (error) {
      toast.error("Failed to delete track", { description: String(error) });
    }
  }
</script>

<div
  class="flex h-full min-w-0 flex-col gap-3 overflow-x-hidden overflow-y-auto p-4"
>
  <header class="flex items-start gap-3">
    <span
      class="mt-1 inline-block size-4 shrink-0 rounded-full border border-black/10"
      style="background: {summary?.color ?? 'transparent'}"
      aria-hidden="true"
    ></span>
    <div class="min-w-0 flex-1">
      <h2
        class="text-foreground truncate text-sm leading-tight font-semibold"
        title={summary?.name}
      >
        {summary?.name ?? "—"}
      </h2>
      <p class="text-muted-foreground mt-0.5 text-[11px]">
        {$t("inspector.track")}
      </p>
    </div>
    <Button
      variant="ghost"
      size="icon-sm"
      onclick={handleToggleVisible}
      disabled={!summary}
      aria-label={summary?.visible
        ? $t("inspector.hideTrack")
        : $t("inspector.showTrack")}
    >
      {#if summary?.visible}
        <EyeIcon class="size-4" />
      {:else}
        <EyeOffIcon class="size-4" />
      {/if}
    </Button>
  </header>

  <section
    class="bg-card border-border rounded-[var(--radius-card)] border p-4"
    aria-label={$t("inspector.statistics")}
  >
    <h3
      class="text-muted-foreground/80 mb-3 text-[10px] font-semibold tracking-wider uppercase"
    >
      {$t("inspector.statistics")}
    </h3>
    {#if summary}
      <dl
        class="grid grid-cols-[6rem_1fr] gap-x-3 gap-y-2 text-xs tabular-nums"
      >
        <dt class="text-muted-foreground">{$t("inspector.distance")}</dt>
        <dd class="min-w-0 truncate font-mono">
          {formatDistanceKm(summary!.distance_km)}
        </dd>
        <dt class="text-muted-foreground">{$t("inspector.duration")}</dt>
        <dd
          class="min-w-0 truncate font-mono"
          title={summary!.duration_seconds !== null
            ? $t("track.durationTooltip")
            : undefined}
        >
          {summary!.duration_seconds !== null
            ? formatDurationSeconds(summary!.duration_seconds!)
            : "—"}
        </dd>
        <dt class="text-muted-foreground">{$t("inspector.points")}</dt>
        <dd class="min-w-0 truncate font-mono">
          {formatPointCount(summary!.point_count)}
        </dd>
        <dt class="text-muted-foreground">{$t("inspector.startTime")}</dt>
        <dd
          class="min-w-0 truncate font-mono"
          title={firstTimestamp(trackDetail) ?? undefined}
        >
          {firstTimestamp(trackDetail) ?? "—"}
        </dd>
      </dl>
    {:else}
      <p class="text-muted-foreground text-xs">{$t("inspector.noTrack")}</p>
    {/if}
  </section>

  <TrackSegmentsTable />

  <section
    class="bg-card border-border rounded-[var(--radius-card)] border p-4"
    aria-label={$t("inspector.elevation")}
  >
    <h3
      class="text-muted-foreground/80 mb-2 flex items-center gap-1.5 text-[10px] font-semibold tracking-wider uppercase"
    >
      <LineChartIcon class="size-3" />
      {$t("inspector.elevation")}
    </h3>
    <p class="text-muted-foreground text-[11px] italic">
      {$t("inspector.elevationSoon")}
    </p>
  </section>

  <section
    class="bg-card border-border rounded-[var(--radius-card)] border p-3"
    aria-label={$t("inspector.lineWidth")}
  >
    <div class="flex items-center gap-3">
      <SlidersHorizontalIcon
        class="text-muted-foreground size-3.5 shrink-0"
        aria-hidden="true"
      />
      <span class="text-muted-foreground text-[11px]"
        >{$t("inspector.width")}</span
      >
      <input
        type="range"
        min="1"
        max="12"
        step="1"
        value={lineWidthDraft}
        class="accent-primary flex-1"
        aria-label={$t("inspector.lineWidth")}
        onchange={handleLineWidthChange}
        disabled={!summary}
      />
      <span class="text-muted-foreground w-8 text-right text-[11px]"
        >{lineWidthDraft}px</span
      >
    </div>
  </section>

  <!--
    Actions — a single-column stack of full-width, equally sized buttons.
    The CJ-4 additions made a mixed grid/stack layout overflow the narrow
    inspector rail; every button now carries `w-full min-w-0` and a
    truncating label span so long localized labels never push past the
    rail width.
  -->
  <section class="flex min-w-0 flex-col gap-2" aria-label="Track actions">
    <Button
      variant="outline"
      size="sm"
      class="w-full min-w-0 justify-start gap-2"
      onclick={handleShowOnMap}
      disabled={!summary}
      data-testid="inspector-show-on-map"
    >
      <LocateIcon class="size-4" />
      <span class="truncate">{$t("track.showOnMap")}</span>
    </Button>
    <Button
      variant="outline"
      size="sm"
      class="w-full min-w-0 justify-start gap-2"
      onclick={handleExportGpx}
      disabled={!summary}
    >
      <DownloadIcon class="size-4" />
      <span class="truncate">{$t("inspector.exportGpx")}</span>
    </Button>
    <Button
      variant="outline"
      size="sm"
      class="w-full min-w-0 justify-start gap-2"
      onclick={handleExportPlt}
      disabled={!summary}
    >
      <FileOutputIcon class="size-4" />
      <span class="truncate">{$t("inspector.exportPlt")}</span>
    </Button>
    <Button
      variant="outline"
      size="sm"
      class="w-full min-w-0 justify-start gap-2"
      onclick={handleSortByTime}
      disabled={!summary}
    >
      <ArrowUpDownIcon class="size-4" />
      <span class="truncate">{$t("trackInspector.sortByTime")}</span>
    </Button>
    <Button
      variant="outline"
      size="sm"
      class="w-full min-w-0 justify-start gap-2"
      onclick={handleCropToView}
      disabled={!summary || !$mapViewportBounds}
    >
      <CropIcon class="size-4" />
      <span class="truncate">{$t("trackInspector.cropToView")}</span>
    </Button>
    <Button
      variant="outline"
      size="sm"
      class="w-full min-w-0 justify-start gap-2"
      onclick={openCropByTime}
      disabled={!summary}
    >
      <CalendarClockIcon class="size-4" />
      <span class="truncate">{$t("trackInspector.cropByTime")}</span>
    </Button>
    <Button
      variant="outline"
      size="sm"
      class="w-full min-w-0 justify-start gap-2"
      onclick={handleSimplify}
      disabled={!summary}
    >
      <WavesIcon class="size-4" />
      <span class="truncate">{$t("trackInspector.simplify")}</span>
    </Button>
    <Button
      variant="outline"
      size="sm"
      class="text-destructive hover:text-destructive w-full min-w-0 justify-start gap-2"
      onclick={handleDelete}
      disabled={!summary}
    >
      <Trash2Icon class="size-4" />
      <span class="truncate">{$t("inspector.deleteTrack")}</span>
    </Button>
  </section>
</div>

<Dialog.Root bind:open={cropTimeOpen}>
  <Dialog.Content class="max-w-sm">
    <Dialog.Header>
      <Dialog.Title>{$t("trackInspector.cropByTime")}</Dialog.Title>
      <Dialog.Description>
        {$t("trackInspector.cropByTimeNote")}
      </Dialog.Description>
    </Dialog.Header>
    <div class="flex flex-col gap-3">
      <label class="flex flex-col gap-1 text-xs">
        <span class="text-muted-foreground">
          {$t("trackInspector.cropByTimeFrom")}
        </span>
        <input
          type="datetime-local"
          bind:value={cropFromDraft}
          class="border-border bg-background rounded-md border px-2 py-1 text-xs"
        />
      </label>
      <label class="flex flex-col gap-1 text-xs">
        <span class="text-muted-foreground">
          {$t("trackInspector.cropByTimeTo")}
        </span>
        <input
          type="datetime-local"
          bind:value={cropToDraft}
          class="border-border bg-background rounded-md border px-2 py-1 text-xs"
        />
      </label>
    </div>
    <Dialog.Footer>
      <Button
        variant="outline"
        size="sm"
        onclick={() => (cropTimeOpen = false)}
      >
        {$t("trackInspector.cancel")}
      </Button>
      <Button size="sm" onclick={handleCropByTime}>
        {$t("trackInspector.crop")}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
