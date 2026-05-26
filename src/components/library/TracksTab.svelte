<script lang="ts">
  /**
   * Tracks tab inside the LibraryRail. Lists every track in the active
   * project across all track layers; header carries the active-track-layer
   * `Select`.
   *
   * Each row exposes: visibility toggle, color swatch popover, name (double
   * click to rename), distance/duration/point-count subline, and a `⋯`
   * actions menu with Export GPX, Export PLT, Set line width, Simplify…,
   * Delete.
   *
   * Clicking a row whose owning layer is not the current active layer
   * switches the active layer first (see `layers` capability extension).
   */
  import { Button } from "$lib/components/ui/button";
  import { Label } from "$lib/components/ui/label";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import * as Popover from "$lib/components/ui/popover";
  import * as Select from "$lib/components/ui/select";
  import { Slider } from "$lib/components/ui/slider";
  import { Switch } from "$lib/components/ui/switch";
  import {
    activeTrackLayerId,
    appState,
    drawingModeActive,
    selectedTrack,
    simplifyState,
  } from "$lib/stores";
  import {
    deleteTrack,
    exportGpx,
    exportTrackPlt,
    getSimplifiedPreview,
    getTrackExportDefaultPath,
    getTracksGeojson,
    renameTrack,
    setTrackColor,
    setTrackLineWidth,
    simplifyTrack,
    toggleTrackVisible,
  } from "$lib/api";
  import { open } from "@tauri-apps/plugin-dialog";
  import { toast } from "svelte-sonner";
  import { isOkStandardTrackName } from "$lib/track-names";
  import { formatTrackStats } from "$lib/track-stats";
  import LibraryRow from "./LibraryRow.svelte";

  interface TrackFeature {
    layerId: bigint;
    trackId: bigint;
    name: string;
    color: string;
    lineWidth: number;
    visible: boolean;
    distanceKm: number;
    durationSeconds: number | null;
    pointCount: number;
  }

  let tracks: TrackFeature[] = $state([]);
  // Inline-popover live-preview toggle. The popover writes through the
  // shared `simplifyState` store so MapView's preview overlay effect can
  // render the simplified geometry without a separate coupling path.
  let simplifyLivePreview = $state(true);
  let simplifyDebounceTimer: ReturnType<typeof setTimeout> | null = null;

  const trackLayers = $derived($appState?.track_layers ?? []);
  const trackLayerSelectValue = $derived(
    $activeTrackLayerId !== null ? $activeTrackLayerId.toString() : "",
  );

  $effect(() => {
    if ($appState) {
      void loadTracks();
    }
  });

  // Sort by (layer, trackId) so rows from one layer cluster.
  function sortKey(t: TrackFeature): string {
    const layerStr = t.layerId.toString().padStart(20, "0");
    const trackStr = t.trackId.toString().padStart(20, "0");
    return `${layerStr}:${trackStr}`;
  }

  async function loadTracks() {
    try {
      const geojson = await getTracksGeojson();
      const next = geojson.features
        .filter((f) => f.geometry.type === "LineString")
        .map((f) => {
          const rawDuration = f.properties!.duration_seconds as
            | number
            | null
            | undefined;
          return {
            layerId: BigInt(f.properties!.layer_id as number),
            trackId: BigInt(f.properties!.track_id as number),
            name: f.properties!.name as string,
            color: f.properties!.color as string,
            lineWidth: Number(f.properties!.line_width ?? 3),
            visible: f.properties!.visible as boolean,
            distanceKm: Number(f.properties!.distance_km ?? 0),
            durationSeconds:
              rawDuration === null || rawDuration === undefined
                ? null
                : Number(rawDuration),
            pointCount: Number(f.properties!.point_count ?? 0),
          } as TrackFeature;
        });
      next.sort((a, b) => sortKey(a).localeCompare(sortKey(b)));
      tracks = next;
    } catch (err) {
      console.error("Failed to load tracks", err);
      toast.error("Failed to load tracks", { description: String(err) });
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
      <Label class="text-muted-foreground text-[10px]">Track layer</Label>
      <Select.Root
        type="single"
        value={trackLayerSelectValue}
        onValueChange={(v) => v && activeTrackLayerId.set(BigInt(v))}
        disabled={$drawingModeActive}
      >
        <Select.Trigger aria-label="Track layer" size="sm" class="w-full">
          {trackLayers.find((l) => String(l.id) === trackLayerSelectValue)
            ?.name ?? "Pick layer"}
        </Select.Trigger>
        <Select.Content>
          {#each trackLayers as layer (layer.id)}
            <Select.Item value={String(layer.id)} label={layer.name}>
              {layer.name}
            </Select.Item>
          {/each}
        </Select.Content>
      </Select.Root>
    {/if}
  </header>

  <div class="flex-1 overflow-y-auto py-1" data-testid="tracks-tab-list">
    {#if tracks.length === 0}
      <div class="text-muted-foreground p-3 text-center text-xs">
        No tracks loaded
      </div>
    {:else}
      {#each tracks as t (trackKey(t))}
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
                class="border-border size-4 shrink-0 rounded-full border"
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
              <span class="text-[10px] leading-tight text-yellow-500">
                Use YYYYMMDD_Callsign
              </span>
            {/if}
          {/snippet}
          {#snippet subline()}
            <span
              class="text-muted-foreground truncate font-mono text-[10px] leading-tight tabular-nums"
              data-testid="track-stats"
            >
              {formatTrackStats(
                t.distanceKm,
                t.durationSeconds,
                t.pointCount,
              )}
            </span>
          {/snippet}
          {#snippet actions()}
            <DropdownMenu.Item onSelect={() => handleExportGpx(t)}>
              Export GPX
            </DropdownMenu.Item>
            <DropdownMenu.Item onSelect={() => handleExportPlt(t)}>
              Export PLT
            </DropdownMenu.Item>
            <DropdownMenu.Separator />
            <Popover.Root>
              <Popover.Trigger
                class="hover:bg-accent hover:text-accent-foreground relative flex w-full cursor-default select-none items-center gap-2 rounded-sm px-2 py-1.5 text-xs"
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
                  else
                    simplifyState.update((s) => ({ ...s, preview: null }));
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
