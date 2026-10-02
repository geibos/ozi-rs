/**
 * What the map writes track names along.
 *
 * OziExplorer draws a track's name on the track. Until 2026-10-01 this
 * application could not: a MapLibre symbol layer needs SDF glyphs, none were
 * bundled, and a remote glyphs URL took the lines down with the labels
 * whenever the laptop was offline. The names were DOM markers then, one per
 * track at the middle of its longest segment, decluttered by hand — and off
 * screen as soon as the operator zoomed in on one end of a long route.
 *
 * The glyphs ship now (`maplibre/glyphs-protocol.ts`), so the map writes the
 * names itself: along the line, repeated, collided and faded the way a map
 * should. What it cannot know is which track the operator is asking about and
 * how far each crew walked, and that is what this module decides.
 */
import { pathLengthKm, type LatLon } from "./geo";

/** What the symbol layer reads off each label feature. */
export interface TrackLabelProperties {
  name: string;
  /** Rendered colour, so the name reads as belonging to its line. */
  color: string;
  /**
   * Lower is placed first. The selected track before any other — the
   * operator asked about that one — and then by distance walked, because a
   * long route is the one a name helps to follow.
   */
  sort_key: number;
}

export type TrackLabelCollection = GeoJSON.FeatureCollection<
  GeoJSON.MultiLineString,
  TrackLabelProperties
>;

/** How far the crew actually walked, across every segment. */
export function trackLengthKm(segments: LatLon[][]): number {
  return segments.reduce((sum, segment) => sum + pathLengthKm(segment), 0);
}

/**
 * One label feature per visible, named track, drawn from its geometry.
 *
 * The geometry is the track's segments and nothing between them: a name
 * written along the straight line across a recording gap sits over forest
 * nobody walked. A part with fewer than two points has no line to follow and
 * is left out.
 *
 * Ranked by kilometres walked, not points logged. Ranking by point count let
 * a navigator logging once a second at a rest stop beat a long route logged
 * once a minute — the density of somebody's GPS deciding whose callsign stays
 * on the map. Found by an outside reviewer, 2026-09-23.
 */
export function trackLabelFeatures(
  tracks: GeoJSON.FeatureCollection,
  selected: { layerId: bigint; trackId: bigint } | null,
): TrackLabelCollection {
  const candidates: Array<{
    name: string;
    color: string;
    segments: LatLon[][];
    isSelected: boolean;
    length: number;
  }> = [];

  for (const feature of tracks.features) {
    const properties = feature.properties ?? {};
    if (properties.visible === false) continue;
    const name = typeof properties.name === "string" ? properties.name : "";
    if (name.trim().length === 0) continue;
    const segments = segmentsOf(feature.geometry).filter(
      (segment) => segment.length >= 2,
    );
    if (segments.length === 0) continue;
    candidates.push({
      name,
      color:
        typeof properties.color === "string"
          ? properties.color
          : "rgba(255,255,255,1)",
      segments,
      isSelected:
        selected !== null &&
        String(selected.layerId) === String(properties.layer_id) &&
        String(selected.trackId) === String(properties.track_id),
      length: trackLengthKm(segments),
    });
  }

  candidates.sort((a, b) => {
    if (a.isSelected !== b.isSelected) return a.isSelected ? -1 : 1;
    return b.length - a.length;
  });

  return {
    type: "FeatureCollection",
    features: candidates.map((candidate, rank) => ({
      type: "Feature",
      geometry: {
        type: "MultiLineString",
        coordinates: candidate.segments.map((segment) =>
          segment.map(({ lon, lat }) => [lon, lat]),
        ),
      },
      properties: {
        name: candidate.name,
        color: candidate.color,
        sort_key: rank,
      },
    })),
  };
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
