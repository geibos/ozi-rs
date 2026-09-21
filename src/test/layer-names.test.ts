import { describe, expect, it } from "vitest";
import { layerDisplayName } from "../lib/layer-names";

describe("layerDisplayName", () => {
  it("shows the core's default layers in the interface language", () => {
    // The core names them in English and they are part of the saved file;
    // the selector of a Russian window should not say "Waypoints".
    expect(layerDisplayName("Tracks", "ru")).toBe("Треки");
    expect(layerDisplayName("Waypoints", "ru")).toBe("Точки");
    expect(layerDisplayName("Tracks", "en")).toBe("Tracks");
  });

  it("leaves a name the operator or an import chose alone", () => {
    expect(layerDisplayName("20260709_Veter3.gpx", "ru")).toBe(
      "20260709_Veter3.gpx",
    );
    expect(layerDisplayName("Северный сектор", "ru")).toBe("Северный сектор");
  });
});
