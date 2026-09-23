/**
 * The coordinate grid as a map layer.
 *
 * Which lines to draw and what to call them is `$lib/graticule`; this puts the
 * answer on the canvas and keeps it there as the camera moves. The lines are a
 * GeoJSON source, the labels are DOM markers — the style bundles no glyphs, so
 * a symbol layer would throw, the same reason the track names are markers.
 *
 * The grid sits under everything a coordinator draws: it is the paper, not the
 * work.
 */
import maplibregl from "maplibre-gl";
import { graticule, graticuleGeoJson, type GridLine } from "$lib/graticule";

const SOURCE_ID = "graticule";
const LINE_LAYER_ID = "graticule-lines";

/** Where the grid goes in the layer order: above the raster, below the work. */
const BENEATH_LAYERS = ["tracks-lines", "tracks-lines-casing", "measure-line"];

export interface GraticuleHandle {
  /** Recompute for the camera as it is now. */
  refresh(): void;
  /** Take the grid off the map. */
  detach(): void;
}

function firstExistingLayer(map: maplibregl.Map, ids: string[]): string | undefined {
  for (const id of ids) {
    if (map.getLayer(id)) return id;
  }
  return undefined;
}

/**
 * Draw the grid on `map` and keep it up to date.
 *
 * Answers a handle rather than nothing: the caller turns the grid off, and a
 * layer that can only be added is a layer that leaks.
 */
export function attachGraticule(map: maplibregl.Map): GraticuleHandle {
  const labels: maplibregl.Marker[] = [];

  const clearLabels = () => {
    for (const marker of labels) marker.remove();
    labels.length = 0;
  };

  const refresh = () => {
    const bounds = map.getBounds();
    const lines = graticule({
      south: bounds.getSouth(),
      west: bounds.getWest(),
      north: bounds.getNorth(),
      east: bounds.getEast(),
    });

    const data = graticuleGeoJson(lines);
    const source = map.getSource(SOURCE_ID) as
      | maplibregl.GeoJSONSource
      | undefined;
    if (source) {
      source.setData(data);
    } else {
      map.addSource(SOURCE_ID, { type: "geojson", data });
      map.addLayer(
        {
          id: LINE_LAYER_ID,
          type: "line",
          source: SOURCE_ID,
          paint: {
            // Thin and pale: a grid that competes with a track is a grid in
            // the way. Dark enough to read over a satellite raster.
            "line-color": "#1f2937",
            "line-width": 0.7,
            "line-opacity": 0.45,
          },
        },
        firstExistingLayer(map, BENEATH_LAYERS),
      );
    }

    placeLabels(map, lines, labels, clearLabels);
  };

  refresh();
  map.on("moveend", refresh);
  map.on("idle", refresh);

  return {
    refresh,
    detach() {
      map.off("moveend", refresh);
      map.off("idle", refresh);
      clearLabels();
      if (map.getLayer(LINE_LAYER_ID)) map.removeLayer(LINE_LAYER_ID);
      if (map.getSource(SOURCE_ID)) map.removeSource(SOURCE_ID);
    },
  };
}

/**
 * One label per line, against the edge the line runs off.
 *
 * Parallels are named down the left edge and meridians along the top, which is
 * where a map reader looks for them and keeps the middle of the map clear.
 */
function placeLabels(
  map: maplibregl.Map,
  lines: GridLine[],
  labels: maplibregl.Marker[],
  clearLabels: () => void,
): void {
  clearLabels();
  const bounds = map.getBounds();
  // A little inside the edge, so the text is not half off the canvas.
  const lonAtLeft = bounds.getWest() + (bounds.getEast() - bounds.getWest()) * 0.02;
  const latAtTop = bounds.getNorth() - (bounds.getNorth() - bounds.getSouth()) * 0.02;

  for (const line of lines) {
    const at =
      line.kind === "lat"
        ? { lat: line.degrees, lon: lonAtLeft }
        : { lat: latAtTop, lon: line.degrees };
    if (
      at.lat < bounds.getSouth() ||
      at.lat > bounds.getNorth() ||
      at.lon < bounds.getWest() ||
      at.lon > bounds.getEast()
    ) {
      continue;
    }

    const element = document.createElement("div");
    element.className = "graticule-label";
    element.textContent = line.label;
    element.setAttribute("data-testid", "graticule-label");
    element.style.cssText = [
      "font: 10px/1.2 ui-monospace, SFMono-Regular, Menlo, monospace",
      "color: #111827",
      "background: rgba(255,255,255,0.72)",
      "padding: 0 3px",
      "border-radius: 2px",
      "pointer-events: none",
      "white-space: nowrap",
    ].join(";");

    labels.push(
      new maplibregl.Marker({
        element,
        anchor: line.kind === "lat" ? "left" : "top",
      })
        .setLngLat([at.lon, at.lat])
        .addTo(map),
    );
  }
}
