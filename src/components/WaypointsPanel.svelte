<script lang="ts">
  import {
    appState,
    activeWaypointLayerId,
    waypointsPanelOpen,
    selectedWaypointId,
  } from "../lib/stores";
  import {
    getWaypoints,
    deleteWaypoint,
    renameWaypoint,
    setWaypointSymbol,
    toggleWaypointVisible,
  } from "../lib/api";
  import type { WaypointData } from "../lib/types";
  import SymbolPicker from "./SymbolPicker.svelte";

  let waypoints: WaypointData[] = $state([]);
  let editingWaypoint: number | null = $state(null);
  let editName = $state("");

  function focusOnMount(node: HTMLInputElement) {
    node.focus();
  }

  $effect(() => {
    const layerId = $activeWaypointLayerId;
    if ($appState && layerId !== null) {
      loadWaypoints();
    } else {
      waypoints = [];
    }
  });

  async function loadWaypoints() {
    try {
      const layerId = $activeWaypointLayerId;
      if ($appState && $appState.waypoint_layer_count > 0 && layerId !== null) {
        waypoints = await getWaypoints(layerId);
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
    const layerId = $activeWaypointLayerId;
    if (layerId === null) return;
    if (editName.trim() && editName !== wp.name) {
      await renameWaypoint(layerId, BigInt(wp.id), editName.trim());
    }
    editingWaypoint = null;
  }

  async function handleDelete(wp: WaypointData) {
    const layerId = $activeWaypointLayerId;
    if (layerId === null) return;
    await deleteWaypoint(layerId, BigInt(wp.id));
  }

  function selectWaypoint(wp: WaypointData) {
    selectedWaypointId.set(BigInt(wp.id));
  }

  async function handleSetSymbol(wp: WaypointData, symbol: string | null) {
    const layerId = $activeWaypointLayerId;
    if (layerId === null) return;
    await setWaypointSymbol(layerId, BigInt(wp.id), symbol);
  }

  async function handleToggleVisible(wp: WaypointData) {
    const layerId = $activeWaypointLayerId;
    if (layerId === null) return;
    await toggleWaypointVisible(layerId, BigInt(wp.id));
    // Optimistically update local list so the checkbox reflects the change
    // even before the `state-changed` event triggers a full refresh.
    waypoints = waypoints.map((w) =>
      w.id === wp.id ? { ...w, visible: !w.visible } : w
    );
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
          <div class="waypoint-row" class:hidden-waypoint={!wp.visible}>
            <input
              type="checkbox"
              class="visibility-toggle"
              checked={wp.visible}
              title={wp.visible ? "Hide waypoint" : "Show waypoint"}
              aria-label="Toggle waypoint visibility"
              onchange={() => handleToggleVisible(wp)}
            />
            <SymbolPicker
              symbol={wp.symbol}
              onSelect={(symbol) => handleSetSymbol(wp, symbol)}
            />

            {#if editingWaypoint === wp.id}
              <input
                class="name-input"
                bind:value={editName}
                onblur={() => commitRename(wp)}
                onkeydown={(e) => e.key === "Enter" && commitRename(wp)}
                use:focusOnMount
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

  .waypoint-row.hidden-waypoint .waypoint-name {
    color: var(--ctp-overlay1);
    font-style: italic;
  }

  .visibility-toggle {
    flex-shrink: 0;
    cursor: pointer;
    margin: 0;
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
