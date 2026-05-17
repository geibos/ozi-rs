<script lang="ts">
  import DownloadIcon from "@lucide/svelte/icons/download";
  import EyeIcon from "@lucide/svelte/icons/eye";
  import EyeOffIcon from "@lucide/svelte/icons/eye-off";
  import FileOutputIcon from "@lucide/svelte/icons/file-output";
  import WavesIcon from "@lucide/svelte/icons/waves";
  import XIcon from "@lucide/svelte/icons/x";
  import { Button } from "$lib/components/ui/button";
  import { Separator } from "$lib/components/ui/separator";
  import * as Tooltip from "$lib/components/ui/tooltip";
  import {
    appState,
    tracksPanelOpen,
    selectedTrack,
    simplifyState,
  } from "$lib/stores";
  import {
    getTracksGeojson,
    renameTrack,
    toggleTrackVisible,
    exportGpx,
    getTrackExportDefaultPath,
    exportTrackPlt,
    setTrackColor,
    setTrackLineWidth,
  } from "$lib/api";
  import { open } from "@tauri-apps/plugin-dialog";
  import { isOkStandardTrackName } from "$lib/track-names";
  import { formatTrackStats } from "$lib/track-stats";
  import SimplifyPanel from "./SimplifyPanel.svelte";

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
  let editingTrack: string | null = $state(null);
  let editName = $state("");

  function focusOnMount(node: HTMLInputElement) {
    node.focus();
  }

  $effect(() => {
    if ($appState) {
      loadTracks();
    }
  });

  async function loadTracks() {
    const geojson = await getTracksGeojson();
    tracks = geojson.features
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
        };
      });
  }

  function colorToHex(color: string) {
    if (/^#[0-9a-f]{6}$/i.test(color)) {
      return color;
    }

    const match = color.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/i);
    if (!match) {
      return "#000000";
    }

    return [match[1], match[2], match[3]]
      .map((channel) => Number(channel).toString(16).padStart(2, "0"))
      .join("")
      .replace(/^/, "#");
  }

  function hexToRgba(hex: string): [number, number, number, number] {
    const normalized = hex.replace("#", "");
    return [
      parseInt(normalized.slice(0, 2), 16),
      parseInt(normalized.slice(2, 4), 16),
      parseInt(normalized.slice(4, 6), 16),
      255,
    ];
  }

  function trackIdentity(track: TrackFeature) {
    return `${track.layerId}:${track.trackId}`;
  }

  function isSelectedTrack(track: TrackFeature) {
    return (
      $selectedTrack?.layerId === track.layerId &&
      $selectedTrack?.trackId === track.trackId
    );
  }

  async function handleColorChange(track: TrackFeature, event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    await setTrackColor(track.layerId, track.trackId, hexToRgba(input.value));
    track.color = input.value;
  }

  async function handleLineWidthChange(track: TrackFeature, event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const width = Number(input.value);
    await setTrackLineWidth(track.layerId, track.trackId, width);
    track.lineWidth = width;
  }

  function startRename(track: TrackFeature) {
    editingTrack = trackIdentity(track);
    editName = track.name;
  }

  async function commitRename(track: TrackFeature) {
    if (editName.trim() && editName !== track.name) {
      await renameTrack(track.layerId, track.trackId, editName.trim());
    }
    editingTrack = null;
  }

  async function handleToggleVisible(track: TrackFeature) {
    await toggleTrackVisible(track.layerId, track.trackId);
  }

  async function handleExport(track: TrackFeature) {
    const defaultPath = await getTrackExportDefaultPath(track.name, "gpx");
    const path = await open({
      save: true,
      defaultPath: defaultPath ?? `${track.name}.gpx`,
      filters: [{ name: "GPX", extensions: ["gpx"] }],
    } as Parameters<typeof open>[0]);
    if (path) {
      await exportGpx(track.layerId, path as string);
    }
  }

  async function handleExportPlt(track: TrackFeature) {
    const defaultPath = await getTrackExportDefaultPath(track.name, "plt");
    const path = await open({
      save: true,
      defaultPath: defaultPath ?? `${track.name}.plt`,
      filters: [{ name: "PLT", extensions: ["plt"] }],
    } as Parameters<typeof open>[0]);
    if (path) {
      await exportTrackPlt(track.layerId, track.trackId, path as string);
    }
  }
</script>

{#if $tracksPanelOpen}
  <div
    class="bg-popover text-popover-foreground border-border fixed top-[60px] right-4 z-[200] flex max-h-[calc(100vh-80px)] w-72 resize flex-col overflow-hidden rounded-lg border shadow-lg"
  >
    <div
      class="bg-card text-muted-foreground border-border flex cursor-move items-center justify-between border-b px-2.5 py-1 text-xs select-none"
    >
      <span>Tracks ({tracks.length})</span>
      <Button
        variant="ghost"
        size="icon-xs"
        onclick={() => tracksPanelOpen.set(false)}
        aria-label="Close tracks panel"
      >
        <XIcon />
      </Button>
    </div>
    <div class="flex-1 overflow-y-auto py-1">
      {#if tracks.length === 0}
        <div class="text-muted-foreground p-3 text-center text-xs">
          No tracks loaded
        </div>
      {:else}
          {#each tracks as track, idx (trackIdentity(track))}
            {#if idx > 0}
              <Separator />
            {/if}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="hover:bg-muted flex cursor-pointer items-center gap-1.5 px-2.5 py-1 transition-colors"
              class:bg-accent={isSelectedTrack(track)}
              class:text-accent-foreground={isSelectedTrack(track)}
              class:opacity-40={!track.visible}
              onclick={() =>
                selectedTrack.set({
                  layerId: track.layerId,
                  trackId: track.trackId,
                })}
            >
              <span
                class="size-2.5 shrink-0 rounded-full"
                style="background: {track.color}"
              ></span>

              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="flex shrink-0 items-center gap-1"
                onclick={(event) => event.stopPropagation()}
                onmousedown={(event) => event.stopPropagation()}
              >
                <input
                  class="border-border h-[18px] w-[18px] rounded-sm border bg-transparent p-0"
                  type="color"
                  value={colorToHex(track.color)}
                  title="Track color"
                  aria-label="Track color"
                  onchange={(event) => handleColorChange(track, event)}
                />
                <input
                  class="accent-primary w-[42px]"
                  type="range"
                  min="1"
                  max="12"
                  step="1"
                  value={track.lineWidth}
                  title={`Track width: ${track.lineWidth}px`}
                  aria-label="Track line width"
                  onchange={(event) => handleLineWidthChange(track, event)}
                />
                <span
                  class="text-muted-foreground min-w-[24px] text-[10px]"
                  >{track.lineWidth}px</span
                >
              </div>

              <div class="flex min-w-0 flex-1 flex-col gap-px">
                {#if editingTrack === trackIdentity(track)}
                  <input
                    class="bg-background text-foreground border-input w-full rounded-sm border px-1 py-px text-xs"
                    bind:value={editName}
                    onblur={() => commitRename(track)}
                    onkeydown={(e) =>
                      e.key === "Enter" && commitRename(track)}
                    use:focusOnMount
                  />
                  {#if !isOkStandardTrackName(editName)}
                    <span class="text-[10px] leading-tight text-yellow-500">
                      Use YYYYMMDD_Callsign
                    </span>
                  {/if}
                {:else}
                  <span
                    class="cursor-text truncate text-xs"
                    ondblclick={() => startRename(track)}
                    title="Double-click to rename">{track.name}</span
                  >
                  {#if !isOkStandardTrackName(track.name)}
                    <span class="text-[10px] leading-tight text-yellow-500">
                      Use YYYYMMDD_Callsign
                    </span>
                  {/if}
                  <span
                    class="text-muted-foreground truncate text-[10px] leading-tight"
                    data-testid="track-stats"
                  >
                    {formatTrackStats(
                      track.distanceKm,
                      track.durationSeconds,
                      track.pointCount,
                    )}
                  </span>
                {/if}
              </div>

              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="flex shrink-0 items-center"
                onclick={(event) => event.stopPropagation()}
              >
                {#if isSelectedTrack(track)}
                  <Tooltip.Root>
                    <Tooltip.Trigger
                      class={"text-muted-foreground hover:text-foreground inline-flex size-6 items-center justify-center rounded-sm"}
                      onclick={() => {
                        simplifyState.set({
                          active: true,
                          layerId: track.layerId,
                          trackId: track.trackId,
                          tolerance: 10,
                          preview: null,
                        });
                      }}
                      aria-label="Simplify track"
                    >
                      <WavesIcon class="size-3.5" />
                    </Tooltip.Trigger>
                    <Tooltip.Content>Simplify track</Tooltip.Content>
                  </Tooltip.Root>
                  <Tooltip.Root>
                    <Tooltip.Trigger
                      class={"text-muted-foreground hover:text-foreground inline-flex size-6 items-center justify-center rounded-sm"}
                      onclick={() => handleExportPlt(track)}
                      aria-label="Export as PLT"
                    >
                      <FileOutputIcon class="size-3.5" />
                    </Tooltip.Trigger>
                    <Tooltip.Content>Export as PLT</Tooltip.Content>
                  </Tooltip.Root>
                {/if}
                <Tooltip.Root>
                  <Tooltip.Trigger
                    class={"text-muted-foreground hover:text-foreground inline-flex size-6 items-center justify-center rounded-sm"}
                    onclick={() => handleToggleVisible(track)}
                    aria-label={track.visible ? "Hide track" : "Show track"}
                  >
                    {#if track.visible}
                      <EyeIcon class="size-3.5" />
                    {:else}
                      <EyeOffIcon class="size-3.5" />
                    {/if}
                  </Tooltip.Trigger>
                  <Tooltip.Content>
                    {track.visible ? "Hide" : "Show"}
                  </Tooltip.Content>
                </Tooltip.Root>
                <Tooltip.Root>
                  <Tooltip.Trigger
                    class={"text-muted-foreground hover:text-foreground inline-flex size-6 items-center justify-center rounded-sm"}
                    onclick={() => handleExport(track)}
                    aria-label="Export to GPX"
                  >
                    <DownloadIcon class="size-3.5" />
                  </Tooltip.Trigger>
                  <Tooltip.Content>Export to GPX</Tooltip.Content>
                </Tooltip.Root>
              </div>
            </div>
          {/each}
      {/if}
    </div>
    <SimplifyPanel />
  </div>
{/if}
