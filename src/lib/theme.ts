import { flavors } from "@catppuccin/palette";
import type { ColorName, FlavorName } from "@catppuccin/palette";

export type ThemeName = "auto" | "latte" | "frappe" | "macchiato" | "mocha";
export type ResolvedTheme = Exclude<ThemeName, "auto">;

export type SemanticToken =
  | "background"
  | "foreground"
  | "card"
  | "card-foreground"
  | "popover"
  | "popover-foreground"
  | "primary"
  | "primary-foreground"
  | "secondary"
  | "secondary-foreground"
  | "muted"
  | "muted-foreground"
  | "accent"
  | "accent-foreground"
  | "destructive"
  | "destructive-foreground"
  | "border"
  | "input"
  | "ring";

type SemanticMap = Readonly<Record<SemanticToken, ColorName>>;

export const SEMANTIC_MAP_LIGHT: SemanticMap = {
  background: "base",
  foreground: "text",
  card: "mantle",
  "card-foreground": "text",
  popover: "base",
  "popover-foreground": "text",
  primary: "blue",
  "primary-foreground": "base",
  secondary: "surface0",
  "secondary-foreground": "text",
  muted: "surface0",
  "muted-foreground": "subtext1",
  accent: "surface1",
  "accent-foreground": "text",
  destructive: "red",
  "destructive-foreground": "base",
  border: "surface1",
  input: "surface1",
  ring: "lavender",
};

export const SEMANTIC_MAP_DARK: SemanticMap = {
  background: "base",
  foreground: "text",
  card: "surface0",
  "card-foreground": "text",
  popover: "surface0",
  "popover-foreground": "text",
  primary: "blue",
  "primary-foreground": "crust",
  secondary: "surface1",
  "secondary-foreground": "text",
  muted: "surface0",
  "muted-foreground": "subtext0",
  accent: "surface1",
  "accent-foreground": "text",
  destructive: "red",
  "destructive-foreground": "crust",
  border: "surface2",
  input: "surface2",
  ring: "lavender",
};

const round = (value: number, digits = 1): number => {
  const factor = 10 ** digits;
  return Math.round(value * factor) / factor;
};

export function formatHslTriplet(hsl: { h: number; s: number; l: number }): string {
  return `${round(hsl.h)} ${round(hsl.s * 100)}% ${round(hsl.l * 100)}%`;
}

function resolveTheme(name: ThemeName): ResolvedTheme {
  if (name !== "auto") {
    return name;
  }
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "mocha" : "latte";
}

export function applySemanticTokens(name: ResolvedTheme): void {
  const flavor: FlavorName = name;
  const palette = flavors[flavor].colors;
  const map: SemanticMap = name === "latte" ? SEMANTIC_MAP_LIGHT : SEMANTIC_MAP_DARK;
  const root = document.documentElement;

  for (const [token, colorName] of Object.entries(map) as [SemanticToken, ColorName][]) {
    const color = palette[colorName];
    root.style.setProperty(`--${token}`, formatHslTriplet(color.hsl));
  }

  root.classList.toggle("dark", name !== "latte");
}

export function applyTheme(name: ThemeName): void {
  const resolved = resolveTheme(name);
  const palette = flavors[resolved].colors;
  const root = document.documentElement;

  for (const [key, color] of Object.entries(palette)) {
    root.style.setProperty(`--ctp-${key}`, color.hex);
  }

  applySemanticTokens(resolved);
}

export function applyStoredTheme(): void {
  applyTheme((localStorage.getItem("theme") ?? "auto") as ThemeName);
}

export function installAutoThemeListener(): () => void {
  const mql = window.matchMedia("(prefers-color-scheme: dark)");
  const handler = () => {
    const stored = (localStorage.getItem("theme") ?? "auto") as ThemeName;
    if (stored === "auto") {
      applyTheme("auto");
    }
  };
  mql.addEventListener("change", handler);
  return () => mql.removeEventListener("change", handler);
}
