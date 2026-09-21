import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

const source = readFileSync(
  join(__dirname, "../components/library/TracksTab.svelte"),
  "utf-8"
);

describe("Library Tracks tab style controls", () => {
  it("reads track line width from GeoJSON properties", () => {
    // The property read itself is exercised in `track-features.test.ts`;
    // here we only confirm the tab still binds the row field.
    expect(source).toContain("t.lineWidth");
  });

  it("uses typed API wrappers for color and line width mutations", () => {
    expect(source).toContain("setTrackColor");
    expect(source).toContain("setTrackLineWidth");
    expect(source).not.toContain("invoke(");
  });

  it("keys and selects tracks by layer and track identity", () => {
    expect(source).toContain("trackKey(t)");
    expect(source).toContain("isSelected(t)");
    expect(source).toContain("{#each tracks as t (trackKey(t))}");
    // Selected state is signalled to the row via the `selected` prop, which
    // LibraryRow translates into the `bg-accent` / `text-accent-foreground`
    // semantic-token utilities — no hand-rolled `.selected` class.
    expect(source).toContain("selected={isSelected(t)}");
  });

  it("renders compact bounded controls without row selection interference", () => {
    expect(source).toContain('type="color"');
    expect(source).toContain("Set line width");
  });
});
