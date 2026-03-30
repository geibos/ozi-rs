import { writable, derived } from "svelte/store";
import type { AppStateDto } from "./types";
import { getAppState } from "./api";

function createAppStore() {
  const { subscribe, set, update } = writable<AppStateDto | null>(null);

  return {
    subscribe,
    async refresh() {
      const state = await getAppState();
      set(state);
    },
  };
}

export const appState = createAppStore();

export const busy = derived(appState, ($s) => $s?.busy ?? false);
export const status = derived(appState, ($s) => $s?.status ?? "");
export const diagnostics = derived(appState, ($s) => $s?.diagnostics ?? []);
export const projects = derived(appState, ($s) => $s?.projects ?? []);
export const currentProject = derived(appState, ($s) => $s?.current_project ?? null);
export const activeMap = derived(appState, ($s) => $s?.active_map ?? null);
export const trackLayerCount = derived(appState, ($s) => $s?.track_layer_count ?? 0);

// UI-only state (not persisted)
export const consoleOpen = writable(false);
export const tracksPanelOpen = writable(true);
export const selectedTheme = writable<string>(
  localStorage.getItem("theme") ?? "auto"
);

selectedTheme.subscribe((v) => localStorage.setItem("theme", v));
