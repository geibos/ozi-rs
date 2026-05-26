<script lang="ts">
  /**
   * Three-pane workspace shell introduced by the `redesign-shell-layout`
   * change.
   *
   *   - `library-rail` (left, 280px) — empty placeholder in this change;
   *     `library-sidebar` fills it.
   *   - `canvas` — flexible center region where `MapView` (mounted in the
   *     root layout) becomes visible while the `/project` route is active.
   *   - `inspector-rail` (right, 360px) — mounted only when `inspectorOpen`
   *     is true. Empty placeholder in this change; `inspector-pane` fills
   *     it and replaces the placeholder store with content-driven logic.
   *
   * Above the canvas: a thin context-bar with four mode-chip placeholders
   * (View / Draw / Edit / Measure, inert) and a Cmd-K trigger button (also
   * inert — palette behaviour lands in `inspector-pane`). Below: a status
   * bar of stable height (`--status-bar-height` from `tokens.css`).
   *
   * `MapView` is NOT mounted from this component — it lives in
   * `src/routes/+layout.svelte` and is toggled visible via the workspace
   * route check there. The canvas slot is the visual region inside which
   * the layout's `MapView` shows.
   */
  import { onDestroy, type Snippet } from "svelte";
  import { commandPaletteOpen, inspectorOpen } from "$lib/stores";

  let {
    libraryRail,
    canvas,
    inspectorRail,
  }: {
    libraryRail?: Snippet;
    canvas?: Snippet;
    inspectorRail?: Snippet;
  } = $props();

  /**
   * The layout's MapView container (sole owner of MapView per the
   * `consolidate-state-event-flow` change) is positioned absolutely into
   * the canvas region via these CSS custom properties on the root element.
   * Writing them at component-mount time and clearing them on destroy keeps
   * the MapView wrapper aware of the current 3-pane geometry without
   * remounting MapView itself.
   *
   *   --canvas-left   = library rail width
   *   --canvas-right  = inspector rail width when open, else 0
   *   --canvas-top    = context-bar height
   *   --canvas-bottom = status-bar height (mirrors `--status-bar-height`)
   */
  const LIBRARY_WIDTH = "280px";
  const INSPECTOR_WIDTH = "360px";
  const CONTEXT_BAR_HEIGHT = "36px";

  function writeCanvasInsets(inspectorVisible: boolean) {
    if (typeof document === "undefined") return;
    const root = document.documentElement;
    root.style.setProperty("--canvas-left", LIBRARY_WIDTH);
    root.style.setProperty(
      "--canvas-right",
      inspectorVisible ? INSPECTOR_WIDTH : "0px",
    );
    root.style.setProperty("--canvas-top", CONTEXT_BAR_HEIGHT);
    root.style.setProperty("--canvas-bottom", "var(--status-bar-height)");
    root.setAttribute("data-workspace-active", "true");
  }

  function clearCanvasInsets() {
    if (typeof document === "undefined") return;
    const root = document.documentElement;
    root.style.removeProperty("--canvas-left");
    root.style.removeProperty("--canvas-right");
    root.style.removeProperty("--canvas-top");
    root.style.removeProperty("--canvas-bottom");
    root.removeAttribute("data-workspace-active");
  }

  $effect(() => {
    writeCanvasInsets($inspectorOpen);
  });

  onDestroy(clearCanvasInsets);
</script>

<div class="shell" class:has-inspector={$inspectorOpen}>
  <aside class="rail library" aria-label="Library">
    {#if libraryRail}
      {@render libraryRail()}
    {:else}
      <div class="placeholder" aria-hidden="true">Library</div>
    {/if}
  </aside>

  <div class="canvas-column">
    <div class="context-bar">
      <div class="mode-chips" role="group" aria-label="Mode (inert placeholder)">
        <button type="button" class="chip" tabindex="-1">View</button>
        <button type="button" class="chip" tabindex="-1">Draw</button>
        <button type="button" class="chip" tabindex="-1">Edit</button>
        <button type="button" class="chip" tabindex="-1">Measure</button>
      </div>
      <button
        type="button"
        class="cmdk-trigger"
        aria-label="Open command palette"
        onclick={() => commandPaletteOpen.set(true)}
      >
        <span class="cmdk-label">Search…</span>
        <kbd class="cmdk-glyph">⌘K</kbd>
      </button>
    </div>

    <main class="canvas" aria-label="Map canvas">
      {#if canvas}
        {@render canvas()}
      {/if}
    </main>

    <div class="status-bar" data-testid="workspace-status-bar"></div>
  </div>

  {#if $inspectorOpen}
    <aside class="rail inspector" aria-label="Inspector">
      {#if inspectorRail}
        {@render inspectorRail()}
      {:else}
        <div class="placeholder" aria-hidden="true">Inspector</div>
      {/if}
    </aside>
  {/if}
</div>

<style>
  .shell {
    flex: 1;
    min-width: 0;
    display: grid;
    grid-template-columns: 280px minmax(0, 1fr);
    grid-template-rows: 1fr;
    height: 100%;
    background: hsl(var(--background));
    color: hsl(var(--foreground));
  }

  .shell.has-inspector {
    grid-template-columns: 280px minmax(0, 1fr) 360px;
  }

  .rail {
    min-width: 0;
    overflow: hidden;
    box-shadow: inset -1px 0 0 var(--inner-border);
    background: hsl(var(--card));
  }

  .rail.inspector {
    box-shadow: inset 1px 0 0 var(--inner-border);
  }

  .placeholder {
    height: 100%;
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: hsl(var(--muted-foreground));
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    opacity: 0.5;
  }

  .canvas-column {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr) var(--status-bar-height);
    min-width: 0;
    min-height: 0;
  }

  .context-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 6px 10px;
    height: 36px;
    background: hsl(var(--background));
    box-shadow: inset 0 -1px 0 var(--inner-border);
    flex-shrink: 0;
  }

  .mode-chips {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .chip {
    appearance: none;
    background: transparent;
    border: 1px solid transparent;
    color: hsl(var(--muted-foreground));
    font-size: 12px;
    font-weight: 500;
    line-height: 1;
    padding: 5px 10px;
    border-radius: var(--radius-pill);
    cursor: default;
    user-select: none;
    transition: background 0.1s, color 0.1s;
  }

  .chip:hover {
    background: hsl(var(--secondary));
    color: hsl(var(--secondary-foreground));
  }

  .cmdk-trigger {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    background: hsl(var(--card));
    color: hsl(var(--muted-foreground));
    border: 1px solid hsl(var(--border));
    border-radius: var(--radius-pill);
    padding: 4px 6px 4px 12px;
    font-size: 12px;
    cursor: pointer;
    min-width: 200px;
    justify-content: space-between;
  }

  .cmdk-trigger:hover {
    background: hsl(var(--accent));
  }

  .cmdk-label {
    flex: 1;
    text-align: left;
  }

  .cmdk-glyph {
    font-family: var(--font-mono);
    font-size: 10px;
    background: hsl(var(--muted));
    color: hsl(var(--muted-foreground));
    border: 1px solid hsl(var(--border));
    border-radius: 4px;
    padding: 1px 5px;
    line-height: 1;
  }

  .canvas {
    position: relative;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  .status-bar {
    background: hsl(var(--card));
    box-shadow: inset 0 1px 0 var(--inner-border);
    color: hsl(var(--muted-foreground));
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    height: var(--status-bar-height);
    display: flex;
    align-items: center;
    padding: 0 10px;
    flex-shrink: 0;
  }
</style>
