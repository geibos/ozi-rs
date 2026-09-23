import { describe, expect, it } from "vitest";
import { declutter, labelAnchor, segmentsOf } from "../lib/track-labels";
import type { LatLon } from "../lib/geo";

/**
 * A day of recordings drew in twelve colours with not one name on it, and
 * "which line is ЛИСА15" was answered by reading a colour off the list and
 * hunting. MapLibre's symbol layer needs SDF glyphs the application does not
 * bundle, and a remote glyphs URL stops the *lines* rendering offline — so
 * the names are DOM markers, and the collision avoidance a symbol layer would
 * have done is here.
 */
describe("labelAnchor", () => {
  it("puts the name half way along the route by length, not by index", () => {
    // Nineteen points crowded at the start — a crew that stopped for twenty
    // minutes — and one leg that is the rest of the walk. By index the label
    // lands at the rest stop; by length it lands on the walk.
    const positions: LatLon[] = [];
    for (let i = 0; i < 19; i += 1) {
      positions.push({ lat: 59.95, lon: 31.6 + i * 0.00001 });
    }
    positions.push({ lat: 59.95, lon: 31.7 });

    const at = labelAnchor(positions);
    expect(at).not.toBeNull();
    expect(at!.lon).toBeGreaterThan(31.64);
    expect(at!.lon).toBeLessThan(31.66);
  });

  it("answers the only point of a one-point track", () => {
    expect(labelAnchor([{ lat: 59.9, lon: 31.6 }])).toEqual({
      lat: 59.9,
      lon: 31.6,
    });
  });

  it("has nothing to say about a track with no points", () => {
    expect(labelAnchor([])).toBeNull();
  });

  it("survives a track that never moved", () => {
    const still = [
      { lat: 59.9, lon: 31.6 },
      { lat: 59.9, lon: 31.6 },
    ];
    expect(labelAnchor(still)).toEqual({ lat: 59.9, lon: 31.6 });
  });
});

describe("declutter", () => {
  const track = (
    key: string,
    name: string,
    lon: number,
    points = 2,
    selected = false,
  ) => ({
    key,
    name,
    color: "rgba(220,38,38,1)",
    selected,
    segments: [
      Array.from({ length: points }, (_, i) => ({
        lat: 59.95,
        lon: lon + i * 0.001,
      })),
    ],
  });

  /** A projection where one degree of longitude is 1000 px. */
  const project = (at: LatLon) => ({ x: at.lon * 1000, y: at.lat * 1000 });

  it("keeps one name where several would land on top of each other", () => {
    const placed = declutter(
      [track("1:1", "ЛИСА15", 31.6), track("1:2", "ЛИСА16", 31.6005)],
      project,
    );
    expect(placed).toHaveLength(1);
  });

  it("keeps both when they are far enough apart", () => {
    const placed = declutter(
      [track("1:1", "ЛИСА15", 31.6), track("1:2", "ЛИСА16", 31.7)],
      project,
    );
    expect(placed.map((p) => p.name).sort()).toEqual(["ЛИСА15", "ЛИСА16"]);
  });

  it("gives the selected track its name even in a crowd", () => {
    // The operator asked about this one; the others are noise around it.
    const placed = declutter(
      [
        track("1:1", "ЛИСА15", 31.6, 40),
        track("1:2", "ЛИСА16", 31.6001, 2, true),
        track("1:3", "ЛИСА17", 31.6002, 30),
      ],
      project,
    );
    expect(placed).toHaveLength(1);
    expect(placed[0].name).toBe("ЛИСА16");
  });

  it("prefers the longer route when nothing is selected", () => {
    // A long route is the one a name helps to follow.
    const placed = declutter(
      [track("1:1", "короткий", 31.6, 2), track("1:2", "длинный", 31.6001, 50)],
      project,
    );
    expect(placed).toHaveLength(1);
    expect(placed[0].name).toBe("длинный");
  });

  it("says nothing about a track the backend could not draw", () => {
    const placed = declutter(
      [{ key: "1:9", name: "пусто", color: "#fff", segments: [] }],
      project,
    );
    expect(placed).toEqual([]);
  });
});

/**
 * Three defects an outside reviewer found on 2026-09-23, each pinned here
 * before it was fixed. All three were mine, and the first was visible in the
 * screenshot I took to prove the feature worked — the name sat in the gap
 * between two drawn segments and I read it as "the middle of the track".
 */
describe("what the reviewer found", () => {
  const project = (at: LatLon) => ({ x: at.lon * 1000, y: at.lat * 1000 });

  it("puts the name on the line, not in the gap between two segments", () => {
    // A track split at a break: the crew walked one stretch, stopped
    // recording, and walked another a kilometre away. Half the *flattened*
    // length falls in the gap, where there is nothing to label.
    const placed = declutter(
      [
        {
          key: "1:1",
          name: "ЛИСА15",
          color: "#f00",
          segments: [
            [
              { lat: 60, lon: 30 },
              { lat: 60, lon: 30.01 },
            ],
            [
              { lat: 60, lon: 31 },
              { lat: 60, lon: 31.01 },
            ],
          ],
        },
      ],
      project,
    );
    expect(placed).toHaveLength(1);
    const lon = placed[0].at.lon;
    const onASegment =
      (lon >= 30 && lon <= 30.01) || (lon >= 31 && lon <= 31.01);
    expect(onASegment, `label landed at ${lon}, which is on no segment`).toBe(
      true,
    );
  });

  it("ranks by how far the crew walked, not by how often the GPS logged", () => {
    // A navigator logging once a second at a rest stop beats a long route
    // logged once a minute, if the count is what decides.
    const restStop = {
      key: "1:1",
      name: "привал",
      color: "#f00",
      segments: [
        Array.from({ length: 1000 }, (_, i) => ({
          lat: 60,
          lon: 30 + i * 0.0000001,
        })),
      ],
    };
    const longRoute = {
      key: "1:2",
      name: "длинный",
      color: "#00f",
      segments: [
        [
          { lat: 60, lon: 30.0001 },
          { lat: 60, lon: 30.05 },
        ],
      ],
    };
    const placed = declutter([restStop, longRoute], project, 100000);
    expect(placed).toHaveLength(1);
    expect(placed[0].name).toBe("длинный");
  });

  it("keeps two long names from overlapping, not just their centres", () => {
    // Centres 60 px apart clears a 48 px centre-to-centre test and still
    // overlaps, because the text is far wider than the gap.
    const wide = (key: string, name: string, lon: number) => ({
      key,
      name,
      color: "#f00",
      segments: [
        [
          { lat: 60, lon },
          { lat: 60, lon: lon + 0.00001 },
        ],
      ],
    });
    const placed = declutter(
      [
        wide("1:1", "20260708_Ветер2_первая_группа", 30),
        wide("1:2", "20260708_Ветер3_вторая_группа", 30.06),
      ],
      project,
    );
    expect(placed).toHaveLength(1);
  });
});

describe("segmentsOf", () => {
  it("keeps a multi-segment track's parts apart", () => {
    const geometry = {
      type: "MultiLineString",
      coordinates: [
        [
          [31.6, 59.95],
          [31.61, 59.951],
        ],
        [
          [31.62, 59.952],
          [31.63, 59.953],
        ],
      ],
    };
    // Two parts, not one list: the gap between them is not a leg of the walk.
    expect(segmentsOf(geometry)).toEqual([
      [
        { lon: 31.6, lat: 59.95 },
        { lon: 31.61, lat: 59.951 },
      ],
      [
        { lon: 31.62, lat: 59.952 },
        { lon: 31.63, lat: 59.953 },
      ],
    ]);
  });

  it("reads a single-part line as one segment", () => {
    expect(
      segmentsOf({
        type: "LineString",
        coordinates: [
          [31.6, 59.95],
          [31.61, 59.951],
        ],
      }),
    ).toEqual([
      [
        { lon: 31.6, lat: 59.95 },
        { lon: 31.61, lat: 59.951 },
      ],
    ]);
  });
});
