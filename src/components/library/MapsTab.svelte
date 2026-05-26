<script lang="ts">
  /**
   * Maps tab inside the LibraryRail. Lists the active project's maps with
   * the active map highlighted and cached vs non-cached maps badged.
   *
   * The tab header carries a single "Maps…" button that opens the bundle
   * loader Sheet through `bundleLoaderOpen` (the Sheet itself is mounted
   * elsewhere, owned by the shell-layout change).
   */
  import MapIcon from "@lucide/svelte/icons/map";
  import ExternalLinkIcon from "@lucide/svelte/icons/external-link";
  import { Button } from "$lib/components/ui/button";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import {
    activeMap,
    bundleLoaderOpen,
    busy,
    currentProject,
    downloadingMaps,
    downloadProgress,
  } from "$lib/stores";
  import { get } from "svelte/store";
  import { openSelectedMap, revealBundle } from "$lib/api";
  import { toast } from "svelte-sonner";
  import LibraryRow from "./LibraryRow.svelte";

  const maps = $derived($currentProject?.maps ?? []);

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

  function openLoader() {
    // Bug 4 fix (fix-redesign-functional-bugs): when a `load_projects` IPC
    // is already in flight (`$busy === true`), opening the loader would
    // queue a second `load_projects` behind the first and freeze the UI
    // for ~5s while the backend serialised the two calls. The Sheet itself
    // still opens, but we short-circuit if busy as a defence-in-depth
    // alongside `disabled={$busy}` on the trigger.
    if (get(busy)) return;
    bundleLoaderOpen.set(true);
  }
</script>

<div class="flex h-full min-h-0 flex-col">
  <header class="border-border flex items-center justify-between border-b px-2 py-1.5">
    <Button
      variant="outline"
      size="xs"
      onclick={openLoader}
      disabled={$busy}
    >
      <MapIcon />
      Maps…
    </Button>
  </header>

  <div class="flex-1 overflow-y-auto py-1">
    {#if maps.length === 0}
      <div class="text-muted-foreground p-3 text-center text-xs">
        No maps in this project
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
                cached
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
