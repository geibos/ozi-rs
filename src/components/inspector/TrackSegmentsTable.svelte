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
  } from "$lib/stores";
  import {
    formatPointCoords,
    formatPointTimestamp,
    loadTrackDetail,
    pageSegmentPoints,
    segmentHeader,
  } from "$lib/track-points";
  import { toast } from "svelte-sonner";
  import type { TrackDetail } from "$lib/types";

  let trackDetail: TrackDetail | null = $state(null);
  let expandedSegments: Record<number, boolean> = $state({});
  let lastLoaded: string | null = $state(null);

  $effect(() => {
    const selected = $selectedTrack;
    if (!selected || !$appState) {
      trackDetail = null;
      lastLoaded = null;
      return;
    }
    // Cache by composite key so quickly bouncing between rows in the
    // Library does not refetch on every store tick.
    const key = `${selected.layerId}:${selected.trackId}`;
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
</script>

<section
  class="bg-card border-border flex min-h-0 flex-col overflow-hidden rounded-[var(--radius-card)] border"
  aria-label="Segments and points"
>
  <header
    class="border-border flex items-center justify-between border-b px-3 py-2"
  >
    <span
      class="text-muted-foreground/80 text-[10px] font-semibold uppercase tracking-wider"
      >Segments / Points</span
    >
    <Button
      variant={$editModeActive ? "default" : "outline"}
      size="xs"
      disabled={!$selectedTrack}
      onclick={toggleEditMode}
    >
      {$editModeActive ? "Stop Edit" : "Edit Mode"}
    </Button>
  </header>

  {#if !$selectedTrack}
    <div class="text-muted-foreground p-3 text-center text-xs">
      Select a track to see points
    </div>
  {:else if !trackDetail}
    <div class="text-muted-foreground p-3 text-center text-xs">
      Loading points…
    </div>
  {:else if trackDetail.segments.length === 0}
    <div class="text-muted-foreground p-3 text-center text-xs">
      Track has no segments
    </div>
  {:else}
    <ScrollArea class="max-h-72 flex-1">
      {#each trackDetail.segments as segment, segIdx (segment.id)}
        {@const paged = pageSegmentPoints(
          segment,
          expandedSegments[segment.id] === true,
        )}
        <div
          class="bg-muted/50 text-muted-foreground border-border px-3 py-1 text-[10px] font-semibold tracking-wider uppercase"
          class:border-t={segIdx > 0}
          class:border-b={true}
        >
          {segmentHeader(segment)}
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
                  {#if formatPointTimestamp(point) !== null}
                    <div
                      class="text-muted-foreground font-mono text-[10px] leading-tight"
                    >
                      {formatPointTimestamp(point)}
                    </div>
                  {/if}
                </Table.Cell>
              </Table.Row>
            {/each}
          </Table.Body>
        </Table.Root>
        {#if paged.hiddenCount > 0}
          <div class="px-3 py-2">
            <Button
              variant="outline"
              size="sm"
              onclick={() => (expandedSegments[segment.id] = true)}
            >
              Show {paged.hiddenCount} more
            </Button>
          </div>
        {/if}
      {/each}
    </ScrollArea>
  {/if}
</section>
