<script lang="ts">
  import { appState, tracksPanelOpen, selectedTrack } from "../lib/stores";
  import { getTracksGeojson, renameTrack, toggleTrackVisible, exportGpx } from "../lib/api";
  import { open } from "@tauri-apps/plugin-dialog";

  interface TrackFeature {
    layerId: bigint;
    trackId: bigint;
    name: string;
    color: string;
    visible: boolean;
  }

  let tracks: TrackFeature[] = $state([]);
  let editingTrack: bigint | null = $state(null);
  let editName = $state("");

  $effect(() => {
    if ($appState) {
      loadTracks();
    }
  });

  async function loadTracks() {
    const geojson = await getTracksGeojson();
    tracks = geojson.features
      .filter((f) => f.geometry.type === "LineString")
      .map((f) => ({
        layerId: BigInt(f.properties!.layer_id as number),
        trackId: BigInt(f.properties!.track_id as number),
        name: f.properties!.name as string,
        color: f.properties!.color as string,
        visible: f.properties!.visible as boolean,
      }));
  }

  function startRename(track: TrackFeature) {
    editingTrack = track.trackId;
    editName = track.name;
  }

  async function commitRename(track: TrackFeature) {
    if (editName.trim() && editName !== track.name) {
      await renameTrack(track.layerId, track.trackId, editName.trim());
    }
    editingTrack = null;
  }

  async function handleToggleVisible(track: TrackFeature) {
    await toggleTrackVisible(track.layerId, track.trackId);
  }

  async function handleExport(track: TrackFeature) {
    const path = await open({
      save: true,
      defaultPath: `${track.name}.gpx`,
      filters: [{ name: "GPX", extensions: ["gpx"] }],
    } as Parameters<typeof open>[0]);
    if (path) {
      await exportGpx(track.layerId, path as string);
    }
  }
</script>

{#if $tracksPanelOpen}
  <div class="panel">
    <div class="panel-header">
      <span>Tracks ({tracks.length})</span>
      <button onclick={() => tracksPanelOpen.set(false)}>✕</button>
    </div>
    <div class="panel-body">
      {#if tracks.length === 0}
        <div class="empty">No tracks loaded</div>
      {:else}
        {#each tracks as track (track.trackId)}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div 
            class="track-row" 
            class:hidden={!track.visible}
            class:selected={$selectedTrack?.trackId === track.trackId}
            onclick={() => selectedTrack.set({ layerId: track.layerId, trackId: track.trackId })}
          >
            <span
              class="color-dot"
              style="background: {track.color}"
            ></span>

            {#if editingTrack === track.trackId}
              <input
                class="name-input"
                bind:value={editName}
                onblur={() => commitRename(track)}
                onkeydown={(e) => e.key === "Enter" && commitRename(track)}
                autofocus
              />
            {:else}
              <span
                class="track-name"
                ondblclick={() => startRename(track)}
                title="Double-click to rename"
              >{track.name}</span>
            {/if}

            <div class="actions">
              <button
                class="icon-btn"
                title={track.visible ? "Hide" : "Show"}
                onclick={() => handleToggleVisible(track)}
              >
                {track.visible ? "👁" : "🙈"}
              </button>
              <button
                class="icon-btn"
                title="Export to GPX"
                onclick={() => handleExport(track)}
              >↓</button>
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
    right: 16px;
    width: 280px;
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

  .track-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    transition: background 0.1s;
    cursor: pointer;
  }

  .track-row:hover {
    background: var(--ctp-surface0);
  }

  .track-row.selected {
    background: var(--ctp-surface1);
  }

  .track-row.hidden {
    opacity: 0.4;
  }

  .color-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .track-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 12px;
    cursor: text;
  }

  .name-input {
    flex: 1;
    font-size: 12px;
    padding: 1px 4px;
  }

  .actions {
    display: flex;
    gap: 2px;
    flex-shrink: 0;
  }

  .icon-btn {
    background: none;
    border: none;
    color: var(--ctp-overlay1);
    padding: 0 3px;
    font-size: 11px;
  }

  .icon-btn:hover {
    color: var(--ctp-text);
  }
</style>
