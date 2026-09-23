<script lang="ts">
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { page } from "$app/state";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { describeFailures, importPaths } from "$lib/actions/import-paths";
  import {
    appendProjectsChunk,
    beginCatalogueRefresh,
    finishCatalogueRefresh,
    applyBundleProgress,
    catalogueError,
    appState,
    commandPaletteOpen,
    currentDownload,
    currentProject,
    downloadPopupHeight,
    finishDownload,
    askBeforeClosing,
    projectDirty,
    projectsLoading,
    requestAllDataFocus,
    resetBundleDownloadState,
    updateDownloadProgress,
  } from "../lib/stores";
  import { loadProjects } from "../lib/api";
  import {
    OFFLINE_CATALOGUE_REASON,
    mayReachNetworkNow,
  } from "../lib/network-reach";
  import { doRedo, doUndo, quickSave } from "$lib/actions/project";
  import CloseGuard from "../components/CloseGuard.svelte";
  import { isEditableTarget } from "$lib/editable-target";
  import { t } from "$lib/i18n";
  import { toast } from "svelte-sonner";
  import { toastOffset } from "$lib/toast-offset";
  import { readyMapName } from "$lib/ready-maps";
  import { installIpcErrorToastObserver } from "../lib/ipc";
  import { applyStoredTheme, installAutoThemeListener } from "../lib/theme";
  import MapView from "../components/MapView.svelte";
  import CommandPalette from "../components/CommandPalette.svelte";
  import Console from "../components/Console.svelte";
  import DownloadPopup from "../components/DownloadPopup.svelte";
  import { Toaster } from "$lib/components/ui/sonner";
  import * as Tooltip from "$lib/components/ui/tooltip";
  import type {
    BundleFileReadyPayload,
    BundleProgressPayload,
    DownloadFinishedPayload,
    DownloadProgressPayload,
    LizaProjectSummaryDto,
  } from "../lib/types";
  import "../app.css";

  let { children } = $props();

  /**
   * Download ids whose first openable map has already been announced.
   *
   * A plain object rather than a `Set`: nothing renders from it, so the
   * reactive collection the linter asks for would only add machinery.
   */
  const announcedReadyMap: Record<string, true> = {};

  const isWorkspace = $derived(page.url.pathname === "/project");
  // Toasts share the bottom-right corner with the download panel; this lifts
  // them clear of it for as long as the panel is on screen.
  const toastEdge = $derived(toastOffset($downloadPopupHeight));

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
          applyBundleProgress(event.payload);
        }),
        listen<LizaProjectSummaryDto[]>("projects-chunk", (event) =>
          appendProjectsChunk(event.payload),
        ),
        // The boundaries of a catalogue walk. Between them the store collects
        // what the walk sends; on a complete one it drops everything else, so
        // a search taken down upstream finally leaves the list.
        listen<void>("catalogue-refresh-started", () =>
          beginCatalogueRefresh(),
        ),
        listen<{ complete: boolean }>("catalogue-refresh-finished", (event) =>
          finishCatalogueRefresh(event.payload.complete),
        ),
        listen<BundleFileReadyPayload>("bundle-file-ready", (event) => {
          // A bundle is fetched whole: the 16 MiB topo layer lands long
          // before the 185 MiB satellite one, and the backend has always
          // made it openable the moment it does. Nothing said so, so the
          // crew waited for the whole bundle. Announced once per download.
          const { download_id, package_name } = event.payload;
          if (announcedReadyMap[download_id]) return;
          const name = readyMapName(
            package_name,
            get(currentProject)?.maps ?? [],
          );
          if (name === null) return;
          announcedReadyMap[download_id] = true;
          toast.success(get(t)("download.mapReady").replace("{name}", name), {
            description: get(t)("download.mapReadyHint"),
          });
        }),
        listen<DownloadFinishedPayload>("download-finished", (event) => {
          const wasShowing = finishDownload(event.payload.download_id);
          // A failure that only cleared the panel used to look like a
          // success: the bundle simply never appeared.
          if (!event.payload.ok && wasShowing) {
            toast.error(get(t)("download.failed"), {
              description: event.payload.message ?? undefined,
            });
          }
        }),
      ]);

      if (cancelled) {
        subscribers.forEach((fn) => fn());
        return;
      }
      unlistens.push(...subscribers);

      // Initial state pull + single-shot startup catalog load. The
      // listener above is already armed, so the `state-changed` emit
      // that load_projects fires on completion is caught. The refresh is
      // strictly background: the loader renders (and keeps clickable) the
      // cache-seeded catalog immediately, with only the small
      // "refreshing list…" hint while this runs.
      await appState.refresh();
      // CJ-2: a cold launch in a field camp reaches for nothing. Without a
      // link the walk only fails slowly, and the cached list the loader has
      // already rendered is the whole truth available — which is exactly
      // what a non-null `catalogueError` makes the interface say ("saved
      // list from {when}"). The refresh button still works; the operator can
      // see the link better than `navigator` can.
      // External review, 2026-09-22.
      if (!mayReachNetworkNow()) {
        projectsLoading.set(false);
        catalogueError.set(OFFLINE_CATALOGUE_REASON);
        return;
      }
      projectsLoading.set(true);
      catalogueError.set(null);
      loadProjects().catch((error) => {
        projectsLoading.set(false);
        catalogueError.set(String(error));
      });
    })();

    // CJ-3: a day's recordings arrive as a handful of files, and dropping
    // them on the window is how anybody expects to hand them over. The
    // dispatch is the picker's, in `$lib/actions/import-paths`, so the two
    // surfaces cannot drift — which is how the picker came to accept `.wpt`
    // while nothing else did.
    (async () => {
      try {
        const unlistenDrop = await getCurrentWebview().onDragDropEvent(
          async (event) => {
            if (event.payload.type !== "drop") return;
            const paths = event.payload.paths ?? [];
            if (paths.length === 0) return;
            const translate = get(t);
            const outcome = await importPaths(paths);
            if (outcome.imported > 0) requestAllDataFocus();
            const summary = translate("tracksTab.importDone")
              .replace("{count}", String(outcome.imported))
              .replace("{total}", String(paths.length));
            if (outcome.failed.length === 0) {
              toast.success(summary);
            } else {
              // Nine files of a day that worked matter more than the one that
              // did not, so this is a success with a caveat.
              toast.warning(summary, {
                description: describeFailures(outcome.failed),
              });
            }
          },
        );
        if (cancelled) unlistenDrop();
        else unlistens.push(unlistenDrop);
      } catch (error) {
        // A webview without drag-and-drop is not a reason to fail the launch;
        // the picker is still there.
        console.error("drag and drop unavailable", error);
      }
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
            // Three answers, so the question is ours rather than the operating
            // system's two-button confirm: the thing an operator almost always
            // wants at this moment is to save and then quit, and it used to
            // not be on offer at all.
            const choice = await askBeforeClosing();
            if (choice === "stay") return;
            if (choice === "save") {
              try {
                await quickSave();
              } catch {
                // `quickSave` has already said what went wrong. The window
                // stays open: quitting after a failed save is the one outcome
                // nobody asked for.
                return;
              }
              // A save-as the operator cancelled leaves the work unsaved, and
              // quitting then would lose it just as surely.
              if (get(projectDirty)) return;
            }
            await getCurrentWindow().destroy();
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
  <!-- Per-file bundle-download progress; fixed bottom-right, survives
       route changes so downloads stay visible while the user works. -->
  <DownloadPopup />
  <!-- Same corner as `DownloadPopup`, and sonner's viewport always draws over
       it, so the toaster steps above the panel while a download runs. -->
  <Toaster richColors closeButton position="bottom-right" offset={toastEdge} />
  <!-- CJ-7: the question asked before a window with unsaved work closes. -->
  <CloseGuard />
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
