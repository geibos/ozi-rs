import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";
import { isStandardTrackName } from "../lib/standard-name";

const tracksTabSource = readFileSync(
  join(__dirname, "../components/library/TracksTab.svelte"),
  "utf-8",
);

/**
 * The warning beside a track's name holds it to the standard as written
 * (п. 14–15): the date, `_`, a callsign in latin letters. It used to accept
 * Cyrillic and `-` after the date because field files carry both — which is
 * why they need renaming, not a reason to call them correct. Tightened
 * 2026-10-08, when the warning learned to offer the standard's name.
 */
describe("Library Tracks tab warning-only validation", () => {
  it("flags what the standard does not allow", () => {
    expect(isStandardTrackName("20260709-ЛИСА15")).toBe(false);
    expect(isStandardTrackName("20240601_Иванов")).toBe(false);
    expect(isStandardTrackName("Track 1")).toBe(false);
    expect(isStandardTrackName("20260709_Lisa15")).toBe(true);
  });

  it("uses the shared track-name helper for warnings", () => {
    expect(tracksTabSource).toContain("isStandardTrackName");
    // The warning is a yellow glyph beside the name rather than a line of
    // text under it — spelling the rule out under every row buried the names.
    expect(tracksTabSource).toContain("text-yellow-500");
    expect(tracksTabSource).toContain("tracksTab.nameHint");
  });

  it("keeps rename non-blocking by still calling renameTrack", () => {
    expect(tracksTabSource).toContain("renameTrack");
    expect(tracksTabSource).not.toContain("invoke(");
  });
});
