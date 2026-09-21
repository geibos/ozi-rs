import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

/**
 * The context bar must stay inside the canvas column.
 *
 * `.canvas-column` was a grid with rows but no columns, so its single
 * implicit column was sized `auto` — that is, to max-content. The context bar
 * is wider than the canvas whenever the inspector is open, so the column grew
 * to fit the bar instead of the bar shrinking to fit the column. The bar then
 * ran under the inspector, which is painted after it: at 1024×640 with a
 * track selected, `document.elementFromPoint` at the centre of Save, Undo,
 * Redo and the palette trigger returned the inspector's header. Every one of
 * them was unclickable, from the moment a track was selected — which is the
 * moment editing starts.
 *
 * jsdom computes no layout, so the measurement lives in the change's record
 * (`openspec/changes/the-toolbar-stays-clickable/`) and was taken on the
 * stand at 880, 1024, 1280, 1300 and 1440 px. What this test can do is pin
 * the three declarations that keep it true, so the next edit to this
 * stylesheet cannot quietly undo them.
 */
const shell = readFileSync(
  join(__dirname, "../components/WorkspaceShell.svelte"),
  "utf-8",
);

function block(selector: string): string {
  const start = shell.indexOf(`  ${selector} {`);
  expect(start, `${selector} is gone from WorkspaceShell`).toBeGreaterThan(-1);
  return shell.slice(start, shell.indexOf("\n  }", start));
}

describe("the context bar cannot outgrow the canvas column", () => {
  it("the canvas column declares a shrinkable column, not an implicit auto one", () => {
    expect(block(".canvas-column")).toContain(
      "grid-template-columns: minmax(0, 1fr)",
    );
  });

  it("the bar may shrink below its content width", () => {
    expect(block(".context-bar")).toContain("min-width: 0");
  });

  it("the actions never give up their width", () => {
    // Undo, redo, save and the palette. Everything else in the bar yields
    // first — the mode chips are inert scaffolding and go entirely.
    expect(block(".bar-actions")).toContain("flex-shrink: 0");
    expect(block(".mode-chips")).toContain("min-width: 0");
  });

  it("the narrow bar is measured against the bar, not the window", () => {
    // The library rail and the inspector are what take the width away, so a
    // window-width media query would ask the wrong question.
    expect(block(".context-bar")).toContain("container-type: inline-size");
    expect(shell).toContain("@container (max-width: 640px)");
    expect(shell).toContain("@container (max-width: 380px)");
  });
});
