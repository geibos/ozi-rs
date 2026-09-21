<script lang="ts">
  /**
   * Tracks tab inside the LibraryRail. Lists every track in the active
   * project across all track layers; header carries the active-track-layer
   * `Select`, a single "Import…" button (GPX/PLT/ZIP, multi-select, routed
   * by extension — owner feedback: the twin import icon buttons were
   * indistinguishable), and an "Import folder…" button for recursive
   * directory import (per-date subfolders included).
   *
   * Each row exposes: visibility toggle, color swatch popover, name (double
   * click to rename), distance/duration/point-count subline, a "Show on
   * map" locate button (writes `mapFocusRequest`; MapView fits the bounds),
   * and a `⋯` actions menu with Export GPX, Export PLT, Set line width,
   * Simplify…, Delete.
   *
   * Clicking a row whose owning layer is not the current active layer
   * switches the active layer first (see `layers` capability extension).
   */
  import { Button, buttonVariants } from "$lib/components/ui/button";
  import { Label } from "$lib/components/ui/label";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import * as Popover from "$lib/components/ui/popover";
  import * as Select from "$lib/components/ui/select";
  import { Slider } from "$lib/components/ui/slider";
  import { Switch } from "$lib/components/ui/switch";
  import * as Tooltip from "$lib/components/ui/tooltip";
  import {
    activeTrackLayerId,
    addWaypointMode,
    appState,
    drawingModeActive,
    drawingPointCount,
    drawingSegmentId,
    drawingTrackId,
    drawingTrackLayerId,
    editModeActive,
    requestTrackFocus,
    requestAllTracksFocus,
    selectedTrack,
    simplifyState,
  } from "$lib/stores";
  import {
    createEmptyTrack,
    deleteTrack,
    exportAllTracksGpx,
    exportGpx,
    exportTrackPlt,
    getSimplifiedPreview,
    getTrackDetail,
    getTrackExportDefaultPath,
    listTracks,
    importGpx,
    importPlt,
    importTracksDirectory,
    renameTrack,
    setAllTracksVisible,
    setTrackColor,
    showOnlyTrack,
    setTrackLineWidth,
    simplifyTrack,
    toggleTrackVisible,
  } from "$lib/api";
  import { open } from "@tauri-apps/plugin-dialog";
  import { toast } from "svelte-sonner";
  import UploadIcon from "@lucide/svelte/icons/upload";
  import FolderOpenIcon from "@lucide/svelte/icons/folder-open";
  import DownloadIcon from "@lucide/svelte/icons/download";
  import LocateIcon from "@lucide/svelte/icons/locate";
  import CircleAlertIcon from "@lucide/svelte/icons/circle-alert";
  import PencilIcon from "@lucide/svelte/icons/pencil";
  import CheckIcon from "@lucide/svelte/icons/check";
  import { isOkStandardTrackName } from "$lib/track-names";
  import { locale, t as i18n } from "$lib/i18n";
  import { layerDisplayName } from "$lib/layer-names";
  import { formatTrackStats } from "$lib/track-stats";
  import LibraryRow from "./LibraryRow.svelte";
  import {
    filterTrackFeatures,
    trackFeaturesFromSummaries,
    type TrackFeature,
  } from "$lib/track-features";
  import { get } from "svelte/store";
  import { Input } from "$lib/components/ui/input";
  import SearchIcon from "@lucide/svelte/icons/search";
  import EyeIcon from "@lucide/svelte/icons/eye";
  import EyeOffIcon from "@lucide/svelte/icons/eye-off";
  import XIcon from "@lucide/svelte/icons/x";

  let tracks: TrackFeature[] = $state([]);
  // A field project carries dozens of tracks named by date and call sign, so
  // finding one by eye is the slowest step in cleaning up a search.
  let trackQuery = $state("");
  const visibleTracks = $derived(filterTrackFeatures(tracks, trackQuery));
  // Inline-popover live-preview toggle. The popover writes through the
  // shared `simplifyState` store so MapView's preview overlay effect can
  // render the simplified geometry without a separate coupling path.
  let simplifyLivePreview = $state(true);
  let simplifyDebounceTimer: ReturnType<typeof setTimeout> | null = null;

  const trackLayers = $derived($appState?.track_layers ?? []);
  const trackLayerSelectValue = $derived(
    $activeTrackLayerId !== null ? $activeTrackLayerId.toString() : "",
  );
  // Import-created layers are named after the source path ("Imported
  // tracks: /very/long/path.gpx") — the trigger text must truncate, not
  // overflow the 280px rail; `title` keeps the full name reachable.
  const activeTrackLayerName = $derived.by(() => {
    const name = trackLayers.find(
      (l) => String(l.id) === trackLayerSelectValue,
    )?.name;
    return name === undefined ? null : layerDisplayName(name, $locale);
  });

  $effect(() => {
    if ($appState) {
      void loadTracks();
    }
  });

  async function handleSetAllVisible(visible: boolean) {
    try {
      await setAllTracksVisible(visible);
    } catch (err) {
      toast.error(get(i18n)("tracksTab.visibilityFailed"), {
        description: String(err),
      });
    }
  }

  async function handleShowOnly(track: TrackFeature) {
    try {
      await showOnlyTrack(track.layerId, track.trackId);
    } catch (err) {
      toast.error(get(i18n)("tracksTab.visibilityFailed"), {
        description: String(err),
      });
    }
  }

  async function loadTracks() {
    try {
      tracks = trackFeaturesFromSummaries(await listTracks());
    } catch (err) {
      console.error("Failed to load tracks", err);
      toast.error(get(i18n)("tracksTab.loadFailed"), {
        description: String(err),
      });
    }
  }

  function trackKey(t: TrackFeature): string {
    return `${t.layerId}:${t.trackId}`;
  }

  function isSelected(t: TrackFeature): boolean {
    return (
      $selectedTrack?.layerId === t.layerId &&
      $selectedTrack?.trackId === t.trackId
    );
  }

  function colorToHex(color: string): string {
    if (/^#[0-9a-f]{6}$/i.test(color)) {
      return color;
    }
    const match = color.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/i);
    if (!match) {
      return "#000000";
    }
    return [match[1], match[2], match[3]]
      .map((c) => Number(c).toString(16).padStart(2, "0"))
      .join("")
      .replace(/^/, "#");
  }

  function hexToRgba(hex: string): [number, number, number, number] {
    const n = hex.replace("#", "");
    return [
      parseInt(n.slice(0, 2), 16),
      parseInt(n.slice(2, 4), 16),
      parseInt(n.slice(4, 6), 16),
      255,
    ];
  }

  async function handleColorChange(t: TrackFeature, event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    await setTrackColor(t.layerId, t.trackId, hexToRgba(input.value));
  }

  async function handleSetLineWidth(t: TrackFeature, width: number) {
    await setTrackLineWidth(t.layerId, t.trackId, width);
  }

  async function handleToggleVisibility(t: TrackFeature) {
    await toggleTrackVisible(t.layerId, t.trackId);
  }

  async function handleRename(t: TrackFeature, newName: string) {
    await renameTrack(t.layerId, t.trackId, newName);
  }

  function handleSelectRow(t: TrackFeature) {
    // Decision 4 of design.md / layers spec: clicking a row in a non-active
    // layer flips the active track layer to the row's owning layer before
    // applying selection. No-op when the layer is already active.
    if ($activeTrackLayerId !== t.layerId && !$drawingModeActive) {
      activeTrackLayerId.set(t.layerId);
    }
    selectedTrack.set({ layerId: t.layerId, trackId: t.trackId });
  }

  async function handleExportGpx(t: TrackFeature) {
    const defaultPath = await getTrackExportDefaultPath(t.name, "gpx");
    const path = await open({
      save: true,
      defaultPath: defaultPath ?? `${t.name}.gpx`,
      filters: [{ name: "GPX", extensions: ["gpx"] }],
    } as Parameters<typeof open>[0]);
    if (path) await exportGpx(t.layerId, path as string);
  }

  async function handleExportPlt(t: TrackFeature) {
    const defaultPath = await getTrackExportDefaultPath(t.name, "plt");
    const path = await open({
      save: true,
      defaultPath: defaultPath ?? `${t.name}.plt`,
      filters: [{ name: "PLT", extensions: ["plt"] }],
    } as Parameters<typeof open>[0]);
    if (path) await exportTrackPlt(t.layerId, t.trackId, path as string);
  }

  /** Strip directories from a path for compact failure reporting. */
  function baseName(path: string): string {
    const parts = path.split(/[\\/]/);
    return parts[parts.length - 1] || path;
  }

  /**
   * Single "Import…" entry point. One dialog with a combined GPX/PLT/ZIP
   * filter (`.zip` because the backend GPX importer unpacks zip archives
   * of gpx — the old twin-button UI wrongly hid that), `multiple: true`.
   * Routing by extension: `.plt` → importPlt, `.gpx` / `.zip` → importGpx.
   * Files import sequentially; per-file failures are collected and the
   * outcome is reported in ONE summary toast.
   */
  async function handleImport() {
    try {
      const selection = await open({
        multiple: true,
        directory: false,
        filters: [
          {
            name: $i18n("tracksTab.importFilterName"),
            extensions: ["gpx", "plt", "zip"],
          },
        ],
      });
      if (!selection) return;
      const paths = (
        Array.isArray(selection) ? selection : [selection]
      ) as string[];

      let imported = 0;
      const failed: string[] = [];
      for (const path of paths) {
        try {
          if (path.toLowerCase().endsWith(".plt")) {
            await importPlt(path);
          } else {
            await importGpx(path);
          }
          imported += 1;
        } catch (err) {
          console.error("Failed to import track file", path, err);
          failed.push(baseName(path));
        }
      }

      const summary = $i18n("tracksTab.importDone")
        .replace("{count}", String(imported))
        .replace("{total}", String(paths.length));
      if (imported > 0) requestAllTracksFocus();
      if (failed.length === 0) {
        toast.success(summary);
      } else {
        toast.error(summary, {
          description: $i18n("tracksTab.importFailedFiles").replace(
            "{files}",
            failed.join(", "),
          ),
        });
      }
    } catch (err) {
      toast.error($i18n("tracksTab.importFailed"), {
        description: String(err),
      });
    }
  }

  /**
   * "Import folder…" — field archives arrive as a directory with per-date
   * subfolders. The backend command walks it recursively and returns a
   * human-readable summary string (per-file failures folded in).
   */
  async function handleImportFolder() {
    try {
      const dir = await open({ directory: true, multiple: false });
      if (!dir) return;
      const summary = await importTracksDirectory(dir as string);
      requestAllTracksFocus();
      toast.success(summary);
    } catch (err) {
      toast.error($i18n("tracksTab.importFolderFailed"), {
        description: String(err),
      });
    }
  }

  /**
   * Hand the day's work over in one file.
   *
   * A folder import makes one layer per navigator, so exporting "today's
   * tracks" was one dialog per layer — the same twenty-six-clicks shape the
   * visibility toggles had.
   */
  async function handleExportAll() {
    try {
      const defaultPath = await getTrackExportDefaultPath(
        $appState?.project_name ?? "tracks",
        "gpx",
      );
      const path = await open({
        save: true,
        defaultPath: defaultPath ?? "tracks.gpx",
        filters: [{ name: "GPX", extensions: ["gpx"] }],
      } as Parameters<typeof open>[0]);
      if (!path) return;
      const count = await exportAllTracksGpx(path as string);
      toast.success(
        $i18n("tracksTab.exportAllDone").replace("{count}", String(count)),
      );
    } catch (err) {
      toast.error($i18n("tracksTab.exportAllFailed"), {
        description: String(err),
      });
    }
  }

  async function handleCreateTrackToggle() {
    // Toggle drawing mode. On exit we ask MapView to finish via the
    // existing `drawingFinishRequested` signal it already listens to,
    // but the legacy Sidebar simply flipped `drawingModeActive` to false
    // and the same wiring works here — MapView's effect drains the
    // pending preview when the mode goes false.
    if ($drawingModeActive) {
      drawingModeActive.set(false);
      return;
    }

    const layerId = $activeTrackLayerId;
    if (layerId === null) return;

    try {
      editModeActive.set(false);
      addWaypointMode.set(false);
      const trackId = await createEmptyTrack(layerId, "New Track");
      const detail = await getTrackDetail(layerId, trackId);
      drawingTrackLayerId.set(layerId);
      drawingTrackId.set(trackId);
      drawingSegmentId.set(BigInt(detail.segments[0].id));
      drawingPointCount.set(0);
      drawingModeActive.set(true);
    } catch (err) {
      console.error("Failed to start track drawing mode", err);
      toast.error("Failed to start track drawing mode", {
        description: String(err),
      });
    }
  }

  async function handleDelete(t: TrackFeature) {
    try {
      await deleteTrack(t.layerId, t.trackId);
    } catch (err) {
      toast.error("Failed to delete track", { description: String(err) });
    }
  }

  function openSimplify(t: TrackFeature) {
    simplifyState.set({
      active: true,
      layerId: t.layerId,
      trackId: t.trackId,
      tolerance: 10,
      preview: null,
    });
    simplifyLivePreview = true;
    schedulePreview();
  }

  function closeSimplify() {
    simplifyState.update((s) => ({ ...s, active: false, preview: null }));
    if (simplifyDebounceTimer) clearTimeout(simplifyDebounceTimer);
  }

  function schedulePreview() {
    if (!simplifyLivePreview) {
      simplifyState.update((s) => ({ ...s, preview: null }));
      return;
    }
    if (simplifyDebounceTimer) clearTimeout(simplifyDebounceTimer);
    simplifyDebounceTimer = setTimeout(async () => {
      const s = $simplifyState;
      if (!s.active) return;
      try {
        const preview = await getSimplifiedPreview(
          s.layerId,
          s.trackId,
          s.tolerance,
        );
        simplifyState.update((cur) => ({ ...cur, preview }));
      } catch (err) {
        console.error("Failed to compute simplified preview", err);
        toast.error("Failed to compute simplified preview", {
          description: String(err),
        });
      }
    }, 300);
  }

  async function commitSimplify() {
    const s = $simplifyState;
    if (!s.active) return;
    try {
      await simplifyTrack(s.layerId, s.trackId, s.tolerance);
      closeSimplify();
    } catch (err) {
      toast.error("Failed to simplify track", { description: String(err) });
    }
  }
</script>

<div class="flex h-full min-h-0 flex-col">
  <header class="border-border border-b px-2 py-1.5">
    {#if trackLayers.length > 0}
      <Label class="text-muted-foreground text-[10px]">
        {$i18n("tracksTab.layer")}
      </Label>
      <div class="flex items-center gap-1">
        <Select.Root
          type="single"
          value={trackLayerSelectValue}
          onValueChange={(v) => v && activeTrackLayerId.set(BigInt(v))}
          disabled={$drawingModeActive}
        >
          <Select.Trigger
            aria-label={$i18n("tracksTab.layer")}
            size="sm"
            class="min-w-0 flex-1"
            title={activeTrackLayerName}
          >
            <span class="min-w-0 flex-1 truncate text-left">
              {activeTrackLayerName ?? $i18n("tracksTab.pickLayer")}
            </span>
          </Select.Trigger>
          <Select.Content class="max-w-72">
            {#each trackLayers as layer (layer.id)}
              <Select.Item
                value={String(layer.id)}
                label={layerDisplayName(layer.name, $locale)}
              >
                <span
                  class="min-w-0 truncate"
                  title={layerDisplayName(layer.name, $locale)}
                >
                  {layerDisplayName(layer.name, $locale)}
                </span>
              </Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>

        <Tooltip.Root>
          <Tooltip.Trigger
            class={buttonVariants({
              variant: $drawingModeActive ? "default" : "ghost",
              size: $drawingModeActive ? "sm" : "icon-sm",
            })}
            aria-label={$drawingModeActive
              ? $i18n("tracksTab.finishTrack").replace(
                  "{count}",
                  String($drawingPointCount),
                )
              : $i18n("tracksTab.createTrack")}
            aria-pressed={$drawingModeActive}
            disabled={$activeTrackLayerId === null}
            onclick={handleCreateTrackToggle}
            data-testid="library-create-track"
          >
            {#if $drawingModeActive}
              <CheckIcon strokeWidth={1.5} />
              <span class="text-xs">
                {$i18n("tracksTab.finishTrackShort").replace(
                  "{count}",
                  String($drawingPointCount),
                )}
              </span>
            {:else}
              <PencilIcon strokeWidth={1.5} />
            {/if}
          </Tooltip.Trigger>
          <Tooltip.Content>
            {$drawingModeActive
              ? $i18n("tracksTab.finishTrack").replace(
                  "{count}",
                  String($drawingPointCount),
                )
              : $i18n("tracksTab.createTrack")}
          </Tooltip.Content>
        </Tooltip.Root>
      </div>

      <div class="mt-1.5 flex items-center gap-1">
        <Button
          variant="outline"
          size="xs"
          class="min-w-0 flex-1 justify-center gap-1.5"
          disabled={$drawingModeActive || $activeTrackLayerId === null}
          onclick={handleImport}
          data-testid="library-import-tracks"
        >
          <UploadIcon class="size-3.5" strokeWidth={1.5} />
          <span class="truncate">{$i18n("tracksTab.import")}</span>
        </Button>
        <Button
          variant="outline"
          size="xs"
          class="min-w-0 flex-1 justify-center gap-1.5"
          disabled={$drawingModeActive || $activeTrackLayerId === null}
          onclick={handleImportFolder}
          data-testid="library-import-folder"
        >
          <FolderOpenIcon class="size-3.5" strokeWidth={1.5} />
          <span class="truncate">{$i18n("tracksTab.importFolder")}</span>
        </Button>
      </div>

      {#if tracks.length > 0}
        <div class="mt-1.5 flex">
          <Button
            variant="outline"
            size="xs"
            class="min-w-0 flex-1 justify-center gap-1.5"
            disabled={$drawingModeActive}
            title={$i18n("tracksTab.exportAllTitle")}
            onclick={handleExportAll}
            data-testid="library-export-all-tracks"
          >
            <DownloadIcon class="size-3.5" strokeWidth={1.5} />
            <span class="truncate">{$i18n("tracksTab.exportAll")}</span>
          </Button>
        </div>
      {/if}
    {/if}

    {#if tracks.length > 0}
      <div class="mt-1.5 flex items-center gap-1">
        <div class="relative min-w-0 flex-1">
          <SearchIcon
            class="text-muted-foreground pointer-events-none absolute top-1/2 left-2 size-3.5 -translate-y-1/2"
            strokeWidth={1.5}
          />
          <Input
            bind:value={trackQuery}
            class="h-7 pr-7 pl-7 text-xs"
            placeholder={$i18n("tracksTab.searchPlaceholder")}
            aria-label={$i18n("tracksTab.searchPlaceholder")}
            data-testid="track-search"
          />
          {#if trackQuery !== ""}
            <button
              type="button"
              class="text-muted-foreground hover:text-foreground absolute top-1/2 right-1 inline-flex size-5 -translate-y-1/2 items-center justify-center rounded-sm border-0 bg-transparent p-0"
              aria-label={$i18n("tracksTab.searchClear")}
              onclick={() => (trackQuery = "")}
              data-testid="track-search-clear"
            >
              <XIcon class="size-3.5" strokeWidth={2} />
            </button>
          {/if}
        </div>
        <Tooltip.Root>
          <Tooltip.Trigger
            class="text-muted-foreground hover:text-foreground inline-flex size-6 shrink-0 items-center justify-center rounded-sm border-0 bg-transparent p-0"
            aria-label={$i18n("tracksTab.showAll")}
            onclick={() => handleSetAllVisible(true)}
            data-testid="tracks-show-all"
          >
            <EyeIcon class="size-3.5" strokeWidth={1.5} />
          </Tooltip.Trigger>
          <Tooltip.Content>{$i18n("tracksTab.showAll")}</Tooltip.Content>
        </Tooltip.Root>
        <Tooltip.Root>
          <Tooltip.Trigger
            class="text-muted-foreground hover:text-foreground inline-flex size-6 shrink-0 items-center justify-center rounded-sm border-0 bg-transparent p-0"
            aria-label={$i18n("tracksTab.hideAll")}
            onclick={() => handleSetAllVisible(false)}
            data-testid="tracks-hide-all"
          >
            <EyeOffIcon class="size-3.5" strokeWidth={1.5} />
          </Tooltip.Trigger>
          <Tooltip.Content>{$i18n("tracksTab.hideAll")}</Tooltip.Content>
        </Tooltip.Root>
        {#if trackQuery !== ""}
          <span
            class="text-muted-foreground shrink-0 font-mono text-[10px] tabular-nums"
            data-testid="track-search-count"
          >
            {$i18n("tracksTab.searchCount")
              .replace("{shown}", String(visibleTracks.length))
              .replace("{total}", String(tracks.length))}
          </span>
        {/if}
      </div>
    {/if}
  </header>

  <div class="flex-1 overflow-y-auto py-1" data-testid="tracks-tab-list">
    {#if tracks.length === 0}
      <div class="text-muted-foreground p-3 text-center text-xs">
        {$i18n("tracksTab.empty")}
      </div>
    {:else if visibleTracks.length === 0}
      <div
        class="text-muted-foreground p-3 text-center text-xs"
        data-testid="track-search-empty"
      >
        {$i18n("tracksTab.searchEmpty")}
      </div>
    {:else}
      {#each visibleTracks as t (trackKey(t))}
        <LibraryRow
          name={t.name}
          visible={t.visible}
          selected={isSelected(t)}
          onToggleVisibility={() => handleToggleVisibility(t)}
          onSelect={() => handleSelectRow(t)}
          onRename={(name) => handleRename(t, name)}
        >
          {#snippet leadingControl()}
            <Popover.Root>
              <Popover.Trigger
                class="border-border size-4 shrink-0 rounded-full border p-0"
                style="background-color: {t.color}"
                aria-label="Track color"
              ></Popover.Trigger>
              <Popover.Content class="w-auto p-2">
                <input
                  class="border-border h-8 w-12 rounded-sm border bg-transparent p-0"
                  type="color"
                  value={colorToHex(t.color)}
                  aria-label="Track color"
                  onchange={(e) => handleColorChange(t, e)}
                />
              </Popover.Content>
            </Popover.Root>
          {/snippet}
          {#snippet nameSuffix()}
            {#if !isOkStandardTrackName(t.name)}
              <!-- A non-standard name is worth flagging, but spelling the rule
                   out under every row buried the names themselves. The glyph
                   carries the same text as its tooltip and label. -->
              <Tooltip.Root>
                <Tooltip.Trigger
                  class="shrink-0 border-0 bg-transparent p-0 text-yellow-500 hover:text-yellow-400"
                  aria-label={$i18n("tracksTab.nameHint")}
                  data-testid="track-name-warning"
                >
                  <CircleAlertIcon class="size-3" strokeWidth={2} />
                </Tooltip.Trigger>
                <Tooltip.Content>{$i18n("tracksTab.nameHint")}</Tooltip.Content>
              </Tooltip.Root>
            {/if}
          {/snippet}
          {#snippet subline()}
            <span
              class="text-muted-foreground truncate font-mono text-[10px] leading-tight tabular-nums"
              data-testid="track-stats"
              title={t.durationSeconds !== null
                ? $i18n("track.durationTooltip")
                : undefined}
            >
              {formatTrackStats(
                t.distanceKm,
                t.durationSeconds,
                t.pointCount,
                $locale,
              )}
            </span>
          {/snippet}
          {#snippet trailingControl()}
            <Tooltip.Root>
              <Tooltip.Trigger
                class="text-muted-foreground hover:text-foreground inline-flex size-6 items-center justify-center rounded-sm border-0 bg-transparent p-0"
                aria-label={$i18n("track.showOnMap")}
                onclick={() => requestTrackFocus(t.layerId, t.trackId)}
                data-testid="track-show-on-map"
              >
                <LocateIcon class="size-3.5" strokeWidth={1.5} />
              </Tooltip.Trigger>
              <Tooltip.Content>{$i18n("track.showOnMap")}</Tooltip.Content>
            </Tooltip.Root>
          {/snippet}
          {#snippet actions()}
            <DropdownMenu.Item onSelect={() => handleShowOnly(t)}>
              {$i18n("tracksTab.onlyThis")}
            </DropdownMenu.Item>
            <DropdownMenu.Separator />
            <DropdownMenu.Item onSelect={() => handleExportGpx(t)}>
              {$i18n("tracksTab.exportGpx")}
            </DropdownMenu.Item>
            <DropdownMenu.Item onSelect={() => handleExportPlt(t)}>
              {$i18n("tracksTab.exportPlt")}
            </DropdownMenu.Item>
            <DropdownMenu.Separator />
            <Popover.Root>
              <Popover.Trigger
                class="hover:bg-accent hover:text-accent-foreground relative flex w-full cursor-default items-center gap-2 rounded-sm border-0 bg-transparent px-2 py-1.5 text-xs select-none"
                onclick={(e: Event) => e.stopPropagation()}
              >
                Set line width
              </Popover.Trigger>
              <Popover.Content class="w-44">
                <Label class="text-xs">Line width: {t.lineWidth}px</Label>
                <Slider
                  type="single"
                  min={1}
                  max={12}
                  step={1}
                  value={t.lineWidth}
                  onValueChange={(v) => handleSetLineWidth(t, v)}
                />
              </Popover.Content>
            </Popover.Root>
            <DropdownMenu.Item
              onSelect={(e: Event) => {
                e.preventDefault();
                openSimplify(t);
              }}
            >
              Simplify…
            </DropdownMenu.Item>
            <DropdownMenu.Separator />
            <DropdownMenu.Item
              variant="destructive"
              onSelect={() => handleDelete(t)}
            >
              Delete
            </DropdownMenu.Item>
          {/snippet}
        </LibraryRow>

        {#if $simplifyState.active && $simplifyState.layerId === t.layerId && $simplifyState.trackId === t.trackId}
          <div
            class="bg-card text-card-foreground border-border mx-2 mt-1 mb-2 flex flex-col gap-2 rounded-md border p-2"
            data-testid="simplify-popover"
          >
            <div class="text-xs font-semibold">Simplify Track</div>
            <Label class="text-xs">
              Tolerance: {$simplifyState.tolerance}m
            </Label>
            <Slider
              type="single"
              min={1}
              max={1000}
              step={1}
              value={$simplifyState.tolerance}
              onValueChange={(v) => {
                simplifyState.update((s) => ({ ...s, tolerance: v as number }));
                schedulePreview();
              }}
            />
            <div class="flex items-center gap-2">
              <Switch
                bind:checked={simplifyLivePreview}
                onCheckedChange={(v) => {
                  simplifyLivePreview = v;
                  if (v) schedulePreview();
                  else simplifyState.update((s) => ({ ...s, preview: null }));
                }}
              />
              <Label class="text-xs">Live preview</Label>
            </div>
            {#if $simplifyState.preview}
              <div
                class="bg-muted text-muted-foreground rounded p-2 text-[11px] leading-snug"
              >
                Original:
                <strong class="text-foreground">
                  {$simplifyState.preview.original_count}
                </strong>
                → Simplified:
                <strong class="text-foreground">
                  {$simplifyState.preview.simplified_count}
                </strong>
              </div>
            {/if}
            <div class="flex justify-end gap-2">
              <Button variant="outline" size="xs" onclick={closeSimplify}>
                Cancel
              </Button>
              <Button
                size="xs"
                disabled={!$simplifyState.preview ||
                  $simplifyState.preview.simplified_count === 0}
                onclick={commitSimplify}
              >
                Confirm
              </Button>
            </div>
          </div>
        {/if}
      {/each}
    {/if}
  </div>
</div>
