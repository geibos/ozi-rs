import { describe, it, expect } from "vitest";
import { readFileSync } from "fs";
import { join } from "path";

const caps = JSON.parse(
  readFileSync(
    join(__dirname, "../../src-tauri/capabilities/default.json"),
    "utf-8"
  )
);

// Every permission listed here must be present in capabilities/default.json.
// Add entries here whenever a new Tauri command or plugin requires a permission.
const REQUIRED_PERMISSIONS = [
  // Window and webview creation from JS
  "core:window:allow-create",
  "core:webview:allow-create-webview-window", // needed by `new WebviewWindow(...)`
  // Dialogs
  "dialog:allow-open",
  "dialog:allow-save",
  // Shell
  "shell:allow-open",
];

const REQUIRED_WINDOWS = ["main", "bundles"];

describe("capabilities/default.json", () => {
  it("grants all required permissions", () => {
    for (const perm of REQUIRED_PERMISSIONS) {
      expect(
        caps.permissions,
        `Missing permission: ${perm}`
      ).toContain(perm);
    }
  });

  it("covers all required window labels", () => {
    for (const label of REQUIRED_WINDOWS) {
      expect(
        caps.windows,
        `Missing window label: ${label}`
      ).toContain(label);
    }
  });
});
