<script lang="ts">
  /**
   * Maps tab inside the LibraryRail. Lists the active project's maps with
   * the active map highlighted and cached vs non-cached maps badged.
   *
   * Owner feedback (first hands-on session): "No maps in this project"
   * with no visible way forward was a dead end. The tab now carries an
   * explicit "Open project…" affordance — a small header button that is
   * always visible, plus a prominent button inside the empty state. Both
   * reuse the working bundle-loader mechanism from the Cmd-K palette's
   * "Switch project" group: set `bundleLoaderOpen` and navigate to the
   * `/` cold-start route where the loader Sheet lives.
   */
  import ExternalLinkIcon from "@lucide/svelte/icons/external-link";
  import FolderOpenIcon from "@lucide/svelte/icons/folder-open";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { Button } from "$lib/components/ui/button";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import {
    activeMap,
    bundleLoaderOpen,
    currentProject,
    downloadingMaps,
    downloadProgress,
  } from "$lib/stores";
  import { openSelectedMap, revealBundle } from "$lib/api";
  import { mapsForLibrary } from "$lib/maps-list";
  import { t } from "$lib/i18n";
  import { toast } from "svelte-sonner";
  import LibraryRow from "./LibraryRow.svelte";

  // A locally opened OZI map belongs to no LizaAlert project, so listing
  // `currentProject.maps` alone showed "No maps in this project" while that
  // map was rendering on the canvas. See `mapsForLibrary`.
  const maps = $derived(mapsForLibrary($currentProject?.maps ?? [], $activeMap));

  /**
   * Open the bundle loader — the same mechanism the command palette's
   * "Switch project" action uses (`bundleLoaderOpen` + goto "/"). The `/`
   * route hosts the loader Sheet; `bundleLoaderOpen` keeps it expanded.
   */
  function handleOpenProject() {
    bundleLoaderOpen.set(true);
    void goto(resolve("/"));
  }

  function isActive(mapName: string): boolean {
    return $activeMap?.package_name === mapName;
  }

  async function handleSwitchTo(mapName: string) {
    try {
      await openSelectedMap(mapName);
    } catch (err) {
      toast.error("Failed to switch map", { description: String(err) });
    }
  }

  async function handleReveal() {
    try {
      await revealBundle();
    } catch (err) {
      toast.error("Failed to reveal bundle", { description: String(err) });
    }
  }
</script>

<div class="flex h-full min-h-0 flex-col">
  <header
    class="border-border flex items-center justify-end border-b px-2 py-1.5"
  >
    <Button
      variant="outline"
      size="xs"
      class="gap-1.5"
      onclick={handleOpenProject}
      data-testid="maps-open-project"
    >
      <FolderOpenIcon class="size-3.5" />
      {$t("mapsTab.openProject")}
    </Button>
  </header>
  <div class="flex-1 overflow-y-auto py-1">
    {#if maps.length === 0}
      <div
        class="flex flex-col items-center gap-3 p-6 text-center"
        data-testid="maps-empty-state"
      >
        <p class="text-muted-foreground text-xs">
          {$t("mapsTab.empty")}
        </p>
        <p class="text-muted-foreground/70 text-[11px] leading-snug">
          {$t("mapsTab.emptyHint")}
        </p>
        <Button
          size="sm"
          class="gap-2"
          onclick={handleOpenProject}
          data-testid="maps-empty-open-project"
        >
          <FolderOpenIcon class="size-4" />
          {$t("mapsTab.openProject")}
        </Button>
      </div>
    {:else}
      {#each maps as m (m.name)}
        {@const active = isActive(m.name)}
        {@const isDownloading = $downloadingMaps.has(m.name)}
        {@const prog = $downloadProgress.get(m.name)}
        {@const pct =
          prog?.total_bytes && prog.total_bytes > 0
            ? Math.round((prog.downloaded_bytes / prog.total_bytes) * 100)
            : null}
        <LibraryRow
          name={m.name}
          visible={true}
          selected={active}
          onSelect={() => {
            if (!active && !isDownloading) void handleSwitchTo(m.name);
          }}
        >
          {#snippet leadingControl()}
            {#if isDownloading}
              <!--
                Bug 3 fix (fix-redesign-functional-bugs): the Library Maps
                tab now mirrors BundleLoader.svelte's per-row progress
                badge, sourced from the same `downloadingMaps` /
                `downloadProgress` stores. No new `listen()` registration
                is added here — the layout owns the events.
              -->
              <span
                class="bg-primary text-background inline-flex h-4 min-w-[2rem] items-center justify-center rounded-sm px-1 text-[10px] font-medium tabular-nums"
                title="Downloading"
              >
                {pct != null ? `${pct}%` : "…"}
              </span>
            {:else if m.downloaded}
              <span
                class="bg-muted text-muted-foreground inline-flex h-4 items-center rounded-sm px-1.5 text-[10px] font-medium uppercase tracking-wide"
                title="Tiles cached locally"
              >
                {$t("mapsTab.cached")}
              </span>
            {:else}
              <span class="size-4"></span>
            {/if}
          {/snippet}
          {#snippet actions()}
            <DropdownMenu.Item onSelect={() => handleSwitchTo(m.name)}>
              Switch to
            </DropdownMenu.Item>
            <DropdownMenu.Item onSelect={handleReveal} disabled={!active}>
              <ExternalLinkIcon class="size-3.5" />
              Reveal in Finder
            </DropdownMenu.Item>
          {/snippet}
        </LibraryRow>
      {/each}
    {/if}
  </div>
</div>
