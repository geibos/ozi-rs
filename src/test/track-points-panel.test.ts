import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

const source = readFileSync(
  join(__dirname, "../components/TrackPointsPanel.svelte"),
  "utf-8"
);

describe("TrackPointsPanel timestamp rendering", () => {
  it("renders backend timestamps only when a point has one", () => {
    expect(source).toContain("{#if point.timestamp}");
    expect(source).toContain("{point.timestamp}");
  });

  it("does not include placeholder text for missing timestamps", () => {
    expect(source).not.toContain("No timestamp");
    expect(source).not.toContain("Timestamp unavailable");
    expect(source).not.toContain("timestamp ||");
    expect(source).not.toContain("timestamp ??");
  });
});
