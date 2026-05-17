import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

const source = readFileSync(
  join(__dirname, "../components/TracksPanel.svelte"),
  "utf-8"
);

describe("TracksPanel style controls", () => {
  it("reads track line width from GeoJSON properties", () => {
    expect(source).toContain("lineWidth");
    expect(source).toContain('f.properties!.line_width');
  });

  it("uses typed API wrappers for color and line width mutations", () => {
    expect(source).toContain("setTrackColor");
    expect(source).toContain("setTrackLineWidth");
    expect(source).not.toContain("invoke(");
  });

  it("keys and selects tracks by layer and track identity", () => {
    expect(source).toContain("trackIdentity(track)");
    expect(source).toContain("isSelectedTrack(track)");
    // Iteration carries `idx` so the migrated panel can place a Separator
    // between rows; the key still discriminates by trackIdentity.
    expect(source).toContain(
      "{#each tracks as track, idx (trackIdentity(track))}"
    );
    // Selected state now lights the semantic-token accent surface via
    // class:bg-accent / class:text-accent-foreground instead of a hand-rolled
    // .selected class.
    expect(source).toContain("class:bg-accent={isSelectedTrack(track)}");
    expect(source).toContain("{#if isSelectedTrack(track)}");
    expect(source).not.toContain(
      "class:selected={$selectedTrack?.trackId === track.trackId}"
    );
    expect(source).not.toContain(
      "{#if $selectedTrack?.trackId === track.trackId}"
    );
  });

  it("renders compact bounded controls without row selection interference", () => {
    expect(source).toContain('type="color"');
    expect(source).toContain('type="range"');
    expect(source).toContain('min="1"');
    expect(source).toContain('max="12"');
    expect(source).toContain('step="1"');
    expect(source).toContain("stopPropagation");
  });
});
