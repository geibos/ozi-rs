<script lang="ts">
  /**
   * InspectorRail — context-sensitive right rail (Decision 1: always
   * mounted, body conditionally rendered).
   *
   * Mounted unconditionally by `WorkspaceShell` in the `inspector-rail`
   * snippet slot. Body presence is driven by selection stores:
   *
   *   - `$selectedTrack`        → `<TrackInspector />`
   *   - `$selectedWaypointId`   → `<WaypointInspector />`
   *   - `$selectedMapInfo`      → `<MapInspector />`
   *
   * Auto-expand on selection transition (Decision 2). The rail keeps its
   * pinned state local — `pinned` is not exported. When pinned the rail
   * stays expanded even after the selection drops to null.
   *
   * The rail also drives the shared `$inspectorOpen` store so
   * `WorkspaceShell` re-flows the canvas insets (the `--canvas-right` CSS
   * variable). That keeps the MapView container aware of the right inset.
   */
  import PinIcon from "@lucide/svelte/icons/pin";
  import { t } from "$lib/i18n";
  import PinOffIcon from "@lucide/svelte/icons/pin-off";
  import {
    inspectorOpen,
    selectedMapInfo,
    selectedTrack,
    selectedWaypointId,
  } from "$lib/stores";
  import TrackInspector from "./inspector/TrackInspector.svelte";
  import WaypointInspector from "./inspector/WaypointInspector.svelte";
  import MapInspector from "./inspector/MapInspector.svelte";

  let pinned = $state(false);

  /**
   * Derived "what does the rail render?" key. `null` means no selection;
   * `track | waypoint | map` selects the matching subcomponent.
   */
  const activeKind: "track" | "waypoint" | "map" | null = $derived.by(() => {
    if ($selectedTrack) return "track";
    if ($selectedWaypointId !== null) return "waypoint";
    if ($selectedMapInfo) return "map";
    return null;
  });

  // Selection drives the open store: rail expanded iff selection present OR
  // user pinned it. We keep `$inspectorOpen` as the layout-visible truth so
  // `WorkspaceShell` re-flows the canvas insets without local custom CSS.
  $effect(() => {
    const open = activeKind !== null || pinned;
    inspectorOpen.set(open);
  });

  function togglePin() {
    pinned = !pinned;
  }
</script>

{#if activeKind === null && !pinned}
  <!--
    Collapsed state — the rail aside is an 8px-wide vertical strip on the
    right edge. The handle widens to ≈12px on hover (negative margin keeps
    grid layout stable so the canvas does not reflow when the user hovers).
    Clicking the pin button expands the rail; subsequent selection clearing
    keeps it open (pin == true).
  -->
  <div class="inspector-rail-collapsed" data-testid="inspector-rail-collapsed">
    <button
      type="button"
      class="edge-handle"
      onclick={togglePin}
      aria-label={$t("shell.pinInspector")}
      title="Pin inspector open"
    >
      <span class="edge-handle-pin">
        <PinIcon class="size-3.5" />
      </span>
    </button>
  </div>
{:else}
  <div class="inspector-rail-inner" data-testid="inspector-rail">
    <header class="rail-header">
      <span class="rail-label">Inspector</span>
      <button
        type="button"
        class="pin-button"
        onclick={togglePin}
        aria-pressed={pinned}
        aria-label={pinned ? "Unpin inspector" : "Pin inspector open"}
        title={pinned ? "Unpin inspector" : "Pin inspector open"}
      >
        {#if pinned}
          <PinOffIcon class="size-3.5" />
        {:else}
          <PinIcon class="size-3.5" />
        {/if}
      </button>
    </header>

    <div class="rail-body" data-active-kind={activeKind ?? "none"}>
      {#if activeKind === "track"}
        <TrackInspector />
      {:else if activeKind === "waypoint"}
        <WaypointInspector />
      {:else if activeKind === "map"}
        <MapInspector />
      {:else}
        <div class="empty-state">
          <p class="empty-title">Nothing selected</p>
          <p class="empty-hint">
            Pick a Track, Waypoint, or Map from the Library to inspect its
            properties.
          </p>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  /*
   * Collapsed state — an 8px-wide column hosting a button that fills the
   * column and widens to 12px on hover/focus. The button is the manual
   * "pin inspector open" affordance the spec contracts for. Cursor reads
   * as ew-resize (drag-style) to hint at the open-rail gesture.
   */
  .inspector-rail-collapsed {
    height: 100%;
    width: 8px;
    position: relative;
  }

  .edge-handle {
    appearance: none;
    position: absolute;
    top: 0;
    right: 0;
    height: 100%;
    width: 8px;
    padding: 0;
    background: hsl(var(--card));
    border: none;
    border-left: 1px solid var(--inner-border);
    color: hsl(var(--muted-foreground));
    cursor: ew-resize;
    display: flex;
    align-items: center;
    justify-content: center;
    transition:
      width 0.12s ease,
      background 0.12s ease,
      color 0.12s ease;
  }

  .edge-handle:hover,
  .edge-handle:focus-visible {
    width: 12px;
    background: hsl(var(--secondary));
    color: hsl(var(--secondary-foreground));
    outline: none;
  }

  .edge-handle-pin {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    opacity: 0;
    transition: opacity 0.12s ease;
  }

  .edge-handle:hover .edge-handle-pin,
  .edge-handle:focus-visible .edge-handle-pin {
    opacity: 1;
  }

  .inspector-rail-inner {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    animation: rail-slide-in 240ms ease-out;
  }

  @keyframes rail-slide-in {
    from {
      transform: translateX(8px);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }

  .rail-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    height: 32px;
    flex-shrink: 0;
    box-shadow: inset 0 -1px 0 var(--inner-border);
  }

  .rail-label {
    color: hsl(var(--muted-foreground));
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    font-weight: 600;
  }

  .pin-button {
    appearance: none;
    background: transparent;
    border: 1px solid transparent;
    color: hsl(var(--muted-foreground));
    border-radius: 6px;
    width: 22px;
    height: 22px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition:
      background 0.12s,
      color 0.12s;
  }

  .pin-button:hover {
    background: hsl(var(--secondary));
    color: hsl(var(--secondary-foreground));
  }

  .pin-button[aria-pressed="true"] {
    color: hsl(var(--primary));
  }

  .rail-body {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .empty-state {
    margin: auto;
    padding: 24px 20px;
    text-align: center;
    max-width: 240px;
  }

  .empty-title {
    color: hsl(var(--foreground));
    font-size: 13px;
    font-weight: 500;
    margin: 0 0 6px 0;
  }

  .empty-hint {
    color: hsl(var(--muted-foreground));
    font-size: 11px;
    line-height: 1.5;
    margin: 0;
  }
</style>
