<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { appState, updateDownloadProgress } from "./lib/stores";
  import { loadProjects } from "./lib/api";
  import MapView from "./components/MapView.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import TracksPanel from "./components/TracksPanel.svelte";
  import TrackPointsPanel from "./components/TrackPointsPanel.svelte";
  import WaypointsPanel from "./components/WaypointsPanel.svelte";
  import Console from "./components/Console.svelte";
  import type { DownloadProgressPayload } from "./lib/types";

  onMount(async () => {
    // Initial state load + auto-scan bundles root
    await appState.refresh();
    loadProjects().catch(() => {}); // non-blocking; fails silently if root not set

    // Listen for backend state-change events
    const unlisten = await listen<void>("state-changed", async () => {
      await appState.refresh();
    });

    const unlistenProgress = await listen<DownloadProgressPayload>(
      "download-progress",
      (event) => updateDownloadProgress(event.payload)
    );

    return () => {
      unlisten();
      unlistenProgress();
    };
  });
</script>

<div class="layout">
  <Sidebar />
  <MapView />
</div>

<TracksPanel />
<TrackPointsPanel />
<WaypointsPanel />
<Console />

<style>
  .layout {
    display: flex;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }
</style>
