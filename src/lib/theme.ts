import { flavors } from "@catppuccin/palette";

export type ThemeName = "auto" | "latte" | "frappe" | "macchiato" | "mocha";

function resolveTheme(name: ThemeName): Exclude<ThemeName, "auto"> {
  if (name !== "auto") {
    return name;
  }

  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "mocha" : "latte";
}

export function applyTheme(name: ThemeName) {
  const palette = flavors[resolveTheme(name)].colors;
  const root = document.documentElement;

  for (const [key, color] of Object.entries(palette)) {
    root.style.setProperty(`--ctp-${key}`, color.hex);
  }
}

export function applyStoredTheme() {
  applyTheme((localStorage.getItem("theme") ?? "auto") as ThemeName);
}
