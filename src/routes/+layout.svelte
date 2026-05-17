<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import { listen } from "@tauri-apps/api/event";
  import { appState, updateDownloadProgress } from "../lib/stores";
  import { loadProjects } from "../lib/api";
  import { applyStoredTheme, installAutoThemeListener } from "../lib/theme";
  import MapView from "../components/MapView.svelte";
  import Console from "../components/Console.svelte";
  import type { DownloadProgressPayload } from "../lib/types";
  import "../app.css";

  let { children } = $props();

  const isWorkspace = $derived(page.url.pathname === "/project");

  applyStoredTheme();

  onMount(() => {
    let cancelled = false;
    let unlistenState: (() => void) | null = null;
    let unlistenProgress: (() => void) | null = null;
    const unlistenAutoTheme = installAutoThemeListener();

    (async () => {
      await appState.refresh();
      loadProjects().catch(() => {});

      if (cancelled) return;

      unlistenState = await listen<void>("state-changed", async () => {
        await appState.refresh();
      });

      unlistenProgress = await listen<DownloadProgressPayload>(
        "download-progress",
        (event) => updateDownloadProgress(event.payload),
      );
    })();

    return () => {
      cancelled = true;
      unlistenState?.();
      unlistenProgress?.();
      unlistenAutoTheme();
    };
  });
</script>

<div class="app-shell">
  {@render children?.()}
  <div class="map-host" class:hidden={!isWorkspace}>
    <MapView />
  </div>
</div>

<Console />

<style>
  .app-shell {
    display: flex;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .map-host {
    flex: 1;
    min-width: 0;
    display: flex;
  }

  .map-host.hidden {
    display: none;
  }
</style>
