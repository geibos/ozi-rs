import { describe, expect, it } from "vitest";
import { segmentsOf, trackLabelFeatures } from "../lib/track-labels";

/**
 * What the map's symbol layer is given to write names along.
 *
 * MapLibre places, repeats, collides and fades the text; what it cannot know
 * is which track the operator is asking about and how far each crew walked.
 * That is decided here, and handed over as a sort key: lower is placed first.
 */
type Position = [number, number];

function track(
  id: number,
  name: string,
  parts: Position[][],
  extra: Record<string, unknown> = {},
): GeoJSON.Feature {
  return {
    type: "Feature",
    geometry: { type: "MultiLineString", coordinates: parts },
    properties: {
      layer_id: 1,
      track_id: id,
      name,
      color: "#ff0000",
      visible: true,
      ...extra,
    },
  };
}

function collection(...features: GeoJSON.Feature[]): GeoJSON.FeatureCollection {
  return { type: "FeatureCollection", features };
}

const SHORT: Position[] = [
  [30, 60],
  [30.001, 60],
];
const LONG: Position[] = [
  [30, 60],
  [30.05, 60],
];

describe("trackLabelFeatures", () => {
  it("gives every visible named track one label, in its own colour", () => {
    const labels = trackLabelFeatures(
      collection(
        track(1, "ЛИСА15", [LONG], { color: "#00aa00" }),
        track(2, "ЛИСА16", [SHORT]),
      ),
      null,
    );
    expect(labels.features.map((f) => f.properties.name).sort()).toEqual([
      "ЛИСА15",
      "ЛИСА16",
    ]);
    const lisa15 = labels.features.find((f) => f.properties.name === "ЛИСА15");
    expect(lisa15?.properties.color).toBe("#00aa00");
  });

  it("writes no name for a hidden track or a track without one", () => {
    const labels = trackLabelFeatures(
      collection(
        track(1, "скрытый", [LONG], { visible: false }),
        track(2, "   ", [LONG]),
        track(3, "виден", [LONG]),
      ),
      null,
    );
    expect(labels.features.map((f) => f.properties.name)).toEqual(["виден"]);
  });

  it("writes along the segments the crew walked, never across the gap", () => {
    // Two stretches a degree of longitude apart. As one line, the leg
    // between them would be a place to write the name, and nobody walked it.
    const labels = trackLabelFeatures(
      collection(
        track(1, "ЛИСА15", [
          [
            [30, 60],
            [30.01, 60],
          ],
          [
            [31, 60],
            [31.01, 60],
          ],
        ]),
      ),
      null,
    );
    expect(labels.features[0].geometry).toEqual({
      type: "MultiLineString",
      coordinates: [
        [
          [30, 60],
          [30.01, 60],
        ],
        [
          [31, 60],
          [31.01, 60],
        ],
      ],
    });
  });

  it("drops a part that is a single point, which no name can follow", () => {
    const labels = trackLabelFeatures(
      collection(track(1, "точка", [[[30, 60]], LONG])),
      null,
    );
    expect(labels.features[0].geometry.coordinates).toEqual([LONG]);
  });

  it("writes nothing for a track with no line at all", () => {
    const labels = trackLabelFeatures(
      collection(track(1, "точка", [[[30, 60]]])),
      null,
    );
    expect(labels.features).toEqual([]);
  });

  it("gives the longer route the room before the shorter one", () => {
    const labels = trackLabelFeatures(
      collection(track(1, "короткий", [SHORT]), track(2, "длинный", [LONG])),
      null,
    );
    const key = (name: string) =>
      labels.features.find((f) => f.properties.name === name)!.properties
        .sort_key;
    expect(key("длинный")).toBeLessThan(key("короткий"));
  });

  it("ranks by how far the crew walked, not by how often the GPS logged", () => {
    // A navigator logging once a second at a rest stop records a thousand
    // points in a few metres; a long route logged once a minute, two.
    const restStop: Position[] = Array.from({ length: 1000 }, (_, i) => [
      30 + i * 0.0000001,
      60,
    ]);
    const labels = trackLabelFeatures(
      collection(track(1, "привал", [restStop]), track(2, "длинный", [LONG])),
      null,
    );
    const key = (name: string) =>
      labels.features.find((f) => f.properties.name === name)!.properties
        .sort_key;
    expect(key("длинный")).toBeLessThan(key("привал"));
  });

  it("gives the selected track the room first, however short", () => {
    const labels = trackLabelFeatures(
      collection(track(1, "короткий", [SHORT]), track(2, "длинный", [LONG])),
      { layerId: 1n, trackId: 1n },
    );
    const key = (name: string) =>
      labels.features.find((f) => f.properties.name === name)!.properties
        .sort_key;
    expect(key("короткий")).toBeLessThan(key("длинный"));
  });

  it("matches the selection by layer as well as by track", () => {
    const labels = trackLabelFeatures(
      collection(
        track(1, "слой 1", [SHORT], { layer_id: 1 }),
        track(1, "слой 2", [LONG], { layer_id: 2 }),
      ),
      { layerId: 1n, trackId: 1n },
    );
    const key = (name: string) =>
      labels.features.find((f) => f.properties.name === name)!.properties
        .sort_key;
    expect(key("слой 1")).toBeLessThan(key("слой 2"));
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
