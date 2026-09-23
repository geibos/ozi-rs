import { describe, expect, it } from "vitest";
import { dictionaryKeys } from "../lib/i18n";
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
    // save/undo/redo branches. The predicate moved to `$lib/editable-target`
    // when the measuring tool's Backspace needed the same answer — two
    // handlers with their own idea of "editable" is how they come to disagree
    // about one keypress. Its own test covers what counts as editable.
    expect(layoutSource).toContain(
      'import { isEditableTarget } from "$lib/editable-target";',
    );
    expect(layoutSource).toContain(
      "if (isEditableTarget(event.target)) return;",
    );
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

  it("prevents the close, asks, and offers saving as well as quitting", () => {
    // The question used to be the operating system's confirm box, which has
    // two buttons — so "save and quit", the answer an operator almost always
    // wants at that moment, was not on offer. It is our own dialog now.
    expect(layoutSource).toContain("event.preventDefault();");
    expect(layoutSource).toContain("askBeforeClosing()");
    expect(layoutSource).toContain('if (choice === "stay") return;');
    expect(layoutSource).toContain("getCurrentWindow().destroy()");

    const guard = readFileSync(
      join(__dirname, "../components/CloseGuard.svelte"),
      "utf-8",
    );
    for (const key of [
      "closeGuard.title",
      "closeGuard.message",
      "closeGuard.saveAndQuit",
      "closeGuard.quit",
      "closeGuard.cancel",
    ]) {
      expect(guard, `the guard must offer ${key}`).toContain(key);
    }
  });

  it("does not quit when the save did not land", () => {
    // Quitting after a failed save, or after a save-as the operator cancelled,
    // loses the work just as surely as quitting without saving.
    expect(layoutSource).toContain("if (get(projectDirty)) return;");
  });

  it("has the capability grants the guard needs at runtime", () => {
    // `onCloseRequested`'s JS wrapper calls `window.destroy()` on every
    // non-prevented close, which is denied unless the capability lists it.
    // The confirm permission is no longer part of this: the question is an
    // in-app dialog, because the plugin's box has only two buttons.
    expect(capabilitiesSource).toContain("core:window:allow-destroy");
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
    // One import, four names now that opening a project is shared too, so
    // the assertion is per name rather than on the whole line.
    for (const name of ["doRedo", "doUndo", "openProjectFile", "quickSave"]) {
      expect(paletteSource).toContain(`    ${name},`);
    }
    expect(paletteSource).toContain('} from "$lib/actions/project";');
    expect(paletteSource).toContain("void quickSave();");
    expect(paletteSource).toContain("void doUndo();");
    expect(paletteSource).toContain("void doRedo();");
  });

  it("has a language toggle item in the Settings group", () => {
    expect(paletteSource).toContain("toggleLocale");
    expect(paletteSource).toContain('"palette.language"');
    expect(paletteSource).toContain('value="setting:language"');
  });

  // The keys, not the spelling of the store read: the palette imports the
  // i18n store under an alias because its tracks loop binds `t`.
  it("localizes the project action labels", () => {
    for (const key of [
      "palette.openProject",
      "palette.saveProject",
      "palette.undo",
      "palette.redo",
    ]) {
      expect(paletteSource).toContain(`"${key}"`);
      expect(dictionaryKeys("ru")).toContain(key);
    }
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

  /**
   * This used to pin `extensions: ["json"]`, which was the defect rather than
   * the contract: the format is `.ozp` and a filter on `json` hid every `.ozp`
   * in the open dialog. Both dialogs read the extension from one module now,
   * which is what is worth pinning — the value itself is covered behaviourally
   * in `project-file-extension.test.ts`.
   */
  it("exposes saveAs, taking its dialog filter from the project-file module", () => {
    expect(actionsSource).toContain("export async function saveAs");
    expect(actionsSource).toContain("PROJECT_SAVE_EXTENSION");
    expect(actionsSource).not.toContain('extensions: ["json"]');
  });
});
