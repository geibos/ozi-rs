<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { appState } from "./lib/stores";
  import MapView from "./components/MapView.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import TracksPanel from "./components/TracksPanel.svelte";
  import Console from "./components/Console.svelte";
  import type { DownloadProgressPayload } from "./lib/types";

  onMount(async () => {
    // Initial state load
    await appState.refresh();

    // Listen for backend state-change events
    const unlisten = await listen<void>("state-changed", async () => {
      await appState.refresh();
    });

    // Download progress events (for progress bar, future feature)
    const unlistenProgress = await listen<DownloadProgressPayload>(
      "download-progress",
      (_event) => {
        // TODO: show download progress bar
      }
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
<Console />

<style>
  .layout {
    display: flex;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }
</style>
