<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import { listen } from "@tauri-apps/api/event";
  import { appState, updateDownloadProgress } from "../lib/stores";
  import { loadProjects } from "../lib/api";
  import { applyStoredTheme, installAutoThemeListener } from "../lib/theme";
  import MapView from "../components/MapView.svelte";
  import Console from "../components/Console.svelte";
  import { Toaster } from "$lib/components/ui/sonner";
  import * as Tooltip from "$lib/components/ui/tooltip";
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
