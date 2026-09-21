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
  import UndoIcon from "@lucide/svelte/icons/undo-2";
  import RedoIcon from "@lucide/svelte/icons/redo-2";
  import SaveIcon from "@lucide/svelte/icons/save";
  import {
    bundleProgress,
    commandPaletteOpen,
    inspectorOpen,
    projectDirty,
  } from "$lib/stores";
  import { doRedo, doUndo, quickSave } from "$lib/actions/project";
  import { locale, progressText, t, toggleLocale } from "$lib/i18n";

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
  const INSPECTOR_COLLAPSED_WIDTH = "8px";
  const CONTEXT_BAR_HEIGHT = "36px";

  function writeCanvasInsets(inspectorVisible: boolean) {
    if (typeof document === "undefined") return;
    const root = document.documentElement;
    root.style.setProperty("--canvas-left", LIBRARY_WIDTH);
    root.style.setProperty(
      "--canvas-right",
      inspectorVisible ? INSPECTOR_WIDTH : INSPECTOR_COLLAPSED_WIDTH,
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
      <div
        class="mode-chips"
        role="group"
        aria-label="Mode (inert placeholder)"
      >
        <button type="button" class="chip" tabindex="-1" aria-disabled="true"
          >{$t("shell.modeView")}</button
        >
        <button type="button" class="chip" tabindex="-1" aria-disabled="true"
          >{$t("shell.modeDraw")}</button
        >
        <button type="button" class="chip" tabindex="-1" aria-disabled="true"
          >{$t("shell.modeEdit")}</button
        >
        <button type="button" class="chip" tabindex="-1" aria-disabled="true"
          >{$t("shell.modeMeasure")}</button
        >
        <span class="chips-divider" aria-hidden="true"></span>
      </div>
      <!--
        Bug 5 fix (fix-redesign-functional-bugs): the previous "Search…"
        copy made the trigger read as a text input — users typed into it
        and got no response. The trigger is now visually a button: a
        "Command palette" label paired with a key-cap `⌘K` chord, no
        input-like ring, no caret. Activating it opens the
        `CommandPalette` dialog whose own `cmdk` Command.Input takes
        focus.
      -->
      <!--
        CJ-7 continuity controls: undo / redo / save plus a dirty
        indicator. All actions run through the shared helpers in
        `$lib/actions/project` — the same path the global keyboard chords
        (Cmd+S / Cmd+Z / Cmd+Shift+Z in `+layout.svelte`) and the command
        palette use. `projectDirty` mirrors `AppStateDto.project_dirty`.
      -->
      <div class="bar-actions">
        <span
          class="dirty-indicator"
          class:dirty={$projectDirty}
          data-testid="dirty-indicator"
          role="status"
        >
          {#if $projectDirty}
            <span class="dirty-dot" aria-hidden="true"></span>
            {$t("shell.unsaved")}
          {:else}
            {$t("shell.saved")}
          {/if}
        </span>
        <button
          type="button"
          class="bar-btn"
          title={$t("shell.undo")}
          aria-label={$t("shell.undo")}
          onclick={() => void doUndo()}
        >
          <UndoIcon class="size-3.5" />
        </button>
        <button
          type="button"
          class="bar-btn"
          title={$t("shell.redo")}
          aria-label={$t("shell.redo")}
          onclick={() => void doRedo()}
        >
          <RedoIcon class="size-3.5" />
        </button>
        <button
          type="button"
          class="bar-btn bar-btn-save"
          title={`${$t("shell.save")} (⌘S)`}
          onclick={() => void quickSave()}
        >
          <SaveIcon class="size-3.5" />
          <span>{$t("shell.save")}</span>
        </button>
        <span class="chips-divider" aria-hidden="true"></span>
        <button
          type="button"
          class="cmdk-trigger"
          aria-label={$t("shell.palette")}
          title={`${$t("shell.palette")} (⌘K)`}
          onclick={() => commandPaletteOpen.set(true)}
        >
          <span class="cmdk-label">{$t("shell.palette")}</span>
          <kbd class="cmdk-glyph">⌘K</kbd>
        </button>
      </div>
    </div>

    <main class="canvas" aria-label="Map canvas">
      {#if canvas}
        {@render canvas()}
      {/if}
    </main>

    <div class="status-bar" data-testid="workspace-status-bar">
      {#if $bundleProgress}
        <!--
          Bug 3 fix (fix-redesign-functional-bugs): the status bar now
          mirrors the bundle-loader's progress line whenever a download is
          in flight. The `bundleProgress` store is fed by the layout-level
          `bundle-progress` listener (single-owner rule); no new
          subscription is registered here.
        -->
        <span class="status-progress" data-testid="status-bundle-progress">
          <span class="status-phase"
            >{$t(`phase.${$bundleProgress.phase}`)}</span
          >
          <span class="status-message"
            >{$progressText(
              $bundleProgress.message_key,
              $bundleProgress.message_args,
              $bundleProgress.message,
            )}</span
          >
          {#if $bundleProgress.completed != null && $bundleProgress.total != null}
            <span class="status-counts">
              {$bundleProgress.completed} / {$bundleProgress.total}
            </span>
          {/if}
        </span>
      {/if}

      <!-- The language switch lived only in the Cmd-K palette, which means a
           Russian-speaking crew had to know the palette exists to get a
           Russian interface. It belongs where it can be found. -->
      <button
        type="button"
        class="status-lang"
        title={$t("shell.language")}
        aria-label={$t("shell.language")}
        onclick={toggleLocale}
        data-testid="status-language"
      >
        {$locale === "ru" ? "RU" : "EN"}
      </button>
    </div>
  </div>

  <aside
    class="rail inspector"
    class:collapsed={!$inspectorOpen}
    aria-label="Inspector"
  >
    {#if inspectorRail}
      {@render inspectorRail()}
    {:else}
      <div class="placeholder" aria-hidden="true">Inspector</div>
    {/if}
  </aside>
</div>

<style>
  .shell {
    flex: 1;
    min-width: 0;
    display: grid;
    grid-template-columns: 280px minmax(0, 1fr) 8px;
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

  .rail.inspector.collapsed {
    /* Visually a narrow strip; InspectorRail renders the edge handle inside.
       overflow: visible lets the 8px → 12px hover widen the handle without
       pushing the grid column (the column stays 8px wide).            */
    background: transparent;
    box-shadow: none;
    overflow: visible;
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

  .chips-divider {
    display: inline-block;
    width: 1px;
    height: 16px;
    margin: 0 6px;
    background: var(--inner-border);
    flex-shrink: 0;
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
    cursor: not-allowed;
    user-select: none;
    opacity: 0.55;
    transition: none;
  }

  .chip[aria-disabled="true"]:hover {
    background: transparent;
    color: hsl(var(--muted-foreground));
  }

  /* CJ-7 continuity controls (right side of the context bar). */
  .bar-actions {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    min-width: 0;
  }

  .bar-btn {
    appearance: none;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-pill);
    color: hsl(var(--muted-foreground));
    font-size: 12px;
    font-weight: 500;
    line-height: 1;
    padding: 5px 8px;
    cursor: pointer;
    user-select: none;
    transition:
      background 0.12s ease,
      color 0.12s ease,
      box-shadow 0.12s ease;
  }

  .bar-btn:hover {
    background: hsl(var(--secondary));
    color: hsl(var(--secondary-foreground));
  }

  .bar-btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px hsl(var(--ring) / 0.45);
  }

  .bar-btn-save {
    padding: 5px 10px;
  }

  /* Unobtrusive save-state readout: quiet "Saved" when clean, a warning
     dot + label while `projectDirty` is true. */
  .dirty-indicator {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: hsl(var(--muted-foreground));
    opacity: 0.6;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-right: 4px;
  }

  .dirty-indicator.dirty {
    color: hsl(var(--foreground));
    opacity: 1;
  }

  .dirty-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: hsl(var(--primary));
    flex-shrink: 0;
  }

  /*
   * Cmd-K trigger — explicitly a button, NOT an input.
   *
   * Discipline (per `fix-redesign-functional-bugs` ui-shell spec):
   *   - No caret, no `cursor: text` on the trigger or its children.
   *   - No focus ring that mimics input focus (use shell button affordance).
   *   - Label is "Command palette", NOT "Search…" with a placeholder
   *     ellipsis that reads as an input awaiting text.
   *   - The `⌘K` chord is a `<kbd>` styled as a key-cap badge,
   *     visually heavier than placeholder copy.
   *   - Hover / focus use background tint + button-style ring matching
   *     other workspace buttons.
   */
  .cmdk-trigger {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    background: transparent;
    color: hsl(var(--muted-foreground));
    border: 1px solid transparent;
    border-radius: var(--radius-pill);
    padding: 4px 6px 4px 12px;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    user-select: none;
    line-height: 1;
    transition:
      background 0.12s ease,
      color 0.12s ease,
      box-shadow 0.12s ease;
  }

  .cmdk-trigger:hover {
    background: hsl(var(--secondary));
    color: hsl(var(--secondary-foreground));
  }

  .cmdk-trigger:focus-visible {
    outline: none;
    border-color: transparent;
    box-shadow: 0 0 0 2px hsl(var(--ring) / 0.45);
  }

  .cmdk-label {
    cursor: pointer;
  }

  .cmdk-glyph {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 600;
    background: hsl(var(--muted));
    color: hsl(var(--muted-foreground));
    border: 1px solid hsl(var(--border));
    border-bottom-width: 2px;
    border-radius: 4px;
    padding: 1px 6px;
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
    gap: 8px;
  }

  .status-lang {
    margin-left: auto;
    appearance: none;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 4px;
    color: hsl(var(--muted-foreground));
    cursor: pointer;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.04em;
    padding: 1px 5px;
  }

  .status-lang:hover {
    background: hsl(var(--accent));
    color: hsl(var(--accent-foreground));
  }

  .status-progress {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .status-phase {
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: hsl(var(--primary));
    font-weight: 600;
  }

  .status-message {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .status-counts {
    color: hsl(var(--foreground));
    font-variant-numeric: tabular-nums;
  }
</style>
