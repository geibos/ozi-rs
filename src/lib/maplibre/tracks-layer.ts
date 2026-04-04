import type { Map as MapLibreMap } from "maplibre-gl";

const TRACKS_SOURCE = "tracks";
const TRACKS_LAYER = "tracks-lines";
const TRACKS_LAYER_LABELS = "tracks-labels";

export function initTracksLayer(map: MapLibreMap) {
  map.addSource(TRACKS_SOURCE, {
    type: "geojson",
    data: { type: "FeatureCollection", features: [] },
  });

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
}

export function updateTracksLayer(
  map: MapLibreMap,
  geojson: GeoJSON.FeatureCollection
) {
  const source = map.getSource(TRACKS_SOURCE) as
    | maplibregl.GeoJSONSource
    | undefined;
  if (source) {
    source.setData(geojson);
  }
}
