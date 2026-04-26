import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

const tracksPanelSource = readFileSync(
  join(__dirname, "../components/TracksPanel.svelte"),
  "utf-8"
);

const apiSource = readFileSync(join(__dirname, "../lib/api.ts"), "utf-8");

describe("track export dialog default paths", () => {
  it("uses a typed API wrapper to request backend-built export defaults", () => {
    expect(apiSource).toContain("getTrackExportDefaultPath");
    expect(apiSource).toContain('invoke("get_track_export_default_path"');
  });

  it("passes backend GPX and PLT defaults into save dialogs", () => {
    expect(tracksPanelSource).toContain("getTrackExportDefaultPath");
    expect(tracksPanelSource).toContain('await getTrackExportDefaultPath(track.name, "gpx")');
    expect(tracksPanelSource).toContain('await getTrackExportDefaultPath(track.name, "plt")');
    expect(tracksPanelSource).toContain('defaultPath: defaultPath ?? `${track.name}.gpx`');
    expect(tracksPanelSource).toContain('defaultPath: defaultPath ?? `${track.name}.plt`');
  });

  it("keeps export dialogs behind API wrappers without direct invoke calls", () => {
    expect(tracksPanelSource).not.toContain("invoke(");
    expect(tracksPanelSource).toContain("exportGpx");
    expect(tracksPanelSource).toContain("exportTrackPlt");
  });
});
