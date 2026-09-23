import { describe, expect, it } from "vitest";
import { formatLatLon, parseLatLon } from "../lib/coordinates";

/**
 * A place arrives in whatever form the person sending it had to hand. Every
 * string below is one that actually travels between a headquarters and a
 * crew, and they are all the same corner of one search area.
 */
describe("parseLatLon", () => {
  const near = (value: number, expected: number) =>
    expect(Math.abs(value - expected)).toBeLessThan(0.0002);

  it("reads decimal degrees off a phone", () => {
    const at = parseLatLon("59.9311, 30.3609");
    expect(at).not.toBeNull();
    near(at!.lat, 59.9311);
    near(at!.lon, 30.3609);
  });

  it("reads them without the comma", () => {
    const at = parseLatLon("59.9311 30.3609");
    near(at!.lat, 59.9311);
    near(at!.lon, 30.3609);
  });

  it("reads degrees, minutes and seconds off a screenshot", () => {
    const at = parseLatLon("59°55'52\"N 30°21'39\"E");
    near(at!.lat, 59.9311);
    near(at!.lon, 30.3608);
  });

  it("reads degrees and decimal minutes, which is what a navigator shows", () => {
    const at = parseLatLon("N 59 55.87 E 30 21.65");
    near(at!.lat, 59.9311);
    near(at!.lon, 30.3608);
  });

  it("reads the letters after the numbers", () => {
    const at = parseLatLon("59.9311N 30.3609E");
    near(at!.lat, 59.9311);
    near(at!.lon, 30.3609);
  });

  it("reads Russian letters, because a Russian keyboard is what is there", () => {
    const at = parseLatLon("59.9311С 30.3609В");
    near(at!.lat, 59.9311);
    near(at!.lon, 30.3609);
  });

  it("takes longitude first when the letters say so", () => {
    const at = parseLatLon("30.3609E, 59.9311N");
    near(at!.lat, 59.9311);
    near(at!.lon, 30.3609);
  });

  it("reads the southern and western hemispheres", () => {
    const at = parseLatLon("33.9249S 18.4241E");
    near(at!.lat, -33.9249);
    near(at!.lon, 18.4241);
  });

  it("reads a decimal comma when nothing else could be the separator", () => {
    const at = parseLatLon("59,9311 30.3609");
    near(at!.lat, 59.9311);
    near(at!.lon, 30.3609);
  });

  it("refuses one number, which is half a place", () => {
    expect(parseLatLon("59.9311")).toBeNull();
  });

  it("refuses three, because there is no way to know which two", () => {
    expect(parseLatLon("59.9311 30.3609 12.0")).toBeNull();
  });

  it("refuses a latitude off the Earth", () => {
    expect(parseLatLon("95.0, 30.0")).toBeNull();
  });

  it("refuses two letters naming the same axis", () => {
    expect(parseLatLon("59.9311N 30.3609S")).toBeNull();
  });

  it("refuses words", () => {
    expect(parseLatLon("где-то у реки")).toBeNull();
    expect(parseLatLon("")).toBeNull();
  });

  it("refuses sixty minutes, which is the next degree", () => {
    expect(parseLatLon("59 60.0 30 21.65")).toBeNull();
  });
});

describe("formatLatLon", () => {
  it("writes a place back the way it reads one best", () => {
    const text = formatLatLon({ lat: 59.9311, lon: 30.3609 });
    expect(text).toBe("59.931100, 30.360900");
    expect(parseLatLon(text)).toEqual({ lat: 59.9311, lon: 30.3609 });
  });
});
