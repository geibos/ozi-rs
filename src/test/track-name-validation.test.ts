import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";
import { isOkStandardTrackName } from "../lib/track-names";

const tracksTabSource = readFileSync(
  join(__dirname, "../components/library/TracksTab.svelte"),
  "utf-8",
);

describe("OK-standard track-name validation", () => {
  it("accepts YYYYMMDD_Callsign names with Cyrillic callsigns", () => {
    expect(isOkStandardTrackName("20240601_Иванов")).toBe(true);
  });

  it("accepts a DASH separator — real field files use it (owner's bundle)", () => {
    expect(isOkStandardTrackName("20260709-ЛИСА15")).toBe(true);
  });

  it("rejects a space separator", () => {
    expect(isOkStandardTrackName("20260709 ЛИСА15")).toBe(false);
  });

  it("rejects legacy placeholder names", () => {
    expect(isOkStandardTrackName("Track 1")).toBe(false);
  });

  it("does not enforce real calendar dates", () => {
    expect(isOkStandardTrackName("99999999_Test")).toBe(true);
  });

  it("requires a non-whitespace callsign after the separator", () => {
    expect(isOkStandardTrackName("20240601_")).toBe(false);
    expect(isOkStandardTrackName("20240601_   ")).toBe(false);
    expect(isOkStandardTrackName("20240601-")).toBe(false);
    expect(isOkStandardTrackName("20240601-   ")).toBe(false);
  });
});

describe("Library Tracks tab warning-only validation", () => {
  it("uses the shared track-name helper for warnings", () => {
    expect(tracksTabSource).toContain("isOkStandardTrackName");
    // Warning surface keeps the same Tailwind utility color so the visual
    // contract carries over from the floating panel. The hint text itself
    // is localized via the i18n dictionary (ru: «Формат: ГГГГММДД_Позывной»).
    expect(tracksTabSource).toContain("text-yellow-500");
    expect(tracksTabSource).toContain("tracksTab.nameHint");
  });

  it("keeps rename non-blocking by still calling renameTrack", () => {
    expect(tracksTabSource).toContain("renameTrack");
    expect(tracksTabSource).not.toContain("invoke(");
  });
});
