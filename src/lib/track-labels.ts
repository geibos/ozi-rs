/**
 * Where a track's name goes on the map, and which names fit.
 *
 * OziExplorer draws a track's name on the track. This application could not:
 * a MapLibre symbol layer needs SDF glyph PBFs, the style declares no `glyphs`
 * URL, and a remote one does not merely fail to show labels — it poisons the
 * tiling of any source that feeds a symbol layer, so the *lines* stop
 * rendering too, offline. A day of recordings drew in twelve colours with not
 * one name on it, and the question "which line is ЛИСА15" was answered by
 * reading a colour off the list and hunting.
 *
 * Bundling glyphs is a repository-size decision with a font licence attached.
 * Labels as DOM markers need neither, work offline by construction, and reuse
 * the pattern the waypoint markers already use. What they do not get for free
 * is collision avoidance, which a symbol layer would have done — hence the
 * declutter here.
 */
import { pathLengthKm, type LatLon } from "./geo";

/** A track that wants its name on the map. */
export interface LabelCandidate {
  /** `${layerId}:${trackId}`, the key the marker map uses. */
  key: string;
  name: string;
  /** Rendered colour, so the label reads as belonging to its line. */
  color: string;
  /**
   * The track's segments, each a run of positions the crew actually walked.
   *
   * Segments, not one flattened list. A track is split where the recording
   * stopped, and half the *flattened* length can fall in the gap between two
   * stretches a kilometre apart — which is where the name landed, on no line
   * at all. Found by an outside reviewer on 2026-09-23, and visible in the
   * screenshot taken to prove the feature worked.
   */
  segments: LatLon[][];
  /** Selected tracks keep their label when space is short. */
  selected?: boolean;
}

export interface PlacedLabel {
  key: string;
  name: string;
  color: string;
  at: LatLon;
}

/**
 * The point half way along the track, measured by cumulative distance rather
 * than by index.
 *
 * Index-midpoint puts the label wherever the GPS happened to log densely — a
 * crew that stopped for twenty minutes drags the name to the rest stop. Half
 * the length is on the line and near the middle of what the eye reads as the
 * route.
 */
export function labelAnchor(positions: LatLon[]): LatLon | null {
  if (positions.length === 0) return null;
  if (positions.length === 1) return positions[0];

  const legs: number[] = [];
  let total = 0;
  for (let i = 1; i < positions.length; i += 1) {
    const dx = positions[i].lon - positions[i - 1].lon;
    const dy = positions[i].lat - positions[i - 1].lat;
    // Planar length is enough: this picks a point on a line, it does not
    // report a distance to anybody.
    const leg = Math.hypot(dx, dy);
    legs.push(leg);
    total += leg;
  }
  if (total === 0) return positions[0];

  let walked = 0;
  for (let i = 0; i < legs.length; i += 1) {
    if (walked + legs[i] >= total / 2) {
      const remaining = total / 2 - walked;
      const t = legs[i] === 0 ? 0 : remaining / legs[i];
      const from = positions[i];
      const to = positions[i + 1];
      return {
        lat: from.lat + (to.lat - from.lat) * t,
        lon: from.lon + (to.lon - from.lon) * t,
      };
    }
    walked += legs[i];
  }
  return positions[positions.length - 1];
}

/**
 * Where a whole track's name goes: half way along its **longest** segment.
 *
 * Measuring half way along the flattened track puts the name in the gap
 * between two stretches whenever the recording stopped in the middle, and a
 * name floating over empty forest belongs to nothing. The longest segment is
 * the stretch the eye reads as the route, and the midpoint of it is on the
 * line by construction.
 */
export function trackLabelAnchor(segments: LatLon[][]): LatLon | null {
  let best: LatLon[] | null = null;
  let bestLength = -1;
  for (const segment of segments) {
    if (segment.length === 0) continue;
    const length = pathLengthKm(segment);
    if (length > bestLength || best === null) {
      best = segment;
      bestLength = length;
    }
  }
  return best === null ? null : labelAnchor(best);
}

/** How far the crew actually walked, across every segment. */
export function trackLengthKm(segments: LatLon[][]): number {
  return segments.reduce((sum, segment) => sum + pathLengthKm(segment), 0);
}

/**
 * Drop the labels that would land on top of each other.
 *
 * A symbol layer would have done this; DOM markers will happily stack twelve
 * names into an unreadable smear at the zoom where a whole day fits on screen.
 *
 * Order decides who survives: a selected track first — the operator asked
 * about that one — then the longest, because a long route is the one a name
 * helps to follow. `project` turns a position into screen pixels; the caller
 * owns it because only the map knows the current camera.
 */
export function declutter(
  candidates: LabelCandidate[],
  project: (at: LatLon) => { x: number; y: number },
  minPixelDistance = 48,
): PlacedLabel[] {
  const withAnchors = candidates
    .map((candidate) => ({
      candidate,
      at: trackLabelAnchor(candidate.segments),
      // Kilometres walked, not points logged. Ranking by point count let a
      // navigator that logs once a second at a rest stop beat a long route
      // logged once a minute — the density of somebody's GPS deciding whose
      // callsign stays on the map. Found by an outside reviewer, 2026-09-23.
      length: trackLengthKm(candidate.segments),
    }))
    .filter(
      (entry): entry is typeof entry & { at: LatLon } => entry.at !== null,
    )
    .sort((a, b) => {
      // `?? false` is not tidiness. `selected` is optional, `Number(undefined)`
      // is `NaN`, and a comparator that returns `NaN` leaves the order
      // untouched — so this sort did nothing at all whenever no track was
      // selected, and the rule it claims to implement was decided by input
      // order. It passed its own tests by luck.
      const selected =
        Number(b.candidate.selected ?? false) -
        Number(a.candidate.selected ?? false);
      if (selected !== 0) return selected;
      return b.length - a.length;
    });

  const kept: PlacedLabel[] = [];
  const keptBoxes: LabelBox[] = [];

  for (const entry of withAnchors) {
    const point = project(entry.at);
    const box = labelBox(entry.candidate.name, point, minPixelDistance);
    if (keptBoxes.some((other) => boxesOverlap(other, box))) continue;
    keptBoxes.push(box);
    kept.push({
      key: entry.candidate.key,
      name: entry.candidate.name,
      color: entry.candidate.color,
      at: entry.at,
    });
  }

  return kept;
}

interface LabelBox {
  left: number;
  right: number;
  top: number;
  bottom: number;
}

/**
 * Roughly how much screen a name takes, centred on its anchor.
 *
 * Comparing anchor points alone let two long names sit sixty pixels apart and
 * overlap anyway: the test was the distance between their middles, and the
 * text is far wider than that. Measuring the real text would mean a canvas
 * and a font load on every camera move; an estimate from the character count
 * is wrong by a few pixels and right about whether two names collide.
 *
 * `minPixelDistance` becomes the breathing room around the box, so short
 * names keep the spacing they had.
 */
function labelBox(
  name: string,
  at: { x: number; y: number },
  padding: number,
): LabelBox {
  // 11px semibold, mixed Cyrillic and digits: about 6.2px a character, plus
  // the halo.
  const halfWidth = (name.length * 6.2) / 2 + padding / 2;
  const halfHeight = 7 + padding / 2;
  return {
    left: at.x - halfWidth,
    right: at.x + halfWidth,
    top: at.y - halfHeight,
    bottom: at.y + halfHeight,
  };
}

function boxesOverlap(a: LabelBox, b: LabelBox): boolean {
  return (
    a.left < b.right && b.left < a.right && a.top < b.bottom && b.top < a.bottom
  );
}

/**
 * A geometry's segments, each a run of positions the crew actually walked.
 *
 * A track is a `MultiLineString`: one part per segment, and the parts are not
 * joined. Flattening them into one list — which is what this did until an
 * outside reviewer pointed at it on 2026-09-23 — makes the gap between two
 * stretches look like a leg of the route, and the name lands in it.
 */
export function segmentsOf(geometry: unknown): LatLon[][] {
  const coordinates =
    geometry && typeof geometry === "object" && "coordinates" in geometry
      ? (geometry as { coordinates: unknown }).coordinates
      : geometry;

  const isPosition = (value: unknown): value is [number, number] =>
    Array.isArray(value) &&
    typeof value[0] === "number" &&
    typeof value[1] === "number";

  // `Point`, `LineString` and `MultiLineString` all turn up, so the shape is
  // read from the data rather than from a `type` field that a stub might not
  // have set.
  if (isPosition(coordinates)) {
    return [[{ lon: coordinates[0], lat: coordinates[1] }]];
  }
  if (!Array.isArray(coordinates)) return [];
  if (coordinates.every(isPosition)) {
    return [coordinates.map(([lon, lat]) => ({ lon, lat }))];
  }
  return coordinates
    .filter(Array.isArray)
    .map((part) =>
      (part as unknown[])
        .filter(isPosition)
        .map(([lon, lat]) => ({ lon, lat })),
    )
    .filter((segment) => segment.length > 0);
}
