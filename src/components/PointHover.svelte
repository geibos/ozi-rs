<script lang="ts">
  /**
   * The selected track's point under the cursor: its number, time and the
   * leg into it, beside the cursor; a click selects it in the points table.
   * See `$lib/point-hover`. Quiet while a tool owns the cursor.
   */
  import type * as maplibregl from "maplibre-gl";
  import { get } from "svelte/store";
  import { boxSelectActive } from "$lib/box-select";
  import { locale, t } from "$lib/i18n";
  import { nearestPointWithin } from "$lib/point-hover";
  import {
    addWaypointMode,
    drawingModeActive,
    editModeActive,
    measuringActive,
    projectionActive,
    ringActive,
    selectedPointId,
    selectedTrack,
    tracksGeometryVersion,
  } from "$lib/stores";
  import { formatLeg, legsOf, type Leg } from "$lib/track-jumps";
  import { loadTrackDetail } from "$lib/track-points";
  import { formatTimestamp } from "$lib/track-stats";

  interface Props {
    map: maplibregl.Map | null;
  }

  let { map }: Props = $props();
  /** The map as a rune, so the listener effect is subscribed before it checks. */
  const currentMap = $derived(map);

  interface HoverPoint {
    id: number;
    lat: number;
    lon: number;
    timestamp: string | null;
    ordinal: number;
    leg: Leg | null;
  }

  const RADIUS_PX = 10;

  let points: HoverPoint[] = [];
  let total = $state(0);
  let hover = $state<{ x: number; y: number; point: HoverPoint } | null>(null);

  // The selected track's points, kept for the cursor to search.
  $effect(() => {
    const selected = $selectedTrack;
    void $tracksGeometryVersion;
    hover = null;
    points = [];
    total = 0;
    if (!selected) return;
    let cancelled = false;
    void loadTrackDetail(selected.layerId, selected.trackId)
      .then((detail) => {
        if (cancelled) return;
        let ordinal = 0;
        points = detail.segments.flatMap((segment) => {
          const legs = legsOf(segment.points);
          return segment.points.map((p, i) => ({
            id: p.id,
            lat: p.lat,
            lon: p.lon,
            timestamp: p.timestamp,
            ordinal: (ordinal += 1),
            leg: legs[i],
          }));
        });
        total = points.length;
      })
      .catch(() => {
        // The table reports a failed load; a missing hover is no worse.
      });
    return () => {
      cancelled = true;
    };
  });

  function toolActive(): boolean {
    return (
      get(boxSelectActive) ||
      get(editModeActive) ||
      get(drawingModeActive) ||
      get(measuringActive) ||
      get(ringActive) ||
      get(projectionActive) ||
      get(addWaypointMode)
    );
  }

  $effect(() => {
    const m = currentMap;
    if (!m) return;
    let frame = 0;
    let last: maplibregl.MapMouseEvent | null = null;

    const update = () => {
      frame = 0;
      const e = last;
      if (!e || points.length === 0 || toolActive()) {
        if (hover) hover = null;
        return;
      }
      const a = m.unproject([e.point.x - RADIUS_PX, e.point.y - RADIUS_PX]);
      const b = m.unproject([e.point.x + RADIUS_PX, e.point.y + RADIUS_PX]);
      const found = nearestPointWithin(
        points,
        {
          west: Math.min(a.lng, b.lng),
          east: Math.max(a.lng, b.lng),
          south: Math.min(a.lat, b.lat),
          north: Math.max(a.lat, b.lat),
        },
        (lon, lat) => m.project([lon, lat]),
        e.point,
        RADIUS_PX,
      );
      hover = found ? { x: e.point.x, y: e.point.y, point: found } : null;
      m.getCanvas().style.cursor = found ? "pointer" : "";
    };
    const onMove = (e: maplibregl.MapMouseEvent) => {
      last = e;
      if (!frame) frame = requestAnimationFrame(update);
    };
    const onOut = () => {
      last = null;
      hover = null;
    };
    const onClick = () => {
      if (hover && !toolActive()) {
        selectedPointId.set(BigInt(hover.point.id));
      }
    };
    m.on("mousemove", onMove);
    m.on("mouseout", onOut);
    m.on("click", onClick);
    return () => {
      if (frame) cancelAnimationFrame(frame);
      m.off("mousemove", onMove);
      m.off("mouseout", onOut);
      m.off("click", onClick);
    };
  });
</script>

{#if hover}
  <div
    class="bg-popover text-popover-foreground border-border pointer-events-none absolute z-30 rounded-md border px-2 py-1 text-[11px] shadow-md"
    style={`left:${hover.x + 14}px; top:${hover.y + 14}px;`}
    data-testid="point-hover"
  >
    <div class="font-medium">
      {$t("pointHover.point")
        .replace("{i}", String(hover.point.ordinal))
        .replace("{n}", String(total))}
    </div>
    {#if hover.point.timestamp}
      <div class="text-muted-foreground font-mono text-[10px]">
        {formatTimestamp(hover.point.timestamp, $locale)}
      </div>
    {/if}
    {#if hover.point.leg}
      <div class="text-muted-foreground font-mono text-[10px]">
        {formatLeg(hover.point.leg, $locale)}
      </div>
    {/if}
  </div>
{/if}
