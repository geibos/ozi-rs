<script lang="ts">
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { page } from "$app/state";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { confirm as confirmDialog } from "@tauri-apps/plugin-dialog";
  import {
    appendProjectsChunk,
    appState,
    bundleProgress,
    commandPaletteOpen,
    currentDownload,
    projectDirty,
    resetBundleDownloadState,
    updateDownloadProgress,
  } from "../lib/stores";
  import { loadProjects } from "../lib/api";
  import { doRedo, doUndo, quickSave } from "$lib/actions/project";
  import { t } from "$lib/i18n";
  import { installIpcErrorToastObserver } from "../lib/ipc";
  import { applyStoredTheme, installAutoThemeListener } from "../lib/theme";
  import MapView from "../components/MapView.svelte";
  import CommandPalette from "../components/CommandPalette.svelte";
  import Console from "../components/Console.svelte";
  import { Toaster } from "$lib/components/ui/sonner";
  import * as Tooltip from "$lib/components/ui/tooltip";
  import type {
    BundleProgressPayload,
    DownloadProgressPayload,
    LizaProjectSummaryDto,
  } from "../lib/types";
  import "../app.css";

  let { children } = $props();

  const isWorkspace = $derived(page.url.pathname === "/project");

  applyStoredTheme();

  onMount(() => {
    let cancelled = false;
    const unlistenAutoTheme = installAutoThemeListener();
    // Mirrors the `ipc-error` class that `invokeIpc` puts on dev rejection
    // toasts onto `data-testid="ipc-error"` on the Sonner `<li>`, because
    // svelte-sonner does not forward `data-*` attributes from `toast({ … })`
    // options through to the rendered toast root.
    const uninstallIpcObserver = installIpcErrorToastObserver();
    const unlistens: Array<() => void> = [];

    (async () => {
      // Register every Tauri event listener BEFORE the first IPC call that
      // can race a backend emit. `load_projects` returns synchronously, but
      // its background thread emits `projects-chunk` (from cached items)
      // and `state-changed` (on refresh completion) almost immediately —
      // missing either would leave the catalog empty until the next
      // refresh. The single-owner rule (`consolidate-state-event-flow`
      // change) means this layout is the only place that subscribes to
      // these events; page-level components read the stores below.
      const subscribers = await Promise.all([
        listen<void>("state-changed", () => {
          void appState.refresh();
        }),
        listen<DownloadProgressPayload>("download-progress", (event) => {
          updateDownloadProgress(event.payload);
          currentDownload.set(event.payload);
        }),
        listen<BundleProgressPayload>("bundle-progress", (event) => {
          bundleProgress.set(event.payload);
        }),
        listen<LizaProjectSummaryDto[]>("projects-chunk", (event) =>
          appendProjectsChunk(event.payload),
        ),
      ]);

      if (cancelled) {
        subscribers.forEach((fn) => fn());
        return;
      }
      unlistens.push(...subscribers);

      // Initial state pull + single-shot startup catalog load. The
      // listener above is already armed, so the `state-changed` emit
      // that load_projects fires on completion is caught.
      await appState.refresh();
      loadProjects().catch(() => {});
    })();

    // CJ-7 close guard: intercept a window close while the project has
    // unsaved changes. `onCloseRequested` defers the close to JS, so the
    // clean-path (not dirty) simply returns and the wrapper destroys the
    // window; the dirty path must call `preventDefault()` BEFORE the first
    // await so the wrapper sees it, then re-close via `destroy()` when the
    // user confirms quitting without saving.
    (async () => {
      try {
        const unlistenClose = await getCurrentWindow().onCloseRequested(
          async (event) => {
            if (!get(projectDirty)) return; // clean — allow the close
            event.preventDefault();
            const translate = get(t);
            const quit = await confirmDialog(translate("closeGuard.message"), {
              title: translate("closeGuard.title"),
              kind: "warning",
              okLabel: translate("closeGuard.quit"),
              cancelLabel: translate("closeGuard.cancel"),
            });
            if (quit) await getCurrentWindow().destroy();
          },
        );
        if (cancelled) unlistenClose();
        else unlistens.push(unlistenClose);
      } catch {
        // Not running inside Tauri (vitest / SSR prerender) — nothing to
        // guard, and the app-level close flow is unaffected.
      }
    })();

    return () => {
      cancelled = true;
      unlistens.forEach((fn) => fn());
      unlistenAutoTheme();
      uninstallIpcObserver();
      // Reset the bundle-loader transient stores so a relaunch (or a
      // hot-reload during dev) starts from a clean slate. activeDownloadId
      // is reset together with the progress/payload stores.
      resetBundleDownloadState(null);
    };
  });

  /**
   * True when the event originates inside a text-editing surface. The CJ-7
   * chords (Cmd+S / Cmd+Z / Cmd+Shift+Z) must not hijack native text
   * editing — Cmd+Z while renaming a track stays the input's own undo.
   */
  function isEditableTarget(target: EventTarget | null): boolean {
    const el = target instanceof HTMLElement ? target : null;
    return Boolean(
      el?.closest(
        'input, textarea, select, [contenteditable="true"], [contenteditable=""]',
      ),
    );
  }

  /**
   * Global keyboard chords (CJ-7):
   *   - Cmd/Ctrl+K       — toggle the command palette (existing behavior;
   *     skipped when focus is already inside the palette dialog, whose own
   *     keydown listeners handle the chord).
   *   - Cmd/Ctrl+S       — quick-save (no dialog when the project path is
   *     known).
   *   - Cmd/Ctrl+Z       — undo; with Shift — redo.
   * `preventDefault` keeps the WebView from treating the chords as text
   * shortcuts. Save/undo/redo are backend no-ops without an active project,
   * so the chords are wired globally rather than per-route.
   */
  function handleGlobalKeydown(event: KeyboardEvent) {
    const mod = event.metaKey || event.ctrlKey;
    if (!mod) return;
    const key = event.key.toLowerCase();
    const target = event.target as HTMLElement | null;
    if (key === "k") {
      if (target?.closest('[data-slot="dialog-content"]')) return;
      event.preventDefault();
      commandPaletteOpen.update((open) => !open);
      return;
    }
    if (isEditableTarget(event.target)) return;
    if (key === "s" && !event.shiftKey && !event.altKey) {
      event.preventDefault();
      void quickSave();
    } else if (key === "z" && !event.altKey) {
      event.preventDefault();
      if (event.shiftKey) void doRedo();
      else void doUndo();
    }
  }
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<Tooltip.Provider delayDuration={300}>
  <!--
    Layout root: the children render the active route. `MapView` is mounted
    HERE (single-owner per `consolidate-state-event-flow`) and stays in the
    DOM across route changes; the visibility toggle below shows it only on
    the workspace route. When the workspace shell is active it writes the
    `--canvas-*` CSS variables on `<html>` so the MapView wrapper positions
    itself into the canvas region of the 3-pane grid without remounting.
  -->
  <div class="relative flex h-full w-full overflow-hidden">
    {@render children?.()}
    <div class="map-host" class:hidden={!isWorkspace}>
      <MapView />
    </div>
  </div>

  <Console />
  <CommandPalette />
  <Toaster richColors closeButton position="bottom-right" />
</Tooltip.Provider>

<style>
  /*
   * MapView wrapper. Two layouts:
   *   - cold-start `/` route: no `--canvas-*` vars on the root, the
   *     wrapper sits in flex flow with `flex: 1` (legacy behaviour).
   *   - workspace `/project` route: `WorkspaceShell` writes
   *     `--canvas-left|right|top|bottom` on `<html>` and the wrapper
   *     pins itself into the canvas region. Mount-once is preserved
   *     because the wrapper itself never unmounts; only its position
   *     and visibility change.
   */
  .map-host {
    display: flex;
    min-width: 0;
    flex: 1;
  }

  /* Promote to absolute positioning when the workspace shell is mounted.
   * `WorkspaceShell.svelte` writes `data-workspace-active` on `<html>` so
   * the descendant selector reaches the wrapper without coupling its
   * mount path to the route file. The native flex fallback (above) takes
   * over on the cold-start route where the attribute is absent. */
  :global(html[data-workspace-active="true"]) :where(.map-host) {
    position: absolute;
    inset: var(--canvas-top, 0) var(--canvas-right, 0) var(--canvas-bottom, 0)
      var(--canvas-left, 0);
    flex: none;
  }
</style>
