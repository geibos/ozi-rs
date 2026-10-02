import type maplibregl from "maplibre-gl";
import type { Map as MapLibreMap } from "maplibre-gl";
import { LABEL_FONT } from "./glyphs-protocol";

const TRACKS_SOURCE = "tracks";
const TRACKS_LAYER = "tracks-lines";
/** One feature per named track, built by `trackLabelFeatures`. */
const TRACK_LABELS_SOURCE = "track-labels";
const TRACKS_LAYER_LABELS = "tracks-labels";
/** Below this the whole district is on screen and names are a smear. */
const LABEL_MIN_ZOOM = 10;
/** The casing drawn under the selected track. */
export const TRACKS_LAYER_SELECTED = "tracks-line-selected";

/** A filter that matches no feature: nothing is selected. */
const MATCH_NOTHING: maplibregl.FilterSpecification = [
  "==",
  ["get", "track_id"],
  -1,
];

export function initTracksLayer(map: MapLibreMap) {
  map.addSource(TRACKS_SOURCE, {
    type: "geojson",
    data: { type: "FeatureCollection", features: [] },
  });

  // Under the coloured line, not over it: the selected track has to stand out
  // from the other eleven while keeping the colour that says whose it is.
  map.addLayer({
    id: TRACKS_LAYER_SELECTED,
    type: "line",
    source: TRACKS_SOURCE,
    filter: MATCH_NOTHING,
    layout: { "line-join": "round", "line-cap": "round" },
    paint: {
      "line-color": "#ffffff",
      "line-opacity": 0.85,
      "line-width": ["+", ["get", "line_width"], 6],
    },
  });

  // The track LINE is the primary visualization and must always render.
  map.addLayer({
    id: TRACKS_LAYER,
    type: "line",
    source: TRACKS_SOURCE,
    filter: ["==", ["get", "visible"], true],
    layout: {
      "line-join": "round",
      "line-cap": "round",
    },
    paint: {
      "line-color": ["get", "color"],
      "line-width": ["get", "line_width"],
    },
  });

  // The names are a SYMBOL layer, which MapLibre refuses to add unless the
  // style declares a `glyphs` URL — `addLayer` THROWS "use of text-field
  // requires a style glyphs property". The style has carried the shipped
  // glyphs since 2026-10-01, but a style without them must still get its
  // lines: in July this throw aborted `initTracksLayer` and every track was
  // invisible while its point markers showed.
  //
  // Their own source, not the lines': the label geometry is simplified harder
  // (`tolerance`), because text follows a smoothed line far better than a GPS
  // trace that zigzags a metre either way, and the drawn line must not lose
  // a point for the sake of the text.
  if (!map.getGlyphs?.()) return;
  try {
    map.addSource(TRACK_LABELS_SOURCE, {
      type: "geojson",
      data: { type: "FeatureCollection", features: [] },
      tolerance: 2,
    });
    map.addLayer({
      id: TRACKS_LAYER_LABELS,
      type: "symbol",
      source: TRACK_LABELS_SOURCE,
      minzoom: LABEL_MIN_ZOOM,
      layout: {
        // Along the line and repeated, so whatever stretch of a long route is
        // on screen carries its name — one name at the middle is off screen
        // as soon as the operator zooms in on one end.
        "symbol-placement": "line",
        "symbol-spacing": 320,
        "symbol-sort-key": ["get", "sort_key"],
        "text-field": ["get", "name"],
        "text-font": [LABEL_FONT],
        "text-size": 12,
        "text-letter-spacing": 0.02,
        "text-max-angle": 35,
        "text-keep-upright": true,
        // Beside the line rather than on it. Laid over the line, the text
        // hides the stretch it names, and the line shows through between the
        // letters — `_` read as `•` on the first look at the stand.
        "text-offset": [0, -0.9],
        "text-padding": 4,
      },
      paint: {
        "text-color": ["get", "color"],
        // A white halo, not a dark one. The names are drawn in the track's
        // own colour — saturated red, blue, teal — over a topographic map
        // that is mostly pale green and grey. A dark halo under a saturated
        // hue turns it muddy; white separates the letters from the map the
        // way a printed map does.
        "text-halo-color": "rgba(255,255,255,0.95)",
        "text-halo-width": 1.6,
        "text-halo-blur": 0.4,
      },
    });
  } catch (error) {
    console.warn("track labels unavailable:", error);
  }
}

export function updateTracksLayer(
  map: MapLibreMap,
  geojson: GeoJSON.FeatureCollection,
) {
  const source = map.getSource(TRACKS_SOURCE) as
    | maplibregl.GeoJSONSource
    | undefined;
  if (source) {
    source.setData(geojson);
  }
}

/** Hand the symbol layer the names to write; see `trackLabelFeatures`. */
export function updateTrackLabels(
  map: MapLibreMap,
  labels: GeoJSON.FeatureCollection,
) {
  const source = map.getSource(TRACK_LABELS_SOURCE) as
    | maplibregl.GeoJSONSource
    | undefined;
  source?.setData(labels);
}

/**
 * Put the casing under one track, or under none.
 *
 * The filter is swapped rather than the layer removed: its place in the stack
 * — below the coloured line — is what keeps the colour true, and re-adding a
 * layer puts it back on top.
 */
export function highlightTrack(
  map: MapLibreMap,
  selected: { layerId: bigint; trackId: bigint } | null,
): void {
  // A style reload can leave the map without the layer for a moment, and a
  // selection arriving then must not throw at the operator.
  if (!map.getLayer?.(TRACKS_LAYER_SELECTED)) return;
  map.setFilter(
    TRACKS_LAYER_SELECTED,
    selected === null
      ? MATCH_NOTHING
      : ([
          "all",
          ["==", ["get", "layer_id"], Number(selected.layerId)],
          ["==", ["get", "track_id"], Number(selected.trackId)],
        ] as maplibregl.FilterSpecification),
  );
}
