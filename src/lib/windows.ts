import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

export async function openBundleLoader() {
  const existing = await WebviewWindow.getByLabel("bundles");
  if (existing) {
    await existing.setFocus();
    return;
  }

  new WebviewWindow("bundles", {
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
