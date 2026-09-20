import type { Map as MapLibreMap } from "maplibre-gl";

const TRACKS_SOURCE = "tracks";
const TRACKS_LAYER = "tracks-lines";
const TRACKS_LAYER_LABELS = "tracks-labels";

export function initTracksLayer(map: MapLibreMap) {
  map.addSource(TRACKS_SOURCE, {
    type: "geojson",
    data: { type: "FeatureCollection", features: [] },
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

  // The on-map name labels are a SYMBOL layer, which MapLibre refuses to add
  // unless the style declares a `glyphs` URL — otherwise `addLayer` THROWS
  // "use of text-field requires a style glyphs property". The app's style has
  // no glyphs (no SDF fonts are bundled yet), so adding this unconditionally
  // aborted `initTracksLayer` and the track line never got wired up — tracks
  // were invisible while their point markers (DOM) still showed. Add the
  // label layer only when glyphs exist, and never let its failure take the
  // line down with it. On-map track labels are a follow-up: bundle SDF glyph
  // PBFs and set `style.glyphs` (also fixes the offline-labels audit finding).
  if (!map.getGlyphs?.()) return;
  try {
    map.addLayer({
      id: TRACKS_LAYER_LABELS,
      type: "symbol",
      source: TRACKS_SOURCE,
      filter: ["==", ["get", "visible"], true],
      layout: {
        "symbol-placement": "line-center",
        "text-field": ["get", "name"],
        "text-size": 11,
        "text-font": ["Open Sans Regular"],
      },
      paint: {
        "text-color": ["get", "color"],
        "text-halo-color": "rgba(0,0,0,0.6)",
        "text-halo-width": 1.5,
      },
    });
  } catch (error) {
    console.warn("track labels unavailable (no glyphs):", error);
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
