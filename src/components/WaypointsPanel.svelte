<script lang="ts">
  import { appState, waypointsPanelOpen, selectedWaypointId } from "../lib/stores";
  import { getWaypoints, deleteWaypoint, renameWaypoint } from "../lib/api";
  import type { WaypointData } from "../lib/types";

  // Note: MVP single-layer assumption.
  // We'll hardcode layerId 1n until backend fully provides layer ID list.
  const currentLayerId = 1n; // TODO: multi-layer

  let waypoints: WaypointData[] = $state([]);
  let editingWaypoint: number | null = $state(null);
  let editName = $state("");

  const SYMBOL_ICONS: Record<string, string> = {
    flag: '🏁',
    camp: '🏕️',
    danger: '⚠️',
    water: '💧',
    shelter: '🏠',
    'meeting-point': '👥',
    start: '🟢',
    finish: '🏁',
    viewpoint: '👁️',
    parking: '🅿️',
  };

  function symbolIcon(symbol?: string): string {
    return symbol ? (SYMBOL_ICONS[symbol] ?? '📍') : '📍';
  }

  $effect(() => {
    if ($appState) {
      loadWaypoints();
    }
  });

  async function loadWaypoints() {
    try {
      if ($appState && $appState.waypoint_layer_count > 0) {
        waypoints = await getWaypoints(currentLayerId);
      } else {
        waypoints = [];
      }
    } catch (e) {
      console.error("Failed to load waypoints", e);
      waypoints = [];
    }
  }

  function startRename(wp: WaypointData) {
    editingWaypoint = wp.id;
    editName = wp.name;
  }

  async function commitRename(wp: WaypointData) {
    if (editName.trim() && editName !== wp.name) {
      await renameWaypoint(currentLayerId, BigInt(wp.id), editName.trim());
    }
    editingWaypoint = null;
  }

  async function handleDelete(wp: WaypointData) {
    await deleteWaypoint(currentLayerId, BigInt(wp.id));
  }

  function selectWaypoint(wp: WaypointData) {
    selectedWaypointId.set(BigInt(wp.id));
  }
</script>

{#if $waypointsPanelOpen}
  <div class="panel">
    <div class="panel-header">
      <span>Waypoints ({waypoints.length})</span>
      <button onclick={() => waypointsPanelOpen.set(false)}>✕</button>
    </div>
    <div class="panel-body">
      {#if waypoints.length === 0}
        <div class="empty">No waypoints</div>
      {:else}
        {#each waypoints as wp (wp.id)}
          <div class="waypoint-row">
            <span class="symbol-icon" title={wp.symbol ?? "default"}>
              {symbolIcon(wp.symbol)}
            </span>
            <!-- TODO: SymbolPicker integration -->

            {#if editingWaypoint === wp.id}
              <input
                class="name-input"
                bind:value={editName}
                onblur={() => commitRename(wp)}
                onkeydown={(e) => e.key === "Enter" && commitRename(wp)}
                autofocus
              />
            {:else}
              <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
              <span
                class="waypoint-name"
                onclick={() => selectWaypoint(wp)}
                ondblclick={() => startRename(wp)}
                title="Click to select, double-click to rename"
              >{wp.name}</span>
            {/if}

            <div class="actions">
              <button
                class="icon-btn danger"
                title="Delete waypoint"
                onclick={() => handleDelete(wp)}
              >🗑</button>
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
    right: 310px; /* offset from tracks panel which is right: 16px, width: 280px */
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

  .waypoint-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    transition: background 0.1s;
  }

  .waypoint-row:hover {
    background: var(--ctp-surface0);
  }

  .symbol-icon {
    font-size: 14px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
  }

  .waypoint-name {
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
    width: 100%;
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
    cursor: pointer;
  }

  .icon-btn:hover {
    color: var(--ctp-text);
  }

  .icon-btn.danger:hover {
    color: var(--ctp-red);
  }
</style>