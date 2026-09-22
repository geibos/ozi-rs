import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";
import { mayReachNetwork } from "../lib/network-reach";

const layoutSource = readFileSync(
  join(__dirname, "../routes/+layout.svelte"),
  "utf-8",
);
const mapViewSource = readFileSync(
  join(__dirname, "../components/MapView.svelte"),
  "utf-8",
);

/**
 * CJ-2 promises a cold launch in a field camp with no link makes zero network
 * requests. The code made two: the map added an OpenStreetMap raster source
 * unconditionally, and the launch sequence always started the catalogue walk.
 * External review, 2026-09-22.
 */
describe("mayReachNetwork", () => {
  it("stops only on a definite no", () => {
    expect(mayReachNetwork(false)).toBe(false);
  });

  it("proceeds when the machine says it has a link", () => {
    expect(mayReachNetwork(true)).toBe(true);
  });

  it("proceeds when nothing is known — an unknown is not an offline", () => {
    // A `navigator` that does not answer must not disable the catalogue for
    // an operator who does have a link.
    expect(mayReachNetwork(undefined)).toBe(true);
  });
});

describe("a cold launch with no link", () => {
  it("does not start the catalogue walk", () => {
    expect(layoutSource).toContain("mayReachNetworkNow()");
    expect(layoutSource).toContain("OFFLINE_CATALOGUE_REASON");
    // The skip has to leave the loader in a settled state, or the list sits
    // on "refreshing…" for the rest of the session.
    expect(layoutSource).toMatch(
      /mayReachNetworkNow\(\)[\s\S]{0,600}projectsLoading\.set\(false\)/,
    );
  });

  it("does not add the OpenStreetMap basemap", () => {
    expect(mapViewSource).toMatch(
      /mayReachNetworkNow\(\)[\s\S]{0,400}map\.addSource\("osm"/,
    );
  });
});
