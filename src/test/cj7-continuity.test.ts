import { describe, expect, it } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

// CJ-7 "Continuity" slice — never lose work. Source-grep wiring assertions
// (same convention as waypoint-visibility.test.ts): the audit-confirmed gaps
// were no Cmd+S, no Cmd+Z, no dirty indicator and no window close guard.
const layoutSource = readFileSync(
  join(__dirname, "../routes/+layout.svelte"),
  "utf-8",
);
const shellSource = readFileSync(
  join(__dirname, "../components/WorkspaceShell.svelte"),
  "utf-8",
);
const paletteSource = readFileSync(
  join(__dirname, "../components/CommandPalette.svelte"),
  "utf-8",
);
const actionsSource = readFileSync(
  join(__dirname, "../lib/actions/project.ts"),
  "utf-8",
);
const capabilitiesSource = readFileSync(
  join(__dirname, "../../src-tauri/capabilities/default.json"),
  "utf-8",
);

describe("CJ-7 global keyboard chords (+layout.svelte)", () => {
  it("wires Cmd/Ctrl+S to quickSave", () => {
    expect(layoutSource).toContain('key === "s"');
    expect(layoutSource).toContain("quickSave()");
  });

  it("wires Cmd/Ctrl+Z to undo and Cmd/Ctrl+Shift+Z to redo", () => {
    expect(layoutSource).toContain('key === "z"');
    expect(layoutSource).toContain("event.shiftKey");
    expect(layoutSource).toContain("doUndo()");
    expect(layoutSource).toContain("doRedo()");
  });

  it("skips the chords while focus is in an editable element", () => {
    // Typing a track name must keep native text editing (Cmd+Z = text
    // undo), so the handler bails out on editable targets before the
    // save/undo/redo branches.
    expect(layoutSource).toContain("isEditableTarget");
    expect(layoutSource).toContain(
      "if (isEditableTarget(event.target)) return;",
    );
    expect(layoutSource).toContain("contenteditable");
  });

  it("delegates to the shared $lib/actions/project helpers", () => {
    expect(layoutSource).toContain(
      'import { doRedo, doUndo, quickSave } from "$lib/actions/project";',
    );
  });
});

describe("CJ-7 window close guard (+layout.svelte)", () => {
  it("registers onCloseRequested and gates on projectDirty", () => {
    expect(layoutSource).toContain("onCloseRequested");
    expect(layoutSource).toContain("getCurrentWindow()");
    expect(layoutSource).toContain("projectDirty");
    // Clean project → the handler returns without preventDefault so the
    // close proceeds.
    expect(layoutSource).toContain("if (!get(projectDirty)) return;");
  });

  it("prevents the close, confirms, then destroys on confirm", () => {
    expect(layoutSource).toContain("event.preventDefault();");
    expect(layoutSource).toContain("confirmDialog(");
    expect(layoutSource).toContain('translate("closeGuard.message")');
    expect(layoutSource).toContain('title: translate("closeGuard.title")');
    expect(layoutSource).toContain("getCurrentWindow().destroy()");
  });

  it("has the capability grants the guard needs at runtime", () => {
    // `onCloseRequested`'s JS wrapper calls `window.destroy()` on every
    // non-prevented close, and the guard shows a plugin-dialog confirm —
    // both are denied unless the capability lists them explicitly.
    expect(capabilitiesSource).toContain("core:window:allow-destroy");
    expect(capabilitiesSource).toContain("dialog:allow-confirm");
  });
});

describe("CJ-7 workspace shell controls (WorkspaceShell.svelte)", () => {
  it("renders a Save control calling quickSave", () => {
    expect(shellSource).toContain("quickSave()");
    expect(shellSource).toContain('$t("shell.save")');
  });

  it("renders a dirty indicator bound to projectDirty", () => {
    expect(shellSource).toContain("$projectDirty");
    expect(shellSource).toContain('data-testid="dirty-indicator"');
    expect(shellSource).toContain('$t("shell.unsaved")');
    expect(shellSource).toContain('$t("shell.saved")');
  });

  it("renders Undo/Redo controls calling the shared helpers", () => {
    // Audit finding: undo was reachable ONLY via the palette. The shell
    // buttons close that gap.
    expect(shellSource).toContain("doUndo()");
    expect(shellSource).toContain("doRedo()");
    expect(shellSource).toContain('$t("shell.undo")');
    expect(shellSource).toContain('$t("shell.redo")');
  });
});

describe("CJ-7 command palette (CommandPalette.svelte)", () => {
  it("delegates save/undo/redo to $lib/actions/project", () => {
    expect(paletteSource).toContain(
      'import { doRedo, doUndo, quickSave } from "$lib/actions/project";',
    );
    expect(paletteSource).toContain("void quickSave();");
    expect(paletteSource).toContain("void doUndo();");
    expect(paletteSource).toContain("void doRedo();");
  });

  it("has a language toggle item in the Settings group", () => {
    expect(paletteSource).toContain("toggleLocale");
    expect(paletteSource).toContain('$t("palette.language")');
    expect(paletteSource).toContain('value="setting:language"');
  });

  it("localizes the project action labels", () => {
    expect(paletteSource).toContain('$t("palette.openProject")');
    expect(paletteSource).toContain('$t("palette.saveProject")');
    expect(paletteSource).toContain('$t("palette.undo")');
    expect(paletteSource).toContain('$t("palette.redo")');
  });
});

describe("CJ-7 shared project actions (src/lib/actions/project.ts)", () => {
  it("quickSave saves straight to the known projectPath without a dialog", () => {
    expect(actionsSource).toContain("get(projectPath)");
    expect(actionsSource).toContain("await saveProject(path);");
    // The dialog is confined to the never-saved fallback (saveAs).
    expect(actionsSource).toMatch(
      /if \(path === null\) \{\s*await saveAs\(\);/,
    );
  });

  it("toasts localized success and failure messages", () => {
    expect(actionsSource).toContain('translate("toast.saved")');
    expect(actionsSource).toContain('translate("toast.saveFailed")');
    expect(actionsSource).toContain('"toast.undoFailed"');
    expect(actionsSource).toContain('"toast.redoFailed"');
  });

  it("exposes saveAs with the palette's project-file dialog filter", () => {
    expect(actionsSource).toContain("export async function saveAs");
    expect(actionsSource).toContain(
      '{ name: "OziRS project", extensions: ["json"] }',
    );
  });
});
