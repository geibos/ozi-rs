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
import type { LatLon } from "./geo";

/** A track that wants its name on the map. */
export interface LabelCandidate {
  /** `${layerId}:${trackId}`, the key the marker map uses. */
  key: string;
  name: string;
  /** Rendered colour, so the label reads as belonging to its line. */
  color: string;
  /** The track's positions, in order, flattened across segments. */
  positions: LatLon[];
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
      at: labelAnchor(candidate.positions),
      length: candidate.positions.length,
    }))
    .filter(
      (entry): entry is typeof entry & { at: LatLon } => entry.at !== null,
    )
    .sort((a, b) => {
      const selected =
        Number(b.candidate.selected) - Number(a.candidate.selected);
      if (selected !== 0) return selected;
      return b.length - a.length;
    });

  const kept: PlacedLabel[] = [];
  const keptPoints: { x: number; y: number }[] = [];

  for (const entry of withAnchors) {
    const point = project(entry.at);
    const clash = keptPoints.some(
      (other) =>
        Math.hypot(other.x - point.x, other.y - point.y) < minPixelDistance,
    );
    if (clash) continue;
    keptPoints.push(point);
    kept.push({
      key: entry.candidate.key,
      name: entry.candidate.name,
      color: entry.candidate.color,
      at: entry.at,
    });
  }

  return kept;
}

/**
 * Every position of a GeoJSON geometry, in order, whatever its nesting.
 *
 * A track is a `MultiLineString` — one part per segment — so the flattened
 * order is the order the crew walked, which is what the midpoint is measured
 * along.
 */
export function positionsOf(geometry: unknown): LatLon[] {
  const out: LatLon[] = [];
  const visit = (coords: unknown): void => {
    if (
      Array.isArray(coords) &&
      typeof coords[0] === "number" &&
      typeof coords[1] === "number"
    ) {
      out.push({ lon: coords[0], lat: coords[1] });
      return;
    }
    if (Array.isArray(coords)) for (const child of coords) visit(child);
  };
  // Given a geometry object, walk its coordinates; given coordinates, walk
  // those. Both shapes turn up: the feature carries the former, a caller that
  // has already unwrapped it passes the latter.
  const coordinates =
    geometry && typeof geometry === "object" && "coordinates" in geometry
      ? (geometry as { coordinates: unknown }).coordinates
      : geometry;
  visit(coordinates);
  return out;
}
