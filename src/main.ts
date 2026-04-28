import "./app.css";
import { mount } from "svelte";
import { applyStoredTheme } from "./lib/theme";

const view = new URLSearchParams(window.location.search).get("view");

applyStoredTheme();

if (view === "bundles") {
  const { default: BundleLoaderView } = await import("./views/BundleLoaderView.svelte");
  mount(BundleLoaderView, { target: document.getElementById("app")! });
} else {
  const { default: App } = await import("./App.svelte");
  mount(App, { target: document.getElementById("app")! });

  // Pre-create bundle loader window (hidden) for instant open later
  import("./lib/windows").then(({ precreateBundleLoader }) => {
    precreateBundleLoader();
  });
}
