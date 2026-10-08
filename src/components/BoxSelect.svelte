<script lang="ts">
  /**
   * The box tool on the map: drag a rectangle over the selected track, and the
   * points inside it are chosen; Shift adds a second box to the first. Then
   * delete the chosen points, or keep only them — one undo step either way.
   *
   * OziExplorer's Selection Control, which the owner reached for on every
   * track of a real search (2026-10-08, `docs/field-notes/`). Kept out of
   * MapView, which only mounts it: the tool owns its drag, its layer and its
   * bar, and goes away whole when it is off.
   */
  import type * as maplibregl from "maplibre-gl";
  import { get } from "svelte/store";
  import { toast } from "svelte-sonner";
  import { Button } from "$lib/components/ui/button";
  import { removeTrackPoints } from "$lib/api";
  import {
    boxBetween,
    boxSelectActive,
    boxSelection,
    combineSelection,
    pointsInBox,
    setBoxSelect,
    type ScreenPoint,
  } from "$lib/box-select";
  import { reportEditFailure } from "$lib/edit-failure";
  import { isEditableTarget } from "$lib/editable-target";
  import { t } from "$lib/i18n";
  import { selectedTrack, tracksGeometryVersion } from "$lib/stores";
  import { loadTrackDetail } from "$lib/track-points";

  interface Props {
    map: maplibregl.Map | null;
  }

  let { map }: Props = $props();

  const SOURCE = "box-selection";
  const LAYER = "box-selection-points";

  let start = $state<ScreenPoint | null>(null);
  let current = $state<ScreenPoint | null>(null);
  /** Coordinates of the chosen points, for drawing them. */
  let chosen = $state<{ lat: number; lon: number }[]>([]);

  const box = $derived(start && current ? boxBetween(start, current) : null);
  const count = $derived($boxSelection?.ids.size ?? 0);

  function ensureLayer(m: maplibregl.Map) {
    if (m.getSource(SOURCE)) return;
    m.addSource(SOURCE, {
      type: "geojson",
      data: { type: "FeatureCollection", features: [] },
    });
    m.addLayer({
      id: LAYER,
      type: "circle",
      source: SOURCE,
      paint: {
        "circle-radius": 5,
        "circle-color": "#ffffff",
        "circle-stroke-color": "#dc2626",
        "circle-stroke-width": 2,
      },
    });
  }

  function drawChosen() {
    const m = map;
    if (!m || !m.isStyleLoaded()) return;
    ensureLayer(m);
    const source = m.getSource(SOURCE) as maplibregl.GeoJSONSource | undefined;
    source?.setData({
      type: "FeatureCollection",
      features: chosen.map((p) => ({
        type: "Feature",
        geometry: { type: "Point", coordinates: [p.lon, p.lat] },
        properties: {},
      })),
    });
  }

  $effect(() => {
    void chosen;
    drawChosen();
  });

  // The selection belongs to one track: choosing another track, or an edit
  // that changed this one's points, leaves ids that no longer mean anything.
  $effect(() => {
    const selected = $selectedTrack;
    void $tracksGeometryVersion;
    const sel = get(boxSelection);
    if (
      sel &&
      (selected?.layerId !== sel.layerId || selected?.trackId !== sel.trackId)
    ) {
      boxSelection.set(null);
    }
  });

  $effect(() => {
    if (!$boxSelection) chosen = [];
  });

  // While the tool is on, a drag draws a box instead of moving the map.
  $effect(() => {
    const m = map;
    const active = $boxSelectActive;
    if (!m) return;
    if (!active) {
      start = null;
      current = null;
      return;
    }
    m.dragPan.disable();
    m.boxZoom.disable();
    const canvas = m.getCanvas();
    canvas.style.cursor = "crosshair";

    const onDown = (e: maplibregl.MapMouseEvent) => {
      if (e.originalEvent.button !== 0) return;
      start = { x: e.point.x, y: e.point.y };
      current = start;
    };
    const onMove = (e: maplibregl.MapMouseEvent) => {
      if (start) current = { x: e.point.x, y: e.point.y };
    };
    const onUp = (e: maplibregl.MapMouseEvent) => {
      if (!start) return;
      const drawn = boxBetween(start, { x: e.point.x, y: e.point.y });
      start = null;
      current = null;
      void choose(drawn, e.originalEvent.shiftKey);
    };
    m.on("mousedown", onDown);
    m.on("mousemove", onMove);
    m.on("mouseup", onUp);
    return () => {
      m.off("mousedown", onDown);
      m.off("mousemove", onMove);
      m.off("mouseup", onUp);
      m.dragPan.enable();
      m.boxZoom.enable();
      canvas.style.cursor = "";
    };
  });

  async function choose(
    drawn: ReturnType<typeof boxBetween>,
    additive: boolean,
  ) {
    const m = map;
    const track = get(selectedTrack);
    if (!m || !track) return;
    // A click is not a box: it clears, as clicking empty space does anywhere.
    if (drawn.right - drawn.left < 4 && drawn.bottom - drawn.top < 4) {
      if (!additive) boxSelection.set(null);
      return;
    }
    const detail = await loadTrackDetail(track.layerId, track.trackId);
    const points = detail.segments.flatMap((s) => s.points);
    const inside = pointsInBox(
      points,
      (lon, lat) => m.project([lon, lat]),
      drawn,
    );
    const previous = get(boxSelection);
    const sameTrack =
      previous?.layerId === track.layerId &&
      previous?.trackId === track.trackId;
    const ids = combineSelection(
      sameTrack ? previous.ids : new Set(),
      inside,
      additive && sameTrack,
    );
    boxSelection.set(
      ids.size > 0
        ? { layerId: track.layerId, trackId: track.trackId, ids }
        : null,
    );
    chosen = points.filter((p) => ids.has(p.id));
  }

  async function remove(keepOnly: boolean) {
    const sel = get(boxSelection);
    if (!sel || sel.ids.size === 0) return;
    try {
      const removed = await removeTrackPoints(
        sel.layerId,
        sel.trackId,
        [...sel.ids].map(BigInt),
        keepOnly,
      );
      boxSelection.set(null);
      tracksGeometryVersion.update((v) => v + 1);
      toast.success($t("boxSelect.removed").replace("{n}", String(removed)));
    } catch (error) {
      reportEditFailure("boxSelect.failed", error);
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (!get(boxSelectActive)) return;
    if (e.key === "Escape") {
      e.preventDefault();
      setBoxSelect(false);
    } else if (
      (e.key === "Delete" || e.key === "Backspace") &&
      !isEditableTarget(e.target) &&
      get(boxSelection)
    ) {
      e.preventDefault();
      void remove(false);
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

{#if $boxSelectActive}
  {#if box}
    <div
      class="pointer-events-none absolute z-30 border-2 border-dashed border-red-600 bg-red-600/10"
      style={`left:${box.left}px; top:${box.top}px; width:${box.right - box.left}px; height:${box.bottom - box.top}px;`}
      data-testid="box-select-rectangle"
    ></div>
  {/if}
  <div
    class="bg-popover text-popover-foreground border-border absolute top-3 left-1/2 z-40 flex -translate-x-1/2 items-center gap-2 rounded-md border px-3 py-1.5 text-xs shadow-lg"
    data-testid="box-select-bar"
  >
    {#if !$selectedTrack}
      <span>{$t("boxSelect.chooseTrack")}</span>
    {:else if count === 0}
      <span class="whitespace-nowrap">{$t("boxSelect.hint")}</span>
    {:else}
      <span
        class="whitespace-nowrap tabular-nums"
        data-testid="box-select-count"
        >{$t("boxSelect.chosen").replace("{n}", String(count))}</span
      >
      <Button
        size="xs"
        variant="destructive"
        data-testid="box-select-delete"
        onclick={() => remove(false)}>{$t("boxSelect.delete")}</Button
      >
      <Button
        size="xs"
        variant="outline"
        data-testid="box-select-keep"
        onclick={() => remove(true)}>{$t("boxSelect.keepOnly")}</Button
      >
      <Button size="xs" variant="ghost" onclick={() => boxSelection.set(null)}
        >{$t("boxSelect.clear")}</Button
      >
    {/if}
    <Button
      size="xs"
      variant="ghost"
      data-testid="box-select-done"
      onclick={() => setBoxSelect(false)}>{$t("boxSelect.done")}</Button
    >
  </div>
{/if}
