import "./app.css";
import { mount } from "svelte";

const view = new URLSearchParams(window.location.search).get("view");

if (view === "bundles") {
  const { default: BundleLoaderView } = await import("./views/BundleLoaderView.svelte");
  mount(BundleLoaderView, { target: document.getElementById("app")! });
} else {
  const { default: App } = await import("./App.svelte");
  mount(App, { target: document.getElementById("app")! });
}
