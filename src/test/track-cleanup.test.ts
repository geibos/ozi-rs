// CJ-4 "Track cleanup" frontend slice. Source-grep style (convention of
// this test directory): the components are wiring-only around the
// generated bindings, so the tests pin the wiring — api wrappers delegate
// to `commands.*`, the Inspector exposes sort/crop, the segments table
// exposes split/join, and MapView gains click-to-select while edit mode
// stops disabling panning.
import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";
import { get } from "svelte/store";
import { setLocale, t } from "../lib/i18n";

const read = (rel: string) => readFileSync(join(__dirname, rel), "utf-8");

const apiSource = read("../lib/api.ts");
const storesSource = read("../lib/stores.ts");
const mapViewSource = read("../components/MapView.svelte");
const inspectorSource = read("../components/inspector/TrackInspector.svelte");
const segmentsTableSource = read(
  "../components/inspector/TrackSegmentsTable.svelte",
);

describe("api.ts track-cleanup wrappers", () => {
  it("sortTrackPoints delegates to the generated binding with number IDs", () => {
    expect(apiSource).toContain("export async function sortTrackPoints");
    expect(apiSource).toContain(
      "commands.sortTrackPoints(toIdNumber(layerId), toIdNumber(trackId))",
    );
  });

  it("cropTrackToExtent delegates and returns the removed-point count", () => {
    expect(apiSource).toMatch(
      /export async function cropTrackToExtent\([^)]*\): Promise<number>/s,
    );
    expect(apiSource).toContain(
      "commands.cropTrackToExtent(toIdNumber(layerId), toIdNumber(trackId), extent)",
    );
  });

  it("cropTrackToTime delegates with nullable ISO bounds", () => {
    expect(apiSource).toMatch(
      /export async function cropTrackToTime\([^)]*from: string \| null,\s*to: string \| null,?\s*\): Promise<number>/s,
    );
    expect(apiSource).toContain(
      "commands.cropTrackToTime(toIdNumber(layerId), toIdNumber(trackId), from, to)",
    );
  });
});

describe("stores for track cleanup", () => {
  it("exposes mapViewportBounds and tracksGeometryVersion", () => {
    expect(storesSource).toContain("export const mapViewportBounds");
    expect(storesSource).toContain("export const tracksGeometryVersion");
  });
});

describe("TrackInspector sort/crop wiring", () => {
  it("wires the sort action to sortTrackPoints with a success toast", () => {
    expect(inspectorSource).toContain(
      "await sortTrackPoints(sel.layerId, sel.trackId)",
    );
    expect(inspectorSource).toContain(
      'toast.success($t("trackInspector.sortDone"))',
    );
    expect(inspectorSource).toContain('$t("trackInspector.sortByTime")');
  });

  it("crops to the viewport bounds only after a confirm dialog", () => {
    const confirmAt = inspectorSource.indexOf("confirmDialog(");
    const cropAt = inspectorSource.indexOf("await cropTrackToExtent(");
    expect(confirmAt).toBeGreaterThan(-1);
    expect(cropAt).toBeGreaterThan(confirmAt);
    expect(inspectorSource).toContain("min_lat: bounds.minLat");
    expect(inspectorSource).toContain("max_lon: bounds.maxLon");
  });

  it("disables crop-to-view until the map has published viewport bounds", () => {
    expect(inspectorSource).toContain(
      "disabled={!summary || !$mapViewportBounds}",
    );
  });

  it("crop-by-time dialog has two datetime-local inputs and the untimed note", () => {
    const inputs = inspectorSource.match(/type="datetime-local"/g) ?? [];
    expect(inputs).toHaveLength(2);
    expect(inspectorSource).toContain('$t("trackInspector.cropByTimeNote")');
    expect(inspectorSource).toContain(
      "await cropTrackToTime(sel.layerId, sel.trackId, from, to)",
    );
    // Either bound may be empty → null (open bound).
    expect(inspectorSource).toMatch(/if \(!draft\) return null/);
  });

  it("reports the removed-point count and bumps the geometry version", () => {
    expect(inspectorSource).toContain('"trackInspector.pointsRemoved"');
    expect(inspectorSource).toContain(
      "tracksGeometryVersion.update((v) => v + 1)",
    );
    // Detail cache key includes the version so sort/crop refresh the pane.
    expect(inspectorSource).toContain(
      "${sel.layerId}:${sel.trackId}:${$tracksGeometryVersion}",
    );
  });
});

describe("TrackSegmentsTable split/join wiring", () => {
  it("splits the segment at the selected point", () => {
    expect(segmentsTableSource).toMatch(
      /splitSegment\(\s*selected\.layerId,\s*selected\.trackId,\s*BigInt\(segment\.id\),\s*pointId,?\s*\)/,
    );
    expect(segmentsTableSource).toContain('$t("points.splitHere")');
  });

  it("hides the split action for the segment's last point only", () => {
    expect(segmentsTableSource).toMatch(
      /BigInt\(points\[points\.length - 1\]\.id\) === pointId\) return false/,
    );
  });

  it("joins a segment with its previous sibling", () => {
    expect(segmentsTableSource).toMatch(
      /joinSegments\(\s*selected\.layerId,\s*selected\.trackId,\s*BigInt\(previous\.id\),\s*BigInt\(segment\.id\),?\s*\)/,
    );
    expect(segmentsTableSource).toContain('$t("points.joinPrevious")');
    // Join is only offered when a previous sibling exists.
    expect(segmentsTableSource).toContain("{#if segIdx > 0}");
  });

  it("invalidates the cached detail through the geometry version", () => {
    expect(segmentsTableSource).toContain(
      "${selected.layerId}:${selected.trackId}:${$tracksGeometryVersion}",
    );
    expect(segmentsTableSource).toContain(
      "tracksGeometryVersion.update((v) => v + 1)",
    );
  });
});

describe("MapView track cleanup integration", () => {
  it("selects a track by clicking the tracks-lines layer", () => {
    expect(mapViewSource).toContain("function handleMapClickForTrackSelect");
    expect(mapViewSource).toContain("handleMapClickForTrackSelect(e)");
    expect(mapViewSource).toMatch(
      /queryRenderedFeatures\(e\.point, \{\s*layers: \["tracks-lines"\],?\s*\}\)/,
    );
    expect(mapViewSource).toMatch(
      /selectedTrack\.set\(\{\s*layerId: BigInt\(props\.layer_id\),\s*trackId: BigInt\(props\.track_id\),?\s*\}\)/,
    );
  });

  it("skips click-select while drawing / edit / add-waypoint modes own clicks", () => {
    const fn = mapViewSource.slice(
      mapViewSource.indexOf("function handleMapClickForTrackSelect"),
      mapViewSource.indexOf("function handleMapClickForDrawing"),
    );
    expect(fn).toContain(
      "if ($drawingModeActive || $editModeActive || $addWaypointMode) return;",
    );
  });

  it("shows a pointer cursor over track lines", () => {
    expect(mapViewSource).toContain('map.on("mouseenter", "tracks-lines"');
    expect(mapViewSource).toContain('map.on("mouseleave", "tracks-lines"');
  });

  it("no longer disables map panning in edit mode", () => {
    const fn = mapViewSource.slice(
      mapViewSource.indexOf("function applyEditModeMapInteraction"),
      mapViewSource.indexOf("async function refreshTrackGeometry"),
    );
    expect(fn).not.toMatch(/dragPan\.(disable|enable)\(/);
    // Drawing mode still owns dragPan — exactly one disable call remains.
    expect(mapViewSource.match(/dragPan\.disable\(\)/g)).toHaveLength(1);
    // And re-enabling after drawing is unconditional now.
    expect(mapViewSource).not.toMatch(
      /if \(!\$editModeActive\) \{\s*map\.dragPan\.enable/,
    );
  });

  it("publishes viewport bounds on moveend and clears them on teardown", () => {
    expect(mapViewSource).toContain('map.on("moveend", updateViewportBounds)');
    expect(mapViewSource).toMatch(
      /mapViewportBounds\.set\(\{\s*minLat: bounds\.getSouth\(\),\s*minLon: bounds\.getWest\(\),\s*maxLat: bounds\.getNorth\(\),\s*maxLon: bounds\.getEast\(\),?\s*\}\)/,
    );
    expect(mapViewSource).toContain("mapViewportBounds.set(null)");
  });

  it("keys the tracks-slice effect on the geometry version", () => {
    expect(mapViewSource).toContain(
      "${$tracksFingerprint}|${$tracksGeometryVersion}",
    );
  });
});

describe("track cleanup i18n", () => {
  it("has natural strings in both locales for the new actions", () => {
    setLocale("en");
    const en = get(t);
    expect(en("trackInspector.sortByTime")).toBe("Sort points by time");
    expect(en("trackInspector.cropToView")).toBe("Crop to map view");
    expect(en("trackInspector.cropByTime")).toBe("Crop by time range");
    expect(en("trackInspector.pointsRemoved")).toBe("{count} points removed");
    expect(en("points.splitHere")).toBe("Split segment here");
    expect(en("points.joinPrevious")).toBe("Join with previous");

    setLocale("ru");
    const ru = get(t);
    expect(ru("trackInspector.sortByTime")).toBe(
      "Сортировать точки по времени",
    );
    expect(ru("trackInspector.cropToView")).toBe("Обрезать по видимой области");
    expect(ru("trackInspector.cropByTime")).toBe("Обрезать по времени");
    expect(ru("trackInspector.pointsRemoved")).toBe("Удалено точек: {count}");
    expect(ru("trackInspector.cropByTimeNote")).toBe(
      "Точки без времени сохраняются.",
    );
    expect(ru("points.splitHere")).toBe("Разрезать сегмент здесь");
    expect(ru("points.joinPrevious")).toBe("Склеить с предыдущим");
  });
});
