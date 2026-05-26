<script lang="ts">
  /**
   * Global Cmd-K command palette.
   *
   * Mounted once at the layout level (`src/routes/+layout.svelte`). Open
   * state is backed by `commandPaletteOpen` in `src/lib/stores.ts`. The
   * global `⌘K` / `Ctrl+K` handler lives in `+layout.svelte`.
   *
   * Groups (fixed order — Decision 4):
   *   1. Open map        — maps in the active project
   *   2. Switch project  — LizaAlert catalog
   *   3. Find track / waypoint (merged)
   *   4. Project actions — Open / Save / Undo / Redo
   *   5. Settings        — theme / units (placeholder) / GPS (placeholder)
   *   6. Recent files    — `ozi:recent-files:v1`
   *
   * Context adaptation (Decision 8 caveat): at cold-start (no active
   * project) groups 1, 3, 4 are hidden.
   *
   * Empty groups (after filtering) are hidden entirely. If every group is
   * empty we render a single "No matches" empty state.
   *
   * Primary action runs on `Enter` (palette closes). Secondary actions:
   *   - `⌘E` / `Ctrl+E` — Export for track/waypoint results
   *   - `⌘R` / `Ctrl+R` — Reveal in Finder for map results
   */
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { get } from "svelte/store";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Command from "$lib/components/ui/command";
  import {
    activeMap,
    activeTrackLayerId,
    activeWaypointLayerId,
    appState,
    bundleLoaderOpen,
    commandPaletteOpen,
    currentProject,
    projects,
    selectedMapInfo,
    selectedTrack,
    selectedWaypointId,
  } from "$lib/stores";
  import {
    exportGpx,
    exportWptWaypoints,
    getTrackExportDefaultPath,
    getWaypoints,
    getWptExportDefaultPath,
    loadProjectFile,
    openSelectedMap,
    redo,
    revealBundle,
    saveProject,
    undo,
  } from "$lib/api";
  import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { appendRecentFile, getRecentFiles } from "$lib/recentFiles";
  import { toast } from "svelte-sonner";
  import type { WaypointData } from "$lib/types";

  // ── Internal state ──────────────────────────────────────────────────────
  let value = $state("");
  let inputRef = $state<HTMLInputElement | null>(null);
  let waypointsCache: WaypointData[] = $state([]);

  // Re-fetch waypoints when palette opens with an active waypoint layer so
  // "Find waypoint" results match current data. Cheap (single IPC) and
  // bounded by `MAX_RECENT_FILES`-style result limits below.
  $effect(() => {
    if ($commandPaletteOpen) {
      void loadWaypointsForSearch();
    } else {
      value = "";
    }
  });

  async function loadWaypointsForSearch() {
    const layerId = get(activeWaypointLayerId);
    if (layerId === null) {
      waypointsCache = [];
      return;
    }
    try {
      waypointsCache = await getWaypoints(layerId);
    } catch {
      waypointsCache = [];
    }
  }

  function close() {
    commandPaletteOpen.set(false);
  }

  // ── Group data sources ──────────────────────────────────────────────────
  const isColdStart = $derived($currentProject === null || $activeMap === null);

  const maps = $derived(
    $currentProject?.maps.filter((m) => m.downloaded).map((m) => m.name) ?? [],
  );

  const tracks = $derived(
    $appState?.tracks.map((t) => ({
      layerId: BigInt(t.layer_id),
      trackId: BigInt(t.track_id),
      name: t.name,
    })) ?? [],
  );

  const projectList = $derived($projects);
  const recentFiles = $derived.by(() =>
    $commandPaletteOpen ? getRecentFiles() : [],
  );

  // ── Primary actions ─────────────────────────────────────────────────────
  async function handleOpenMap(mapName: string) {
    close();
    try {
      await openSelectedMap(mapName);
      const am = get(activeMap);
      const cp = get(currentProject);
      if (am && cp) {
        // Spec: append to recent-files on EVERY successful map open. The
        // bundle-loader flow does the same — both call sites share this
        // helper.
        appendRecentFile({
          projectSlug: am.project_name,
          mapPath: am.local_path,
          mapName: am.package_name,
          openedAt: Date.now(),
        });
        await goto(resolve("/project"));
      }
    } catch (error) {
      toast.error("Failed to open map", { description: String(error) });
    }
  }

  function handleSwitchProject(slug: string) {
    close();
    bundleLoaderOpen.set(true);
    void goto(resolve("/"));
    // The bundle-loader Sheet (or current loader UI) reads `bundleLoaderOpen`
    // and renders. Pre-scroll into the chosen project is a UX nice-to-have
    // we defer — clicking the row in the sheet remains required.
    toast.message(`Switch to ${slug}`, {
      description: "Pick the bundle in the loader to continue.",
    });
  }

  function handleFindTrack(layerId: bigint, trackId: bigint) {
    selectedTrack.set({ layerId, trackId });
    activeTrackLayerId.set(layerId);
    close();
  }

  function handleFindWaypoint(id: bigint) {
    selectedWaypointId.set(id);
    close();
  }

  async function handleProjectOpen() {
    close();
    try {
      const path = await openDialog({
        filters: [{ name: "OziRS project", extensions: ["json"] }],
      } as Parameters<typeof openDialog>[0]);
      if (path) await loadProjectFile(path as string);
    } catch (error) {
      toast.error("Failed to open project", { description: String(error) });
    }
  }

  async function handleProjectSave() {
    close();
    try {
      const path = await saveDialog({
        filters: [{ name: "OziRS project", extensions: ["json"] }],
      });
      if (path) await saveProject(path);
    } catch (error) {
      toast.error("Failed to save project", { description: String(error) });
    }
  }

  async function handleUndo() {
    close();
    try {
      await undo();
    } catch (error) {
      toast.error("Undo failed", { description: String(error) });
    }
  }

  async function handleRedo() {
    close();
    try {
      await redo();
    } catch (error) {
      toast.error("Redo failed", { description: String(error) });
    }
  }

  function handleThemeSetting() {
    close();
    toast.message("Theme picker", {
      description: "Use the existing Theme Picker in the sidebar.",
    });
  }

  function handleUnitsSetting() {
    close();
    toast.message("Units — coming soon");
  }

  function handleGpsSetting() {
    close();
    toast.message("GPS — coming soon");
  }

  // ── Secondary actions ───────────────────────────────────────────────────
  async function exportTrack(layerId: bigint, _trackId: bigint, name: string) {
    try {
      const defaultPath = await getTrackExportDefaultPath(name, "gpx");
      const path = await saveDialog({
        defaultPath: defaultPath ?? `${name}.gpx`,
        filters: [{ name: "GPX", extensions: ["gpx"] }],
      });
      if (path) await exportGpx(layerId, path);
    } catch (error) {
      toast.error("Failed to export track", { description: String(error) });
    }
  }

  async function exportWaypoint(layerId: bigint) {
    try {
      const defaultPath = await getWptExportDefaultPath(layerId);
      const path = await saveDialog({
        defaultPath: defaultPath ?? "waypoints.wpt",
        filters: [{ name: "OziExplorer WPT", extensions: ["wpt"] }],
      });
      if (path) await exportWptWaypoints(layerId, path);
    } catch (error) {
      toast.error("Failed to export waypoints", {
        description: String(error),
      });
    }
  }

  async function handleRevealMap() {
    try {
      await revealBundle();
    } catch (error) {
      toast.error("Failed to reveal map", { description: String(error) });
    }
  }

  /**
   * Resolve "what is the currently highlighted result, and what kind?"
   * The bits-ui CommandPrimitive does not expose the highlighted item via
   * a stable API, so we listen for `⌘E` / `⌘R` and let the Command
   * primitive's own `value` (the `value` prop on the highlighted Item)
   * tell us. Each result uses an opaque token like `track:LAYER:TRACK:name`
   * as its primitive value.
   */
  function parseHighlighted(token: string):
    | { kind: "track"; layerId: bigint; trackId: bigint; name: string }
    | { kind: "waypoint"; layerId: bigint; id: bigint; name: string }
    | { kind: "map"; name: string }
    | null {
    if (token.startsWith("track:")) {
      const [, layer, id, ...name] = token.split(":");
      return {
        kind: "track",
        layerId: BigInt(layer),
        trackId: BigInt(id),
        name: name.join(":"),
      };
    }
    if (token.startsWith("waypoint:")) {
      const [, layer, id, ...name] = token.split(":");
      return {
        kind: "waypoint",
        layerId: BigInt(layer),
        id: BigInt(id),
        name: name.join(":"),
      };
    }
    if (token.startsWith("map:")) {
      return { kind: "map", name: token.slice(4) };
    }
    return null;
  }

  function handleKeydown(event: KeyboardEvent) {
    if (!$commandPaletteOpen) return;
    const mod = event.metaKey || event.ctrlKey;
    if (!mod) return;
    const key = event.key.toLowerCase();
    if (key !== "e" && key !== "r") return;
    const highlighted = parseHighlighted(value);
    if (!highlighted) return;
    if (key === "e" && highlighted.kind === "track") {
      event.preventDefault();
      close();
      void exportTrack(
        highlighted.layerId,
        highlighted.trackId,
        highlighted.name,
      );
    } else if (key === "e" && highlighted.kind === "waypoint") {
      event.preventDefault();
      close();
      void exportWaypoint(highlighted.layerId);
    } else if (key === "r" && highlighted.kind === "map") {
      event.preventDefault();
      close();
      void handleRevealMap();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<Dialog.Root
  open={$commandPaletteOpen}
  onOpenChange={(v) => commandPaletteOpen.set(v)}
>
  <Dialog.Content
    class="rounded-xl! top-1/3 max-w-xl translate-y-0 overflow-hidden p-0"
    showCloseButton={false}
  >
    <Dialog.Header class="sr-only">
      <Dialog.Title>Command palette</Dialog.Title>
      <Dialog.Description>Search for an action or item.</Dialog.Description>
    </Dialog.Header>
    <Command.Root bind:value class="rounded-lg border-none shadow-md">
      <Command.Input
        bind:ref={inputRef}
        placeholder="Search or jump to…"
      />
      <Command.List class="max-h-[60vh]">
        <Command.Empty>No matches.</Command.Empty>

        {#if !isColdStart && maps.length > 0}
          <Command.Group heading="Open map">
            {#each maps as mapName (mapName)}
              <Command.Item
                value={`map:${mapName}`}
                onSelect={() => handleOpenMap(mapName)}
              >
                <span class="flex-1 truncate">{mapName}</span>
                <Command.Shortcut>⌘R</Command.Shortcut>
              </Command.Item>
            {/each}
          </Command.Group>
        {/if}

        {#if projectList.length > 0}
          <Command.Group heading="Switch project">
            {#each projectList as project (project.slug)}
              <Command.Item
                value={`project:${project.slug}:${project.name}`}
                onSelect={() => handleSwitchProject(project.slug)}
              >
                <span class="flex-1 truncate">{project.name}</span>
                <span class="text-muted-foreground font-mono text-[10px]"
                  >{project.slug}</span
                >
              </Command.Item>
            {/each}
          </Command.Group>
        {/if}

        {#if !isColdStart && (tracks.length > 0 || waypointsCache.length > 0)}
          <Command.Group heading="Find track / waypoint">
            {#each tracks as t (`${t.layerId}-${t.trackId}`)}
              <Command.Item
                value={`track:${t.layerId}:${t.trackId}:${t.name}`}
                onSelect={() => handleFindTrack(t.layerId, t.trackId)}
              >
                <span class="text-muted-foreground">Track</span>
                <span class="flex-1 truncate">{t.name}</span>
                <Command.Shortcut>⌘E</Command.Shortcut>
              </Command.Item>
            {/each}
            {#each waypointsCache as w (w.id)}
              <Command.Item
                value={`waypoint:${$activeWaypointLayerId ?? 0}:${w.id}:${w.name}`}
                onSelect={() => handleFindWaypoint(BigInt(w.id))}
              >
                <span class="text-muted-foreground">Waypoint</span>
                <span class="flex-1 truncate">{w.name}</span>
                <Command.Shortcut>⌘E</Command.Shortcut>
              </Command.Item>
            {/each}
          </Command.Group>
        {/if}

        {#if !isColdStart}
          <Command.Group heading="Project actions">
            <Command.Item value="action:open" onSelect={handleProjectOpen}>
              <span class="flex-1">Open project…</span>
            </Command.Item>
            <Command.Item value="action:save" onSelect={handleProjectSave}>
              <span class="flex-1">Save project…</span>
            </Command.Item>
            <Command.Item value="action:undo" onSelect={handleUndo}>
              <span class="flex-1">Undo</span>
            </Command.Item>
            <Command.Item value="action:redo" onSelect={handleRedo}>
              <span class="flex-1">Redo</span>
            </Command.Item>
          </Command.Group>
        {/if}

        <Command.Group heading="Settings">
          <Command.Item value="setting:theme" onSelect={handleThemeSetting}>
            <span class="flex-1">Theme</span>
          </Command.Item>
          <Command.Item value="setting:units" onSelect={handleUnitsSetting}>
            <span class="flex-1">Units</span>
            <span class="text-muted-foreground text-[10px]">Coming soon</span>
          </Command.Item>
          <Command.Item value="setting:gps" onSelect={handleGpsSetting}>
            <span class="flex-1">GPS</span>
            <span class="text-muted-foreground text-[10px]">Coming soon</span>
          </Command.Item>
        </Command.Group>

        {#if recentFiles.length > 0}
          <Command.Group heading="Recent files">
            {#each recentFiles as r (r.mapPath)}
              <Command.Item
                value={`recent:${r.mapPath}`}
                onSelect={() => {
                  // The full open flow runs through `openSelectedMap`, which
                  // requires the current project to match. We invoke the
                  // same code path the bundle loader does — load the
                  // project, then open the map. Cheap version: just call
                  // `openSelectedMap(mapName)` when the project is already
                  // current; otherwise hint to the user.
                  if ($currentProject?.name === r.projectSlug) {
                    void handleOpenMap(r.mapName);
                  } else {
                    close();
                    selectedMapInfo.set({
                      projectSlug: r.projectSlug,
                      packageName: r.mapName,
                      localPath: r.mapPath,
                    });
                    toast.message(`Recent — ${r.mapName}`, {
                      description: `Open project "${r.projectSlug}" from the bundle loader, then this map will be available.`,
                    });
                  }
                }}
              >
                <span class="flex-1 truncate">{r.mapName}</span>
                <span
                  class="text-muted-foreground truncate font-mono text-[10px]"
                  >{r.projectSlug}</span
                >
                <Command.Shortcut>⌘R</Command.Shortcut>
              </Command.Item>
            {/each}
          </Command.Group>
        {/if}
      </Command.List>
    </Command.Root>
  </Dialog.Content>
</Dialog.Root>
