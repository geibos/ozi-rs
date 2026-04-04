<script lang="ts">
  import {
    appState,
    editModeActive,
    trackPointsPanelOpen,
    selectedTrack,
    selectedPointId,
  } from "../lib/stores";
  import { getTrackDetail } from "../lib/api";
  import type { TrackDetail } from "../lib/types";

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
      // Reset expansion state when changing tracks
      expandedSegments = {};
    } catch (e) {
      console.error("Failed to load track details", e);
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
  <div class="panel">
    <div class="panel-header">
      <span>Track Points</span>
      <div class="header-actions">
        <button
          class:active-edit={$editModeActive}
          disabled={!$selectedTrack}
          onclick={toggleEditMode}
        >
          {$editModeActive ? "Stop Edit" : "Edit Mode"}
        </button>
        <button onclick={() => trackPointsPanelOpen.set(false)}>✕</button>
      </div>
    </div>
    <div class="panel-body">
      {#if !$selectedTrack}
        <div class="empty">Select a track to see points</div>
      {:else if !trackDetail}
        <div class="empty">Loading points...</div>
      {:else}
        {#each trackDetail.segments as segment}
          <div class="segment-group">
            <div class="segment-header">
              Segment {segment.id} ({segment.points.length} points)
            </div>
            <div class="points-list">
              {#each segment.points.slice(0, expandedSegments[segment.id] ? undefined : 1000) as point}
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div
                  class="point-row"
                  class:selected={$selectedPointId === BigInt(point.id)}
                  onclick={() => handlePointClick(point.id)}
                >
                  <span class="bullet">•</span>
                  <span class="coords">
                    {point.lat.toFixed(5)}, {point.lon.toFixed(5)}
                    {#if point.elevation !== undefined && point.elevation !== null}
                      <span class="elevation">, ele={point.elevation.toFixed(1)}m</span>
                    {/if}
                  </span>
                </div>
              {/each}
              {#if !expandedSegments[segment.id] && segment.points.length > 1000}
                <button
                  class="show-more"
                  onclick={() => expandedSegments[segment.id] = true}
                >
                  Show {segment.points.length - 1000} more
                </button>
              {/if}
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </div>
{/if}

<style>
  .panel {
    position: fixed;
    top: 60px;
    right: 310px; /* Offset to not entirely overlap the Tracks panel */
    width: 320px;
    max-height: calc(100vh - 80px);
    background: var(--ctp-mantle);
    border: 1px solid var(--ctp-surface0);
    border-radius: 6px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
    display: flex;
    flex-direction: column;
    z-index: 200;
    resize: both;
    overflow: hidden;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 6px 10px;
    background: var(--ctp-crust);
    border-bottom: 1px solid var(--ctp-surface0);
    cursor: move;
    font-size: 12px;
    color: var(--ctp-subtext1);
    user-select: none;
  }

  .panel-header button {
    background: none;
    border: none;
    color: var(--ctp-overlay1);
    padding: 0 2px;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .header-actions button {
    font-size: 10px;
    border-radius: 4px;
    border: 1px solid var(--ctp-surface1);
    padding: 2px 6px;
    color: var(--ctp-text);
    background: var(--ctp-surface0);
  }

  .header-actions button:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .header-actions .active-edit {
    border-color: var(--ctp-green);
    color: var(--ctp-green);
  }

  .panel-body {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
  }

  .empty {
    padding: 12px;
    color: var(--ctp-overlay1);
    text-align: center;
    font-size: 11px;
  }

  .segment-group {
    margin-bottom: 8px;
  }

  .segment-header {
    font-size: 10px;
    font-weight: 600;
    color: var(--ctp-overlay1);
    padding: 4px 10px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    background: var(--ctp-crust);
    border-top: 1px solid var(--ctp-surface0);
    border-bottom: 1px solid var(--ctp-surface0);
  }

  .segment-group:first-child .segment-header {
    border-top: none;
  }

  .points-list {
    display: flex;
    flex-direction: column;
    padding: 4px 0;
  }

  .point-row {
    display: flex;
    align-items: flex-start;
    gap: 6px;
    padding: 2px 10px 2px 16px;
    font-size: 11px;
    color: var(--ctp-text);
    cursor: pointer;
  }

  .point-row:hover {
    background: var(--ctp-surface0);
  }

  .point-row.selected {
    background: var(--ctp-surface1);
  }

  .bullet {
    color: var(--ctp-surface2);
  }

  .coords {
    font-family: monospace;
    white-space: pre;
  }

  .elevation {
    color: var(--ctp-subtext0);
  }

  .show-more {
    margin: 6px 16px;
    font-size: 11px;
    color: var(--ctp-text);
    background: var(--ctp-surface0);
    border: 1px solid var(--ctp-surface1);
    border-radius: 4px;
    padding: 4px 8px;
    cursor: pointer;
    align-self: flex-start;
  }

  .show-more:hover {
    background: var(--ctp-surface1);
  }
</style>
