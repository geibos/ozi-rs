<script lang="ts">
  /**
   * Segments / points table — port of the retired `TrackPointsPanel`
   * verbatim where possible. Subscribes to the SAME stores the legacy panel
   * did: `$selectedPointId`, `$activeTrackLayerId`, `$editModeActive`.
   * Bidirectional highlight (map click ↔ row click) is preserved because
   * the underlying stores are unchanged.
   *
   * Data fetching is delegated to `src/lib/track-points.ts` (the
   * `loadTrackDetail` helper). Formatting helpers (`formatPointCoords`,
   * `formatPointTimestamp`, `segmentHeader`, `pageSegmentPoints`) also live
   * there — they are pure functions, no Svelte runes / no DOM.
   */
  import { Button } from "$lib/components/ui/button";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import * as Table from "$lib/components/ui/table";
  import {
    activeTrackLayerId,
    appState,
    editModeActive,
    selectedPointId,
    selectedTrack,
    tracksGeometryVersion,
    focusTrackPoint,
  } from "$lib/stores";
  import {
    formatPointCoords,
    formatPointTimestamp,
    loadTrackDetail,
    pageSegmentPoints,
    segmentHeader,
  } from "$lib/track-points";
  import { joinSegments, splitSegment, trimTrackAtPoint } from "$lib/api";
  import { reportEditFailure } from "$lib/edit-failure";
  import { locale, t } from "$lib/i18n";
  import { toast } from "svelte-sonner";
  import type { SegmentDetail, TrackDetail } from "$lib/types";

  // `$state<T>(...)`, not an annotated `let`: with the annotation TypeScript
  // narrows the variable to `null` at every point before its first assignment,
  // which is every inline `$derived` below. See `docs/backlog.md`.
  let trackDetail = $state<TrackDetail | null>(null);
  let expandedSegments: Record<number, boolean> = $state({});
  let lastLoaded = $state<string | null>(null);

  $effect(() => {
    const selected = $selectedTrack;
    if (!selected || !$appState) {
      trackDetail = null;
      lastLoaded = null;
      return;
    }
    // Cache by composite key so quickly bouncing between rows in the
    // Library does not refetch on every store tick.
    // $tracksGeometryVersion is part of the key so CJ-4 cleanup actions
    // (sort / crop / split / join) invalidate the cached detail.
    const key = `${selected.layerId}:${selected.trackId}:${$tracksGeometryVersion}`;
    if (key === lastLoaded) return;
    void loadDetail(selected.layerId, selected.trackId, key);
  });

  async function loadDetail(layerId: bigint, trackId: bigint, key: string) {
    try {
      trackDetail = await loadTrackDetail(layerId, trackId);
      expandedSegments = {};
      lastLoaded = key;
      // Keep $activeTrackLayerId aligned with the selection so the map
      // tooling that depends on it (legacy convention) keeps working.
      activeTrackLayerId.set(layerId);
    } catch (e) {
      console.error("Failed to load track details", e);
      toast.error("Failed to load track details", { description: String(e) });
      trackDetail = null;
    }
  }

  function handlePointClick(id: number) {
    selectedPointId.set(BigInt(id));
  }

  function toggleEditMode() {
    if (!$selectedTrack) return;
    editModeActive.update((current) => !current);
  }

  // ── CJ-4: split / join segments ──────────────────────────────────────

  /**
   * The selected point can split its segment when it belongs to this
   * segment and is not the segment's last point (the backend rejects a
   * last-point split — we pre-hide only that obvious case, other backend
   * rejections surface as toasts).
   */
  function canSplitAt(segment: SegmentDetail): boolean {
    const pointId = $selectedPointId;
    if (pointId === null) return false;
    const points = segment.points;
    if (points.length === 0) return false;
    if (BigInt(points[points.length - 1].id) === pointId) return false;
    return points.some((p) => BigInt(p.id) === pointId);
  }

  async function handleSplit(segment: SegmentDetail) {
    const selected = $selectedTrack;
    const pointId = $selectedPointId;
    if (!selected || pointId === null) return;
    try {
      await splitSegment(
        selected.layerId,
        selected.trackId,
        BigInt(segment.id),
        pointId,
      );
      // Bumping the version invalidates this table's cache (the $effect
      // key above) and re-renders the map line via MapView's slice effect.
      tracksGeometryVersion.update((v) => v + 1);
    } catch (error) {
      toast.error($t("points.splitFailed"), { description: String(error) });
    }
  }

  async function handleJoin(previous: SegmentDetail, segment: SegmentDetail) {
    const selected = $selectedTrack;
    if (!selected) return;
    try {
      await joinSegments(
        selected.layerId,
        selected.trackId,
        BigInt(previous.id),
        BigInt(segment.id),
      );
      tracksGeometryVersion.update((v) => v + 1);
    } catch (error) {
      toast.error($t("points.joinFailed"), { description: String(error) });
    }
  }
  /**
   * Trim the track at this point, keeping it.
   *
   * Two trims compose into a crop by selection: cut before the start, cut
   * after the end. That is the same result with half the interface, and each
   * half is a gesture on its own — "the drive to the start is the first twenty
   * minutes" needs only one of them.
   */
  async function handleTrim(pointId: number, before: boolean) {
    const selected = $selectedTrack;
    if (!selected) return;
    try {
      const removed = await trimTrackAtPoint(
        selected.layerId,
        selected.trackId,
        BigInt(pointId),
        before,
      );
      if (removed === 0) {
        toast.message($t("inspector.trimNothing"));
        return;
      }
      tracksGeometryVersion.update((v) => v + 1);
      toast.success($t("inspector.trimmed").replace("{n}", String(removed)));
    } catch (error) {
      reportEditFailure("inspector.trimFailed", error);
    }
  }
  /**
   * The track's points in order, across its segments.
   *
   * A walkthrough goes through the recording, not through one segment of it:
   * a track recorded in two sittings is still one walk, and stepping off the
   * end of the first segment should land on the start of the second.
   */
  const orderedPoints = $derived(
    (trackDetail?.segments ?? []).flatMap((s) =>
      s.points.map((p) => ({ id: p.id, lat: p.lat, lon: p.lon })),
    ),
  );

  const currentPointIndex = $derived(
    $selectedPointId === null
      ? -1
      : orderedPoints.findIndex((p) => BigInt(p.id) === $selectedPointId),
  );

  /**
   * Step to the next or previous point and take the map with it.
   *
   * A walkthrough that left the map where it was would be a list. With nothing
   * selected, the first step lands on the first point rather than doing
   * nothing: that is what "next" means from nowhere.
   */
  function stepPoint(delta: number) {
    if (orderedPoints.length === 0) return;
    const from = currentPointIndex;
    const to =
      from === -1
        ? delta > 0
          ? 0
          : orderedPoints.length - 1
        : Math.min(orderedPoints.length - 1, Math.max(0, from + delta));
    const point = orderedPoints[to];
    selectedPointId.set(BigInt(point.id));
    focusTrackPoint(point.lat, point.lon);
  }
</script>

<!-- `shrink-0`: this card was the only shrinkable child of the inspector
     column, so whenever the rail ran short of room it — and only it —
     collapsed, hiding the points behind the next card instead of letting the
     rail scroll. Its own `max-h-72` already caps its height. -->
<section
  class="bg-card border-border flex shrink-0 flex-col overflow-hidden rounded-[var(--radius-card)] border"
  aria-label={$t("inspector.segmentsPoints")}
>
  <header
    class="border-border flex items-center justify-between border-b px-3 py-2"
  >
    <span
      class="text-muted-foreground/80 text-[10px] font-semibold tracking-wider uppercase"
      >{$t("inspector.segmentsPoints")}</span
    >
    <!-- Stepping through the points is how a recording is reviewed; without
         controls the only way was to click each row, which loses your place
         the moment the list scrolls. -->
    <span class="flex items-center gap-1">
      {#if orderedPoints.length > 0 && currentPointIndex >= 0}
        <span
          class="text-muted-foreground text-[10px] tabular-nums"
          data-testid="point-position"
          >{$t("inspector.pointPosition")
            .replace("{i}", String(currentPointIndex + 1))
            .replace("{n}", String(orderedPoints.length))}</span
        >
      {/if}
      <button
        class="text-muted-foreground hover:text-foreground border-0 bg-transparent px-1 leading-none disabled:opacity-40"
        title={$t("inspector.previousPoint")}
        aria-label={$t("inspector.previousPoint")}
        data-testid="previous-point"
        disabled={orderedPoints.length === 0 || currentPointIndex === 0}
        onclick={() => stepPoint(-1)}>↑</button
      >
      <button
        class="text-muted-foreground hover:text-foreground border-0 bg-transparent px-1 leading-none disabled:opacity-40"
        title={$t("inspector.nextPoint")}
        aria-label={$t("inspector.nextPoint")}
        data-testid="next-point"
        disabled={orderedPoints.length === 0 ||
          currentPointIndex === orderedPoints.length - 1}
        onclick={() => stepPoint(1)}>↓</button
      >
    </span>
    <Button
      variant={$editModeActive ? "default" : "outline"}
      size="xs"
      disabled={!$selectedTrack}
      onclick={toggleEditMode}
    >
      {$editModeActive ? $t("inspector.stopEdit") : $t("inspector.editMode")}
    </Button>
  </header>

  {#if !$selectedTrack}
    <div class="text-muted-foreground p-3 text-center text-xs">
      Select a track to see points
    </div>
  {:else if !trackDetail}
    <div class="text-muted-foreground p-3 text-center text-xs">
      {$t("inspector.loadingPoints")}
    </div>
  {:else if trackDetail.segments.length === 0}
    <div class="text-muted-foreground p-3 text-center text-xs">
      {$t("inspector.noSegments")}
    </div>
  {:else}
    <ScrollArea class="max-h-72 flex-1">
      {#each trackDetail.segments as segment, segIdx (segment.id)}
        {@const paged = pageSegmentPoints(
          segment,
          expandedSegments[segment.id] === true,
        )}
        <div
          class="bg-muted/50 text-muted-foreground border-border flex items-center justify-between gap-2 px-3 py-1 text-[10px] font-semibold tracking-wider uppercase"
          class:border-t={segIdx > 0}
          class:border-b={true}
        >
          <span class="min-w-0 truncate">{segmentHeader(segment, $locale)}</span
          >
          {#if segIdx > 0}
            <Button
              variant="ghost"
              size="xs"
              class="shrink-0 normal-case"
              onclick={() =>
                handleJoin(trackDetail!.segments[segIdx - 1], segment)}
            >
              {$t("points.joinPrevious")}
            </Button>
          {/if}
        </div>
        <Table.Root>
          <Table.Body>
            {#each paged.visible as point (point.id)}
              <Table.Row
                data-state={$selectedPointId === BigInt(point.id)
                  ? "selected"
                  : undefined}
                class="cursor-pointer text-[11px]"
                onclick={() => handlePointClick(point.id)}
              >
                <Table.Cell class="text-border w-4 px-2 py-1">•</Table.Cell>
                <Table.Cell class="px-2 py-1 font-mono">
                  {formatPointCoords(point)}
                  {#if formatPointTimestamp(point, $locale) !== null}
                    <div
                      class="text-muted-foreground font-mono text-[10px] leading-tight"
                    >
                      {formatPointTimestamp(point, $locale)}
                    </div>
                  {/if}
                </Table.Cell>
                <!-- Trimming lives on the row because that is where the
                     operator has already found the point: they can see it, on
                     the map and here, which is the whole reason this is easier
                     than cropping by a time they would have to work out. -->
                <Table.Cell class="w-14 px-1 py-1 text-right whitespace-nowrap">
                  <button
                    class="text-muted-foreground hover:text-foreground border-0 bg-transparent px-1 leading-none"
                    title={$t("inspector.trimBefore")}
                    aria-label={$t("inspector.trimBefore")}
                    data-testid="trim-before"
                    onclick={(e) => {
                      e.stopPropagation();
                      void handleTrim(point.id, true);
                    }}>⇤</button
                  >
                  <button
                    class="text-muted-foreground hover:text-foreground border-0 bg-transparent px-1 leading-none"
                    title={$t("inspector.trimAfter")}
                    aria-label={$t("inspector.trimAfter")}
                    data-testid="trim-after"
                    onclick={(e) => {
                      e.stopPropagation();
                      void handleTrim(point.id, false);
                    }}>⇥</button
                  >
                </Table.Cell>
              </Table.Row>
            {/each}
          </Table.Body>
        </Table.Root>
        {#if canSplitAt(segment)}
          <div class="border-border border-b px-3 py-1.5">
            <Button
              variant="outline"
              size="xs"
              onclick={() => handleSplit(segment)}
            >
              {$t("points.splitHere")}
            </Button>
          </div>
        {/if}
        {#if paged.hiddenCount > 0}
          <div class="px-3 py-2">
            <Button
              variant="outline"
              size="sm"
              onclick={() => (expandedSegments[segment.id] = true)}
            >
              {$t("inspector.showMore").replace(
                "{count}",
                String(paged.hiddenCount),
              )}
            </Button>
          </div>
        {/if}
      {/each}
    </ScrollArea>
  {/if}
</section>
