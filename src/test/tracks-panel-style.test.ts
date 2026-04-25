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

  it("renders compact bounded controls without row selection interference", () => {
    expect(source).toContain('type="color"');
    expect(source).toContain('type="range"');
    expect(source).toContain('min="1"');
    expect(source).toContain('max="12"');
    expect(source).toContain('step="1"');
    expect(source).toContain("stopPropagation");
  });
});
