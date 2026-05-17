<script lang="ts">
  import XIcon from "@lucide/svelte/icons/x";
  import { Button } from "$lib/components/ui/button";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import * as Table from "$lib/components/ui/table";
  import * as Tooltip from "$lib/components/ui/tooltip";
  import {
    appState,
    editModeActive,
    trackPointsPanelOpen,
    selectedTrack,
    selectedPointId,
  } from "$lib/stores";
  import { getTrackDetail } from "$lib/api";
  import { toast } from "svelte-sonner";
  import type { TrackDetail } from "$lib/types";

  let trackDetail: TrackDetail | null = $state(null);
  let expandedSegments: Record<number, boolean> = $state({});

  $effect(() => {
    if ($appState && $selectedTrack) {
      loadDetail($selectedTrack.layerId, $selectedTrack.trackId);
    } else if (!$selectedTrack) {
      trackDetail = null;
    }
  });

  async function loadDetail(layerId: bigint, trackId: bigint) {
    try {
      trackDetail = await getTrackDetail(layerId, trackId);
      expandedSegments = {};
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

{#if $trackPointsPanelOpen}
  <div
    class="bg-popover text-popover-foreground border-border fixed top-[60px] right-[310px] z-[200] flex max-h-[calc(100vh-80px)] w-80 resize flex-col overflow-hidden rounded-lg border shadow-lg"
  >
    <div
      class="bg-card text-muted-foreground border-border flex cursor-move items-center justify-between border-b px-2.5 py-1 text-xs select-none"
    >
      <span>Track Points</span>
      <div class="flex items-center gap-1.5">
          <Tooltip.Root>
            <Tooltip.Trigger>
              {#snippet child({ props })}
                <Button
                  {...props}
                  variant={$editModeActive ? "default" : "outline"}
                  size="xs"
                  disabled={!$selectedTrack}
                  onclick={toggleEditMode}
                >
                  {$editModeActive ? "Stop Edit" : "Edit Mode"}
                </Button>
              {/snippet}
            </Tooltip.Trigger>
            <Tooltip.Content>
              {$selectedTrack
                ? "Toggle drag-to-edit on the selected track"
                : "Select a track to enable edit mode"}
            </Tooltip.Content>
          </Tooltip.Root>
        <Button
          variant="ghost"
          size="icon-xs"
          onclick={() => trackPointsPanelOpen.set(false)}
          aria-label="Close track points panel"
        >
          <XIcon />
        </Button>
      </div>
    </div>

    {#if !$selectedTrack}
      <div class="text-muted-foreground p-3 text-center text-xs">
        Select a track to see points
      </div>
    {:else if !trackDetail}
      <div class="text-muted-foreground p-3 text-center text-xs">
        Loading points…
      </div>
    {:else}
      <ScrollArea class="flex-1">
        {#each trackDetail.segments as segment, segIdx (segment.id)}
          <div
            class="bg-card text-muted-foreground border-border px-2.5 py-1 text-[10px] font-semibold tracking-wider uppercase"
            class:border-t={segIdx > 0}
            class:border-b={true}
          >
            Segment {segment.id} ({segment.points.length} points)
          </div>
          <Table.Root>
            <Table.Body>
              {#each segment.points.slice(0, expandedSegments[segment.id] ? undefined : 1000) as point (point.id)}
                <Table.Row
                  data-state={$selectedPointId === BigInt(point.id)
                    ? "selected"
                    : undefined}
                  class="cursor-pointer text-[11px]"
                  onclick={() => handlePointClick(point.id)}
                >
                  <Table.Cell class="text-border w-4 px-2 py-1">•</Table.Cell>
                  <Table.Cell class="px-2 py-1 font-mono">
                    {point.lat.toFixed(5)}, {point.lon.toFixed(5)}
                    {#if point.elevation !== undefined && point.elevation !== null}
                      <span class="text-muted-foreground">
                        , ele={point.elevation.toFixed(1)}m
                      </span>
                    {/if}
                    {#if point.timestamp}
                      <div
                        class="text-muted-foreground font-mono text-[10px] leading-tight"
                      >
                        {point.timestamp}
                      </div>
                    {/if}
                  </Table.Cell>
                </Table.Row>
              {/each}
            </Table.Body>
          </Table.Root>
          {#if !expandedSegments[segment.id] && segment.points.length > 1000}
            <div class="px-3 py-2">
              <Button
                variant="outline"
                size="sm"
                onclick={() => (expandedSegments[segment.id] = true)}
              >
                Show {segment.points.length - 1000} more
              </Button>
            </div>
          {/if}
        {/each}
      </ScrollArea>
    {/if}
  </div>
{/if}
