import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

let bundleWindow: WebviewWindow | null = null;

export async function precreateBundleLoader() {
  bundleWindow = new WebviewWindow("bundles", {
    url: "/?view=bundles",
    title: "Map Bundles",
    width: 460,
    height: 580,
    minWidth: 340,
    minHeight: 400,
    center: true,
    resizable: true,
    visible: false,
  });
}

export async function openBundleLoader() {
  const existing = bundleWindow ?? (await WebviewWindow.getByLabel("bundles"));
  if (existing) {
    await existing.show();
    await existing.setFocus();
    return;
  }

  // Fallback: create new window if pre-creation failed
  bundleWindow = new WebviewWindow("bundles", {
    url: "/?view=bundles",
    title: "Map Bundles",
    width: 460,
    height: 580,
    minWidth: 340,
    minHeight: 400,
    center: true,
    resizable: true,
  });
}
