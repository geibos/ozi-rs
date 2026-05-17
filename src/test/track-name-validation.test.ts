import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";
import { isOkStandardTrackName } from "../lib/track-names";

const tracksPanelSource = readFileSync(
  join(__dirname, "../components/TracksPanel.svelte"),
  "utf-8"
);

describe("OK-standard track-name validation", () => {
  it("accepts YYYYMMDD_Callsign names with Cyrillic callsigns", () => {
    expect(isOkStandardTrackName("20240601_Иванов")).toBe(true);
  });

  it("rejects legacy placeholder names", () => {
    expect(isOkStandardTrackName("Track 1")).toBe(false);
  });

  it("does not enforce real calendar dates", () => {
    expect(isOkStandardTrackName("99999999_Test")).toBe(true);
  });

  it("requires a non-whitespace callsign after the underscore", () => {
    expect(isOkStandardTrackName("20240601_")).toBe(false);
    expect(isOkStandardTrackName("20240601_   ")).toBe(false);
  });
});

describe("TracksPanel warning-only validation", () => {
  it("uses the shared track-name helper for warnings", () => {
    expect(tracksPanelSource).toContain("isOkStandardTrackName");
    // The legacy `.ok-name-warning` CSS class is gone; the warning surface
    // now uses a Tailwind warning colour utility instead (`text-yellow-500`).
    expect(tracksPanelSource).toContain("text-yellow-500");
    expect(tracksPanelSource).toContain("Use YYYYMMDD_Callsign");
  });

  it("keeps rename non-blocking by still calling renameTrack", () => {
    expect(tracksPanelSource).toContain("await renameTrack");
    expect(tracksPanelSource).not.toContain("invoke(");
  });
});
