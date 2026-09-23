import { describe, expect, it, vi } from "vitest";
import { get } from "svelte/store";
import { readFileSync } from "fs";
import { join } from "path";
import type { MessageKey } from "../lib/i18n";

const smokeSource = readFileSync(
  join(__dirname, "../../tools/ozi-rs-mcp/tests/smoke_core_workflow.rs"),
  "utf-8",
);

// Fresh module state per read: i18n keeps a module-level store.
async function messageIn(
  locale: "ru" | "en",
  key: MessageKey,
): Promise<string> {
  vi.resetModules();
  const i18n = await import("../lib/i18n");
  i18n.setLocale(locale);
  return get(i18n.t)(key);
}

/**
 * The customer-journey smoke drives the packaged app through Appium, and
 * Appium can only see what WKWebView publishes — labels and titles, not the
 * text inside a button. So the smoke finds its elements and waits for its
 * states by the literal wording of those labels.
 *
 * That makes the translations and a Rust test file a contract with nothing
 * holding it together, and both sides have broken it:
 *
 *   - the drawing toggle went from "Завершить трек (N точек)" to
 *     "Завершить трек · точек: N" (the old shape reads as an error for N = 1
 *     and N = 2) and the smoke went on waiting for a string that no longer
 *     existed;
 *   - the map canvas selector was left English-only when the rest of the file
 *     was made bilingual, so every map click failed the first time the app
 *     under test opened in its default language.
 *
 * Each cost a build, a launch and a timeout to discover. This is the same
 * contract, checked in a second, in both languages.
 */
const CONTRACT: { key: MessageKey; what: string }[] = [
  { key: "tracksTab.finishTrack", what: "the drawing toggle with its count" },
  { key: "shell.mapCanvas", what: "the map canvas the smoke clicks on" },
  { key: "tracksTab.createTrack", what: "the drawing toggle at rest" },
  { key: "layers.menu", what: "the layer menu the smoke opens" },
  { key: "layers.new", what: "the new-layer entry" },
  { key: "layers.create", what: "the button that commits a new layer" },
  { key: "layers.delete", what: "the delete-layer entry" },
  {
    key: "layers.defaultTrackName",
    what: "the name a new track layer arrives with, which is why the journey needs no typing",
  },
  {
    key: "tracksTab.layer",
    what: "the layer select, whose accessible name carries which layer is active",
  },
];

describe("the smoke gate's label contract", () => {
  for (const { key, what } of CONTRACT) {
    for (const locale of ["ru", "en"] as const) {
      it(`knows ${what} as ${locale} renders it`, async () => {
        const rendered = await messageIn(locale, key);
        expect(rendered, `${key} is missing from ${locale}`).toBeTruthy();

        expect(
          smokeSource,
          "tools/ozi-rs-mcp/tests/smoke_core_workflow.rs looks for a label " +
            `${locale} no longer renders. It should contain "${rendered}" ` +
            `(${key}). Update the matching constant or helper there — ` +
            "otherwise the gate only finds out after a build and a launch.",
        ).toContain(rendered);
      });
    }
  }
});
