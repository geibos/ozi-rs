// @vitest-environment jsdom
/**
 * The inspector's list of jumps and outliers: choosing an entry takes the
 * selection and the map to its point, and each kind offers its own edits —
 * a jump is split, an outlier's apex is deleted, or deleted with a split.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { get } from "svelte/store";
import { tick } from "svelte";

const { cutOutTrackPoint, deleteTrackPoint, splitSegment, trimTrackAtPoint } =
  vi.hoisted(() => ({
    cutOutTrackPoint: vi.fn(async () => {}),
    deleteTrackPoint: vi.fn(async () => {}),
    splitSegment: vi.fn(async () => {}),
    trimTrackAtPoint: vi.fn(async () => 5),
  }));

vi.mock("$lib/api", () => ({
  cutOutTrackPoint,
  deleteTrackPoint,
  splitSegment,
  trimTrackAtPoint,
}));

import TrackJumps from "../components/inspector/TrackJumps.svelte";
import {
  mapFocusRequest,
  selectedPointId,
  tracksGeometryVersion,
} from "../lib/stores";
import { setLocale } from "../lib/i18n";
import type { TrackDetail } from "../lib/types";

const M_PER_DEG_LAT = 111_195;

/**
 * Forty points 10 m apart going north; point 11 sits 400 m east and back (an
 * outlier), and from point 31 on the track resumes 900 m further (a jump).
 */
function dirtyTrack(): TrackDetail {
  const start = Date.parse("2026-10-06T09:00:00Z");
  const points = Array.from({ length: 40 }, (_, i) => ({
    id: i + 1,
    lat: 59.9 + (i * 10 + (i >= 30 ? 900 : 0)) / M_PER_DEG_LAT,
    lon: 30.3 + (i === 10 ? 400 / (M_PER_DEG_LAT * 0.5) : 0),
    elevation: null,
    timestamp: new Date(start + i * 5000).toISOString(),
  }));
  return { id: 7, name: "20261006_Lisa1", segments: [{ id: 3, points }] };
}

describe("the jumps and outliers list", () => {
  beforeEach(async () => {
    setLocale("ru");
    selectedPointId.set(null);
    vi.clearAllMocks();
    render(TrackJumps, {
      props: { detail: dirtyTrack(), layerId: 5n, trackId: 7n },
    });
    await tick();
  });

  afterEach(() => cleanup());

  it("lists the outlier and the jump in track order", () => {
    const entries = screen.getAllByTestId("track-jump");
    expect(entries.map((e) => e.dataset.kind)).toEqual(["outlier", "jump"]);
    expect(entries[0].textContent).toContain("Выброс");
    expect(entries[0].textContent).toContain("точка 11");
    expect(entries[1].textContent).toContain("точка 31");
    expect(screen.getByTestId("track-jumps-count").textContent).toBe("2");
  });

  it("takes the selection and the map to the chosen point", async () => {
    await fireEvent.click(screen.getAllByTestId("track-jump")[1]);
    expect(get(selectedPointId)).toBe(31n);
    expect(get(mapFocusRequest)).toMatchObject({ kind: "waypoint" });
  });

  it("splits a jump between its two points", async () => {
    await fireEvent.click(screen.getAllByTestId("track-jump")[1]);
    await tick();
    const version = get(tracksGeometryVersion);
    await fireEvent.click(screen.getByTestId("jump-split"));
    await tick();
    expect(splitSegment).toHaveBeenCalledWith(5n, 7n, 3n, 30n);
    expect(get(tracksGeometryVersion)).toBe(version + 1);
    expect(screen.queryByTestId("jump-cut-out")).toBeNull();
  });

  it("deletes an outlier's apex, or deletes it and splits there", async () => {
    await fireEvent.click(screen.getAllByTestId("track-jump")[0]);
    await tick();
    expect(screen.queryByTestId("jump-split")).toBeNull();
    await fireEvent.click(screen.getByTestId("jump-cut-out"));
    await tick();
    expect(cutOutTrackPoint).toHaveBeenCalledWith(5n, 7n, 3n, 11n);
    expect(get(selectedPointId)).toBeNull();

    await fireEvent.click(screen.getAllByTestId("track-jump")[0]);
    await tick();
    await fireEvent.click(screen.getByTestId("jump-delete-apex"));
    await tick();
    expect(deleteTrackPoint).toHaveBeenCalledWith(5n, 7n, 3n, 11n);
  });
});

describe("a break in the list", () => {
  const brokenTrack = (): TrackDetail => {
    const start = Date.parse("2026-09-26T09:00:00Z");
    const later = Date.parse("2026-10-08T06:00:00Z");
    const points = Array.from({ length: 20 }, (_, i) => ({
      id: i + 1,
      lat: 59.9 + (i * 10) / 111_195,
      lon: 30.3,
      elevation: null,
      timestamp: new Date((i < 5 ? start : later) + i * 5000).toISOString(),
    }));
    return { id: 7, name: "20261008_Lisa15", segments: [{ id: 3, points }] };
  };

  beforeEach(async () => {
    setLocale("ru");
    selectedPointId.set(null);
    vi.clearAllMocks();
    render(TrackJumps, {
      props: { detail: brokenTrack(), layerId: 5n, trackId: 7n },
    });
    await tick();
  });

  afterEach(() => cleanup());

  it("says how long the break was, and cuts either side of it", async () => {
    const entry = screen.getByTestId("track-jump");
    expect(entry.dataset.kind).toBe("break");
    expect(entry.textContent).toContain("Перерыв");
    await fireEvent.click(entry);
    await tick();
    await fireEvent.click(screen.getByTestId("break-trim-before"));
    await tick();
    expect(trimTrackAtPoint).toHaveBeenCalledWith(5n, 7n, 6n, true);
    await fireEvent.click(screen.getByTestId("track-jump"));
    await tick();
    await fireEvent.click(screen.getByTestId("break-trim-after"));
    await tick();
    expect(trimTrackAtPoint).toHaveBeenLastCalledWith(5n, 7n, 5n, false);
  });
});
