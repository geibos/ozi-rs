import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

/**
 * The simplify tolerance crosses the boundary in metres.
 *
 * The slider in the Tracks tab is labelled «Допуск: {n} м» and runs from 1 to
 * 1000. Its number was handed to a Rust parameter called `tolerance`, whose
 * unit was written down in one doc comment three files away — kilometres. So
 * the gentlest setting the interface offered, one metre, simplified at one
 * kilometre, and a day's walk around a search area came back as two or three
 * points. The operator saw a slider that destroyed the track wherever they put
 * it. Reported by the owner on 2026-09-23 as "нормализация очень сильно
 * чистит".
 *
 * A comment is not what keeps a unit; a name is. This checks the name survived
 * the next regeneration of the bindings, and that nothing on the way in
 * quietly converts twice.
 */
describe("the simplify tolerance", () => {
  const bindings = readFileSync(
    join(__dirname, "../lib/bindings.ts"),
    "utf-8",
  );

  it("is named for its unit in the generated bindings", () => {
    for (const command of ["simplifyTrack", "getSimplifiedPreview"]) {
      const signature = bindings.match(
        new RegExp(`async ${command}\\(([^)]*)\\)`),
      );
      expect(signature, `${command} is not in the bindings`).not.toBe(null);
      expect(
        (signature as RegExpMatchArray)[1],
        `${command} takes a tolerance whose unit is not in its name. The ` +
          "slider shows metres and the algorithm works in kilometres; the " +
          "only thing that kept those apart for four months was a comment, " +
          "and it did not.",
      ).toContain("toleranceM");
    }
  });

  it("reaches the command as the slider's own number", () => {
    // The conversion belongs on the Rust side, once. A second one here would
    // divide by a million and simplify nothing at all — which looks like a
    // feature that does not work rather than one that works too hard.
    const api = readFileSync(join(__dirname, "../lib/api.ts"), "utf-8");
    const tab = readFileSync(
      join(__dirname, "../components/library/TracksTab.svelte"),
      "utf-8",
    );
    expect(api).not.toMatch(/toleranceM\s*[*/]\s*1000/);
    expect(tab).not.toMatch(/toleranceM\s*[*/]\s*1000/);
    expect(tab).toContain("simplifyTrack(s.layerId, s.trackId, s.toleranceM)");
  });

  it("is what the slider is labelled in", () => {
    const i18n = readFileSync(join(__dirname, "../lib/i18n.ts"), "utf-8");
    expect(i18n).toContain('"tracksTab.tolerance": "Допуск: {tolerance} м"');
    expect(i18n).toContain('"tracksTab.tolerance": "Tolerance: {tolerance} m"');
  });
});
