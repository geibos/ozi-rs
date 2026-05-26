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
  import DownloadIcon from "@lucide/svelte/icons/download";
  import EyeIcon from "@lucide/svelte/icons/eye";
  import EyeOffIcon from "@lucide/svelte/icons/eye-off";
  import FileOutputIcon from "@lucide/svelte/icons/file-output";
  import LineChartIcon from "@lucide/svelte/icons/line-chart";
  import SlidersHorizontalIcon from "@lucide/svelte/icons/sliders-horizontal";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import WavesIcon from "@lucide/svelte/icons/waves";
  import { Button } from "$lib/components/ui/button";
  import {
    appState,
    selectedTrack,
    simplifyState,
  } from "$lib/stores";
  import {
    deleteTrack,
    exportGpx,
    exportTrackPlt,
    getTrackDetail,
    getTrackExportDefaultPath,
    setTrackLineWidth,
    toggleTrackVisible,
  } from "$lib/api";
  import { open } from "@tauri-apps/plugin-dialog";
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
    const key = `${sel.layerId}:${sel.trackId}`;
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

<div class="flex h-full flex-col gap-3 overflow-y-auto p-4">
  <header class="flex items-start gap-3">
    <span
      class="mt-1 inline-block size-4 shrink-0 rounded-full border border-black/10"
      style="background: {summary?.color ?? 'transparent'}"
      aria-hidden="true"
    ></span>
    <div class="min-w-0 flex-1">
      <h2
        class="text-foreground truncate text-sm font-semibold leading-tight"
        title={summary?.name}
      >
        {summary?.name ?? "—"}
      </h2>
      <p class="text-muted-foreground mt-0.5 text-[11px]">Track</p>
    </div>
    <Button
      variant="ghost"
      size="icon-sm"
      onclick={handleToggleVisible}
      disabled={!summary}
      aria-label={summary?.visible ? "Hide track" : "Show track"}
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
    aria-label="Track statistics"
  >
    <h3
      class="text-muted-foreground/80 mb-3 text-[10px] font-semibold uppercase tracking-wider"
    >
      Statistics
    </h3>
    {#if summary}
      <dl
        class="grid grid-cols-[6rem_1fr] gap-x-3 gap-y-2 text-xs tabular-nums"
      >
        <dt class="text-muted-foreground">Distance</dt>
        <dd class="font-mono">{formatDistanceKm(summary!.distance_km)}</dd>
        <dt class="text-muted-foreground">Duration</dt>
        <dd class="font-mono">
          {summary!.duration_seconds !== null
            ? formatDurationSeconds(summary!.duration_seconds!)
            : "—"}
        </dd>
        <dt class="text-muted-foreground">Points</dt>
        <dd class="font-mono">{formatPointCount(summary!.point_count)}</dd>
        <dt class="text-muted-foreground">Start time</dt>
        <dd class="font-mono">{firstTimestamp(trackDetail) ?? "—"}</dd>
      </dl>
    {:else}
      <p class="text-muted-foreground text-xs">No track selected.</p>
    {/if}
  </section>

  <TrackSegmentsTable />

  <section
    class="bg-card border-border rounded-[var(--radius-card)] border p-4"
    aria-label="Elevation chart placeholder"
  >
    <h3
      class="text-muted-foreground/80 mb-2 flex items-center gap-1.5 text-[10px] font-semibold uppercase tracking-wider"
    >
      <LineChartIcon class="size-3" />
      Elevation
    </h3>
    <p class="text-muted-foreground text-[11px] italic">
      Elevation chart — coming in a follow-up change
    </p>
  </section>

  <section
    class="bg-card border-border rounded-[var(--radius-card)] border p-3"
    aria-label="Line width"
  >
    <div class="flex items-center gap-3">
      <SlidersHorizontalIcon
        class="text-muted-foreground size-3.5 shrink-0"
        aria-hidden="true"
      />
      <span class="text-muted-foreground text-[11px]">Width</span>
      <input
        type="range"
        min="1"
        max="12"
        step="1"
        value={lineWidthDraft}
        class="accent-primary flex-1"
        aria-label="Track line width"
        onchange={handleLineWidthChange}
        disabled={!summary}
      />
      <span class="text-muted-foreground w-8 text-right text-[11px]"
        >{lineWidthDraft}px</span
      >
    </div>
  </section>

  <section class="flex flex-col gap-2" aria-label="Track actions">
    <div class="grid grid-cols-2 gap-2">
      <Button
        variant="outline"
        size="sm"
        class="justify-start gap-2"
        onclick={handleExportGpx}
        disabled={!summary}
      >
        <DownloadIcon class="size-4" />
        Export GPX
      </Button>
      <Button
        variant="outline"
        size="sm"
        class="justify-start gap-2"
        onclick={handleExportPlt}
        disabled={!summary}
      >
        <FileOutputIcon class="size-4" />
        Export PLT
      </Button>
    </div>
    <Button
      variant="outline"
      size="sm"
      class="justify-start gap-2"
      onclick={handleSimplify}
      disabled={!summary}
    >
      <WavesIcon class="size-4" />
      Simplify
    </Button>
    <Button
      variant="outline"
      size="sm"
      class="justify-start gap-2 text-destructive hover:text-destructive"
      onclick={handleDelete}
      disabled={!summary}
    >
      <Trash2Icon class="size-4" />
      Delete track
    </Button>
  </section>
</div>
