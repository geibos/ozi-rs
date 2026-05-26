<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import { listen } from "@tauri-apps/api/event";
  import {
    appendProjectsChunk,
    appState,
    bundleProgress,
    currentDownload,
    resetBundleDownloadState,
    updateDownloadProgress,
  } from "../lib/stores";
  import { loadProjects } from "../lib/api";
  import { applyStoredTheme, installAutoThemeListener } from "../lib/theme";
  import MapView from "../components/MapView.svelte";
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

    return () => {
      cancelled = true;
      unlistens.forEach((fn) => fn());
      unlistenAutoTheme();
      // Reset the bundle-loader transient stores so a relaunch (or a
      // hot-reload during dev) starts from a clean slate. activeDownloadId
      // is reset together with the progress/payload stores.
      resetBundleDownloadState(null);
    };
  });
</script>

<Tooltip.Provider delayDuration={300}>
  <div class="flex h-full w-full overflow-hidden">
    {@render children?.()}
    <div class="flex min-w-0 flex-1" class:hidden={!isWorkspace}>
      <MapView />
    </div>
  </div>

  <Console />
  <Toaster richColors closeButton position="bottom-right" />
</Tooltip.Provider>
