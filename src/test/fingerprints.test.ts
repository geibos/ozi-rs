import { describe, expect, it } from "vitest";
import { fingerprintTracks, fingerprintWaypoints } from "../lib/stores";
import type { LayerSummaryDto, TrackSummary } from "../lib/types";

/**
 * Slice-selector pinning tests (`consolidate-state-event-flow`). The
 * `MapView` slice effects rely on these fingerprints being stable when
 * nothing render-relevant has changed and changing when something has —
 * if either property regresses, MapView either skips a needed refresh
 * (stale tracks/markers) or re-fetches on every `state-changed` burst
 * (the original lag bug).
 */

const TRACK_LAYER: LayerSummaryDto = { id: 1, name: "Tracks" };
const TRACK_LAYER_HIDDEN: LayerSummaryDto = {
  id: 1,
  name: "Tracks",
  visible: false,
};
const WAYPOINT_LAYER: LayerSummaryDto = { id: 10, name: "Waypoints" };
const WAYPOINT_LAYER_B: LayerSummaryDto = { id: 20, name: "Aux" };

function track(
  partial: Partial<TrackSummary> & { layer_id: number; track_id: number },
): TrackSummary {
  return {
    name: "T",
    color: "#ff0000",
    line_width: 2,
    visible: true,
    distance_km: 0,
    duration_seconds: null,
    point_count: 10,
    ...partial,
  };
}

describe("fingerprintTracks", () => {
  it("returns a stable string for identical input", () => {
    const a = fingerprintTracks(
      [TRACK_LAYER],
      [track({ layer_id: 1, track_id: 1 })],
    );
    const b = fingerprintTracks(
      [TRACK_LAYER],
      [track({ layer_id: 1, track_id: 1 })],
    );
    expect(a).toBe(b);
  });

  it("is order-independent", () => {
    const a = fingerprintTracks(
      [TRACK_LAYER],
      [
        track({ layer_id: 1, track_id: 1 }),
        track({ layer_id: 1, track_id: 2 }),
      ],
    );
    const b = fingerprintTracks(
      [TRACK_LAYER],
      [
        track({ layer_id: 1, track_id: 2 }),
        track({ layer_id: 1, track_id: 1 }),
      ],
    );
    expect(a).toBe(b);
  });

  it("changes when a track is added", () => {
    const a = fingerprintTracks(
      [TRACK_LAYER],
      [track({ layer_id: 1, track_id: 1 })],
    );
    const b = fingerprintTracks(
      [TRACK_LAYER],
      [
        track({ layer_id: 1, track_id: 1 }),
        track({ layer_id: 1, track_id: 2 }),
      ],
    );
    expect(a).not.toBe(b);
  });

  it("changes when a track's point count changes (drag, simplify)", () => {
    const a = fingerprintTracks(
      [TRACK_LAYER],
      [track({ layer_id: 1, track_id: 1, point_count: 100 })],
    );
    const b = fingerprintTracks(
      [TRACK_LAYER],
      [track({ layer_id: 1, track_id: 1, point_count: 99 })],
    );
    expect(a).not.toBe(b);
  });

  it("changes on color, line width, or visibility flip", () => {
    const base = fingerprintTracks(
      [TRACK_LAYER],
      [track({ layer_id: 1, track_id: 1 })],
    );
    expect(base).not.toBe(
      fingerprintTracks(
        [TRACK_LAYER],
        [track({ layer_id: 1, track_id: 1, color: "#00ff00" })],
      ),
    );
    expect(base).not.toBe(
      fingerprintTracks(
        [TRACK_LAYER],
        [track({ layer_id: 1, track_id: 1, line_width: 4 })],
      ),
    );
    expect(base).not.toBe(
      fingerprintTracks(
        [TRACK_LAYER],
        [track({ layer_id: 1, track_id: 1, visible: false })],
      ),
    );
  });

  it("changes when layer visibility flips", () => {
    const visible = fingerprintTracks([TRACK_LAYER], []);
    const hidden = fingerprintTracks([TRACK_LAYER_HIDDEN], []);
    expect(visible).not.toBe(hidden);
  });

  it("ignores fields that MapView doesn't render against", () => {
    // distance_km and duration_seconds are status-panel concerns, not
    // map-render inputs. Including them in the fingerprint would force a
    // GeoJSON re-fetch every time the status panel refreshes — exactly the
    // kind of waste this change is closing.
    const a = fingerprintTracks(
      [TRACK_LAYER],
      [
        track({
          layer_id: 1,
          track_id: 1,
          distance_km: 1.0,
          duration_seconds: 60,
        }),
      ],
    );
    const b = fingerprintTracks(
      [TRACK_LAYER],
      [
        track({
          layer_id: 1,
          track_id: 1,
          distance_km: 2.5,
          duration_seconds: 120,
        }),
      ],
    );
    expect(a).toBe(b);
  });

  it("is stable across null/undefined sentinels", () => {
    expect(fingerprintTracks(null, null)).toBe(
      fingerprintTracks(undefined, undefined),
    );
    expect(fingerprintTracks([], [])).toBe(fingerprintTracks(null, null));
  });
});

describe("fingerprintWaypoints", () => {
  it("returns a stable string for identical input", () => {
    const a = fingerprintWaypoints([WAYPOINT_LAYER, WAYPOINT_LAYER_B]);
    const b = fingerprintWaypoints([WAYPOINT_LAYER, WAYPOINT_LAYER_B]);
    expect(a).toBe(b);
  });

  it("is order-independent", () => {
    const a = fingerprintWaypoints([WAYPOINT_LAYER, WAYPOINT_LAYER_B]);
    const b = fingerprintWaypoints([WAYPOINT_LAYER_B, WAYPOINT_LAYER]);
    expect(a).toBe(b);
  });

  it("changes when a layer is added or removed", () => {
    const one = fingerprintWaypoints([WAYPOINT_LAYER]);
    const two = fingerprintWaypoints([WAYPOINT_LAYER, WAYPOINT_LAYER_B]);
    expect(one).not.toBe(two);
  });

  it("changes when a layer's visibility flips", () => {
    const visible = fingerprintWaypoints([WAYPOINT_LAYER]);
    const hidden = fingerprintWaypoints([
      { ...WAYPOINT_LAYER, visible: false },
    ]);
    expect(visible).not.toBe(hidden);
  });

  it("ignores the human-readable layer name", () => {
    // Layer renames don't affect marker geometry; the popup text inside a
    // marker reads `wp.name` (per waypoint), not the parent layer's name.
    const a = fingerprintWaypoints([{ id: 10, name: "Old" }]);
    const b = fingerprintWaypoints([{ id: 10, name: "Renamed" }]);
    expect(a).toBe(b);
  });

  it("handles null and empty input symmetrically", () => {
    expect(fingerprintWaypoints(null)).toBe(fingerprintWaypoints([]));
    expect(fingerprintWaypoints(undefined)).toBe(fingerprintWaypoints([]));
  });
});
