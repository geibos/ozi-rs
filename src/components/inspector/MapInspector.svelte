<script lang="ts">
  /**
   * Map Inspector — read-only calibration metadata for the active map.
   *
   * Rendered by `InspectorRail` when the Library Maps tab's "Map info"
   * affordance is active (`$selectedMapInfo` non-null). No write affordances
   * for calibration; future `map-calibration-editor` change owns that.
   *
   * Data source: existing `getOziMetadata(mapPath)` in `src/lib/api.ts`.
   * "Reveal in Finder" uses the existing `revealBundle` IPC (no new
   * Tauri command).
   */
  import FolderOpenIcon from "@lucide/svelte/icons/folder-open";
  import MapIcon from "@lucide/svelte/icons/map";
  import { Button } from "$lib/components/ui/button";
  import { getOziMetadata, revealBundle } from "$lib/api";
  import { activeMap, selectedMapInfo } from "$lib/stores";
  import { toast } from "svelte-sonner";
  import type { OziMetadataDto } from "$lib/types";

  let metadata: OziMetadataDto | null = $state(null);
  let loading = $state(false);
  let lastLoadedPath: string | null = $state(null);

  $effect(() => {
    const info = $selectedMapInfo;
    if (!info) {
      metadata = null;
      lastLoadedPath = null;
      return;
    }
    if (info.localPath === lastLoadedPath) return;
    void loadMetadata(info.localPath);
  });

  async function loadMetadata(path: string) {
    loading = true;
    try {
      metadata = await getOziMetadata(path);
      lastLoadedPath = path;
    } catch (error) {
      console.error("MapInspector: getOziMetadata failed", error);
      // Not all maps are OZF2 — SQLite bundles return an error here. We show
      // the basic info from `selectedMapInfo` regardless.
      metadata = null;
      lastLoadedPath = path;
    } finally {
      loading = false;
    }
  }

  async function handleReveal() {
    try {
      await revealBundle();
    } catch (error) {
      toast.error("Failed to reveal in file manager", {
        description: String(error),
      });
    }
  }

  function formatBounds(
    bounds: [number, number, number, number] | null | undefined,
  ): string {
    if (!bounds) return "—";
    const [minLon, minLat, maxLon, maxLat] = bounds;
    return `${minLat.toFixed(4)}, ${minLon.toFixed(4)} → ${maxLat.toFixed(4)}, ${maxLon.toFixed(4)}`;
  }

  function formatResolution(
    levels: OziMetadataDto["levels"] | undefined,
  ): string {
    if (!levels || levels.length === 0) return "—";
    const base = levels[0];
    return `${base.width} × ${base.height}px`;
  }
</script>

<div class="flex h-full flex-col gap-4 overflow-y-auto p-4">
  <header class="flex items-start gap-3">
    <div
      class="bg-muted text-muted-foreground flex size-9 shrink-0 items-center justify-center rounded-xl"
    >
      <MapIcon class="size-5" />
    </div>
    <div class="min-w-0 flex-1">
      <h2
        class="text-foreground truncate text-sm font-semibold leading-tight"
        title={$selectedMapInfo?.packageName}
      >
        {$selectedMapInfo?.packageName ?? "—"}
      </h2>
      <p
        class="text-muted-foreground mt-0.5 truncate text-xs"
        title={$selectedMapInfo?.projectSlug ?? ""}
      >
        {$selectedMapInfo?.projectSlug ?? $activeMap?.project_name ?? "—"}
      </p>
    </div>
  </header>

  <section
    class="bg-card border-border rounded-[var(--radius-card)] border p-4"
    aria-label="Calibration metadata"
  >
    <h3
      class="text-muted-foreground/80 mb-3 text-[10px] font-semibold uppercase tracking-wider"
    >
      Calibration
    </h3>
    {#if loading}
      <p class="text-muted-foreground text-xs">Loading metadata…</p>
    {:else if metadata}
      <dl class="grid grid-cols-[5rem_1fr] gap-x-3 gap-y-2 text-xs">
        <dt class="text-muted-foreground">CRS</dt>
        <dd class="font-mono">{metadata.projection || "—"}</dd>
        <dt class="text-muted-foreground">Datum</dt>
        <dd class="font-mono">{metadata.datum || "—"}</dd>
        <dt class="text-muted-foreground">Bounds</dt>
        <dd class="font-mono text-[11px]">{formatBounds(metadata.bounds)}</dd>
        <dt class="text-muted-foreground">Resolution</dt>
        <dd class="font-mono">{formatResolution(metadata.levels)}</dd>
        <dt class="text-muted-foreground">Native zoom</dt>
        <dd class="font-mono">{metadata.native_zoom}</dd>
      </dl>
    {:else}
      <p class="text-muted-foreground text-xs">
        No OZF2 calibration metadata available (this map may be a SQLite
        bundle).
      </p>
    {/if}
  </section>

  <section class="flex flex-col gap-2" aria-label="Map actions">
    <Button
      variant="outline"
      size="sm"
      class="justify-start gap-2"
      onclick={handleReveal}
    >
      <FolderOpenIcon class="size-4" />
      Reveal in Finder
    </Button>
  </section>
</div>
