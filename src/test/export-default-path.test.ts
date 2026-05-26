import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

// Track export wiring now lives in the Library Tracks tab. The legacy
// `TracksPanel.svelte` was deleted by `redesign-library-sidebar`.
const tracksTabSource = readFileSync(
  join(__dirname, "../components/library/TracksTab.svelte"),
  "utf-8"
);

const apiSource = readFileSync(join(__dirname, "../lib/api.ts"), "utf-8");

describe("track export dialog default paths", () => {
  it("uses a typed API wrapper to request backend-built export defaults", () => {
    expect(apiSource).toContain("getTrackExportDefaultPath");
    expect(apiSource).toContain('invoke("get_track_export_default_path"');
  });

  it("passes backend GPX and PLT defaults into save dialogs", () => {
    expect(tracksTabSource).toContain("getTrackExportDefaultPath");
    expect(tracksTabSource).toContain('await getTrackExportDefaultPath(t.name, "gpx")');
    expect(tracksTabSource).toContain('await getTrackExportDefaultPath(t.name, "plt")');
    expect(tracksTabSource).toContain('defaultPath: defaultPath ?? `${t.name}.gpx`');
    expect(tracksTabSource).toContain('defaultPath: defaultPath ?? `${t.name}.plt`');
  });

  it("keeps export dialogs behind API wrappers without direct invoke calls", () => {
    expect(tracksTabSource).not.toContain("invoke(");
    expect(tracksTabSource).toContain("exportGpx");
    expect(tracksTabSource).toContain("exportTrackPlt");
  });
});
