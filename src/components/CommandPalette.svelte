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
    bundleLoaderPreselect,
    commandPaletteOpen,
    measuringActive,
    setMeasuring,
    ringActive,
    setRing,
    projectionActive,
    setProjection,
    currentProject,
    projects,
    selectedMapInfo,
    selectedTrack,
    selectedWaypointId,
    selectedTheme,
    measureMode,
  } from "$lib/stores";
  import {
    exportGpx,
    exportWptWaypoints,
    getTrackExportDefaultPath,
    getWaypoints,
    getWptExportDefaultPath,
    revealBundle,
  } from "$lib/api";
  import {
    doRedo,
    doUndo,
    openProjectFile,
    quickSave,
    startNewProject,
  } from "$lib/actions/project";
  import { openMapShowingDownload } from "$lib/actions/open-map";
  // Aliased: the tracks loop below binds `t`, and a store read inside it
  // would resolve to the loop variable.
  import { t as i18n, toggleLocale } from "$lib/i18n";
  import { applyTheme, type ThemeName } from "$lib/theme";
  import { save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { appendRecentFile, getRecentFiles } from "$lib/recentFiles";
  import { getRecentProjects } from "$lib/recent-projects";
  import { toast } from "svelte-sonner";
  import { paletteProjects } from "$lib/palette-projects";
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
  /**
   * "Cold start" means there is no workspace on screen — no map.
   *
   * It used to also require a LizaAlert project, so opening a local OZI map
   * left the palette offering nothing but Settings: save, undo, redo and the
   * track search were all hidden while a project with dozens of tracks was
   * open. The bundle-specific group guards on the project itself.
   */
  const isColdStart = $derived($activeMap === null);

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

  /**
   * The catalogue is ~13k projects since the listing started paginating, and
   * `cmdk` renders and re-filters every item it is given on each keystroke.
   * Filter here and hand it a screenful: without a query the most recent
   * entries, with one the matches by name or slug.
   */
  const PALETTE_PROJECT_LIMIT = 40;
  const projectList = $derived(
    paletteProjects($projects, value, PALETTE_PROJECT_LIMIT),
  );
  // Re-read when the palette opens: another surface may have saved since.
  const recentProjects = $derived.by(() => {
    void $commandPaletteOpen;
    return getRecentProjects();
  });

  const recentFiles = $derived.by(() =>
    $commandPaletteOpen ? getRecentFiles() : [],
  );

  // ── Primary actions ─────────────────────────────────────────────────────
  async function handleOpenMap(mapName: string) {
    close();
    try {
      // A map that is not on disk starts a download instead of opening: the
      // helper puts its progress on screen with a cancel, and there is nothing
      // to record or navigate to until it lands.
      if (!(await openMapShowingDownload(mapName))) return;
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
      toast.error($i18n("palette.openMapFailed"), {
        description: String(error),
      });
    }
  }

  function handleSwitchProject(slug: string) {
    close();
    // The loader reads this and selects the project itself — finding it by
    // hand in a thirteen-thousand-row list is not a "switch".
    bundleLoaderPreselect.set(slug);
    bundleLoaderOpen.set(true);
    void goto(resolve("/"));
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

  function handleProjectNew() {
    close();
    // Asking before discarding lives in the action, with the same question
    // the close guard asks.
    void startNewProject();
  }

  function handleProjectOpen() {
    close();
    // Dialog, remembering and framing live in `$lib/actions/project`: the
    // launch screen and the recents below ask for the same thing.
    void openProjectFile();
  }

  /**
   * Reopen a project from the recents.
   *
   * A path can go stale — the file moved, the disk is not mounted, the crew
   * is on the other machine. Rather than leaving an entry that fails every
   * time it is chosen, a failure drops it and says so.
   */
  function handleOpenRecentProject(path: string) {
    close();
    // A stale path is dropped by the action — the file moved, the disk is not
    // mounted, the crew is on the other machine.
    void openProjectFile(path);
  }

  // Save / undo / redo delegate to the shared CJ-7 actions in
  // `$lib/actions/project` — the same code path the global keyboard chords
  // and the workspace-shell buttons use (toasts and dialog fallback live
  // there, not here).
  function handleProjectSave() {
    close();
    void quickSave();
  }

  function handleUndo() {
    close();
    void doUndo();
  }

  function handleRedo() {
    close();
    void doRedo();
  }

  function handleLanguageToggle() {
    toggleLocale();
    close();
  }

  /**
   * The colour themes, as palette entries.
   *
   * `ui-shell` has required a theme selector all along and `ThemePicker.svelte`
   * has existed all along, mounted nowhere: the workspace redesign took away
   * the sidebar it lived in and left this entry raising a toast that said, in
   * English, to use a picker in a sidebar that is not there. A setting the
   * operator can reach is the requirement; the palette is where the other
   * settings already are, and it is searchable, which a select in a corner is
   * not.
   */
  const THEMES: { value: ThemeName; label: string }[] = [
    { value: "native-auto", label: "Native — Auto" },
    { value: "native-light", label: "Native — Light" },
    { value: "native-dark", label: "Native — Dark" },
    { value: "latte", label: "Catppuccin — Latte" },
    { value: "frappe", label: "Catppuccin — Frappé" },
    { value: "macchiato", label: "Catppuccin — Macchiato" },
    { value: "mocha", label: "Catppuccin — Mocha" },
  ];

  function handleThemeChoice(theme: ThemeName) {
    close();
    // The store persists the choice; `applyTheme` puts it on the screen now
    // rather than at the next launch.
    selectedTheme.set(theme);
    void applyTheme(theme);
  }

  function handleUnitsSetting() {
    close();
    toast.message(`${$i18n("palette.units")} — ${$i18n("palette.comingSoon")}`);
  }

  function handleGpsSetting() {
    close();
    toast.message(`${$i18n("palette.gps")} — ${$i18n("palette.comingSoon")}`);
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
      toast.error($i18n("palette.exportTrackFailed"), {
        description: String(error),
      });
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
      toast.error($i18n("palette.exportWaypointsFailed"), {
        description: String(error),
      });
    }
  }

  async function handleRevealMap() {
    try {
      await revealBundle();
    } catch (error) {
      toast.error($i18n("palette.revealMapFailed"), {
        description: String(error),
      });
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
  function parseHighlighted(
    token: string,
  ):
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
    class="top-1/3 max-w-xl translate-y-0 overflow-hidden rounded-xl! p-0"
    showCloseButton={false}
  >
    <Dialog.Header class="sr-only">
      <Dialog.Title>{$i18n("palette.title")}</Dialog.Title>
      <Dialog.Description>{$i18n("palette.description")}</Dialog.Description>
    </Dialog.Header>
    <Command.Root bind:value class="rounded-lg border-none shadow-md">
      <Command.Input
        bind:ref={inputRef}
        placeholder={$i18n("palette.searchPlaceholder")}
      />
      <Command.List class="max-h-[60vh]">
        <Command.Empty>{$i18n("palette.noMatches")}</Command.Empty>

        {#if !isColdStart && maps.length > 0}
          <Command.Group heading={$i18n("palette.groupOpenMap")}>
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
          <Command.Group heading={$i18n("palette.groupSwitchProject")}>
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
          <Command.Group heading={$i18n("palette.groupFind")}>
            {#each tracks as t (`${t.layerId}-${t.trackId}`)}
              <Command.Item
                value={`track:${t.layerId}:${t.trackId}:${t.name}`}
                onSelect={() => handleFindTrack(t.layerId, t.trackId)}
              >
                <span class="text-muted-foreground"
                  >{$i18n("palette.track")}</span
                >
                <span class="flex-1 truncate">{t.name}</span>
                <Command.Shortcut>⌘E</Command.Shortcut>
              </Command.Item>
            {/each}
            {#each waypointsCache as w (w.id)}
              <Command.Item
                value={`waypoint:${$activeWaypointLayerId ?? 0}:${w.id}:${w.name}`}
                onSelect={() => handleFindWaypoint(BigInt(w.id))}
              >
                <span class="text-muted-foreground"
                  >{$i18n("palette.waypoint")}</span
                >
                <span class="flex-1 truncate">{w.name}</span>
                <Command.Shortcut>⌘E</Command.Shortcut>
              </Command.Item>
            {/each}
          </Command.Group>
        {/if}

        {#if !isColdStart}
          <Command.Group heading={$i18n("palette.groupProjectActions")}>
            <Command.Item value="action:new" onSelect={handleProjectNew}>
              <span class="flex-1">{$i18n("palette.newProject")}</span>
            </Command.Item>
            <Command.Item value="action:open" onSelect={handleProjectOpen}>
              <span class="flex-1">{$i18n("palette.openProject")}</span>
            </Command.Item>
            <Command.Item value="action:save" onSelect={handleProjectSave}>
              <span class="flex-1">{$i18n("palette.saveProject")}</span>
            </Command.Item>
            <Command.Item value="action:undo" onSelect={handleUndo}>
              <span class="flex-1">{$i18n("palette.undo")}</span>
            </Command.Item>
            <Command.Item value="action:redo" onSelect={handleRedo}>
              <span class="flex-1">{$i18n("palette.redo")}</span>
            </Command.Item>
          </Command.Group>
        {/if}

        <Command.Group heading={$i18n("palette.groupSettings")}>
          {#each THEMES as theme (theme.value)}
            <Command.Item
              value={`setting:theme:${theme.value}`}
              onSelect={() => handleThemeChoice(theme.value)}
            >
              <span class="flex-1">
                {$i18n("palette.theme")} — {theme.label}
              </span>
              {#if $selectedTheme === theme.value}
                <span aria-hidden="true">✓</span>
              {/if}
            </Command.Item>
          {/each}
          <!-- The `palette.language` key renders the OTHER language (the
               toggle target), e.g. "Language: Русский" while English is
               active. -->
          <Command.Item
            value="setting:language"
            onSelect={handleLanguageToggle}
          >
            <span class="flex-1">{$i18n("palette.language")}</span>
          </Command.Item>
          <Command.Item value="setting:units" onSelect={handleUnitsSetting}>
            <span class="flex-1">{$i18n("palette.units")}</span>
            <span class="text-muted-foreground text-[10px]"
              >{$i18n("palette.comingSoon")}</span
            >
          </Command.Item>
          <Command.Item
            value="tool:measure"
            onSelect={() => {
              // Toggling, not starting: the same key that opened the palette
              // is how a crew reaches for it again to put the tape away.
              // Reaching for the tape while it is measuring an area switches
              // it to a length rather than putting it away, because that is
              // what picking the other tool means.
              setMeasuring(
                !($measuringActive && $measureMode === "distance"),
                "distance",
              );
              close();
            }}
          >
            <span class="flex-1"
              >{$measuringActive && $measureMode === "distance"
                ? $i18n("palette.measureStop")
                : $i18n("palette.measure")}</span
            >
          </Command.Item>
          <Command.Item
            value="tool:area"
            onSelect={() => {
              // A separate tool, not a number beside the length. Owner,
              // 2026-09-23: a length is what a crew measures most often, and
              // an area quoted for an open path is a meaningless number in a
              // place people read numbers.
              setMeasuring(
                !($measuringActive && $measureMode === "area"),
                "area",
              );
              close();
            }}
          >
            <span class="flex-1"
              >{$measuringActive && $measureMode === "area"
                ? $i18n("palette.areaStop")
                : $i18n("palette.area")}</span
            >
          </Command.Item>
          <Command.Item
            value="tool:ring"
            onSelect={() => {
              setRing(!$ringActive);
              close();
            }}
          >
            <span class="flex-1"
              >{$ringActive
                ? $i18n("palette.ringStop")
                : $i18n("palette.ring")}</span
            >
          </Command.Item>
          <Command.Item
            value="tool:projection"
            onSelect={() => {
              setProjection(!$projectionActive);
              close();
            }}
          >
            <span class="flex-1"
              >{$projectionActive
                ? $i18n("palette.projectionStop")
                : $i18n("palette.projection")}</span
            >
          </Command.Item>
          <Command.Item value="setting:gps" onSelect={handleGpsSetting}>
            <span class="flex-1">{$i18n("palette.gps")}</span>
            <span class="text-muted-foreground text-[10px]"
              >{$i18n("palette.comingSoon")}</span
            >
          </Command.Item>
        </Command.Group>

        {#if recentProjects.length > 0}
          <Command.Group heading={$i18n("palette.groupRecentProjects")}>
            {#each recentProjects as p (p.path)}
              <Command.Item
                value={`recent-project:${p.path}`}
                onSelect={() => void handleOpenRecentProject(p.path)}
              >
                <span class="flex-1 truncate">{p.name}</span>
                <span class="text-muted-foreground truncate text-[10px]"
                  >{p.path}</span
                >
              </Command.Item>
            {/each}
          </Command.Group>
        {/if}

        {#if recentFiles.length > 0}
          <Command.Group heading={$i18n("palette.groupRecent")}>
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
                    toast.message(
                      $i18n("palette.recentMap").replace("{name}", r.mapName),
                      {
                        description: $i18n("palette.recentNeedsProject"),
                      },
                    );
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
