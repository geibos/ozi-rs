import { flavors } from "@catppuccin/palette";
import type { ColorName, FlavorName } from "@catppuccin/palette";

/**
 * Theme identifiers.
 *
 *   - `native-auto` / `native-light` / `native-dark` — the native Zinc + Teal
 *     default, declared in `src/lib/tokens.css`. `native-auto` tracks the OS
 *     `prefers-color-scheme` setting.
 *   - `auto` — Catppuccin Auto (back-compat alias). Selecting any of the
 *     Catppuccin entries implicitly opts the user into the Catppuccin pack.
 *   - `latte` / `frappe` / `macchiato` / `mocha` — Catppuccin flavours.
 *
 * The fresh-install default (no `localStorage["theme"]`) is `native-auto`.
 * A user who previously selected a Catppuccin flavour keeps it on next
 * launch.
 */
export type ThemeName =
  | "native-auto"
  | "native-light"
  | "native-dark"
  | "auto"
  | "latte"
  | "frappe"
  | "macchiato"
  | "mocha";

export type ResolvedNativeTheme = "native-light" | "native-dark";
export type ResolvedCatppuccinTheme = Exclude<FlavorName, never>;
export type ResolvedTheme = ResolvedNativeTheme | ResolvedCatppuccinTheme;

const CATPPUCCIN_FLAVOURS: ReadonlyArray<ThemeName> = [
  "auto",
  "latte",
  "frappe",
  "macchiato",
  "mocha",
];

export function isCatppuccinTheme(name: ThemeName): boolean {
  return CATPPUCCIN_FLAVOURS.includes(name);
}

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

export function formatHslTriplet(hsl: {
  h: number;
  s: number;
  l: number;
}): string {
  return `${round(hsl.h)} ${round(hsl.s * 100)}% ${round(hsl.l * 100)}%`;
}

const CATPPUCCIN_PALETTE_KEYS: ReadonlyArray<ColorName> = [
  "rosewater",
  "flamingo",
  "pink",
  "mauve",
  "red",
  "maroon",
  "peach",
  "yellow",
  "green",
  "teal",
  "sky",
  "sapphire",
  "blue",
  "lavender",
  "text",
  "subtext1",
  "subtext0",
  "overlay2",
  "overlay1",
  "overlay0",
  "surface2",
  "surface1",
  "surface0",
  "base",
  "mantle",
  "crust",
];

const NATIVE_SEMANTIC_TOKENS: ReadonlyArray<SemanticToken> = [
  "background",
  "foreground",
  "card",
  "card-foreground",
  "popover",
  "popover-foreground",
  "primary",
  "primary-foreground",
  "secondary",
  "secondary-foreground",
  "muted",
  "muted-foreground",
  "accent",
  "accent-foreground",
  "destructive",
  "destructive-foreground",
  "border",
  "input",
  "ring",
];

function resolveNative(name: ThemeName): ResolvedNativeTheme {
  if (name === "native-light") return "native-light";
  if (name === "native-dark") return "native-dark";
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "native-dark"
    : "native-light";
}

function resolveCatppuccin(name: ThemeName): FlavorName {
  if (name === "latte") return "latte";
  if (name === "frappe") return "frappe";
  if (name === "macchiato") return "macchiato";
  if (name === "mocha") return "mocha";
  // `auto`
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "mocha"
    : "latte";
}

export function applySemanticTokens(name: FlavorName): void {
  const palette = flavors[name].colors;
  const map: SemanticMap =
    name === "latte" ? SEMANTIC_MAP_LIGHT : SEMANTIC_MAP_DARK;
  const root = document.documentElement;

  for (const [token, colorName] of Object.entries(map) as [
    SemanticToken,
    ColorName,
  ][]) {
    const color = palette[colorName];
    root.style.setProperty(`--${token}`, formatHslTriplet(color.hsl));
  }

  root.classList.toggle("dark", name !== "latte");
}

/**
 * Clear runtime semantic-token and palette overrides so the static
 * `:root` / `.dark` rules from `tokens.css` take effect. Used when the
 * user switches back from a Catppuccin flavour to the native default.
 */
function clearCatppuccinOverrides(): void {
  const root = document.documentElement;
  for (const token of NATIVE_SEMANTIC_TOKENS) {
    root.style.removeProperty(`--${token}`);
  }
  for (const key of CATPPUCCIN_PALETTE_KEYS) {
    root.style.removeProperty(`--ctp-${key}`);
  }
}

/**
 * Load the Catppuccin theme-pack stylesheet on demand. The `?inline` query
 * resolves to a side-effect import that Vite inlines into the bundle on
 * first call; subsequent calls are no-ops.
 */
let catppuccinPackLoaded = false;
async function ensureCatppuccinPackLoaded(): Promise<void> {
  if (catppuccinPackLoaded) return;
  await import("./themes/catppuccin.css");
  catppuccinPackLoaded = true;
}

function applyNativeTheme(name: ThemeName): void {
  clearCatppuccinOverrides();
  const resolved = resolveNative(name);
  document.documentElement.classList.toggle("dark", resolved === "native-dark");
}

async function applyCatppuccinTheme(name: ThemeName): Promise<void> {
  await ensureCatppuccinPackLoaded();
  const resolved = resolveCatppuccin(name);
  const palette = flavors[resolved].colors;
  const root = document.documentElement;
  for (const [key, color] of Object.entries(palette)) {
    root.style.setProperty(`--ctp-${key}`, color.hex);
  }
  applySemanticTokens(resolved);
}

/**
 * Apply a theme. Returns a promise so callers that need to await the
 * Catppuccin pack load can do so; the native path completes synchronously
 * but is still returned as a resolved promise for a uniform signature.
 */
export function applyTheme(name: ThemeName): Promise<void> {
  if (isCatppuccinTheme(name)) {
    return applyCatppuccinTheme(name);
  }
  applyNativeTheme(name);
  return Promise.resolve();
}

export function applyStoredTheme(): void {
  const stored = (localStorage.getItem("theme") ?? "native-auto") as ThemeName;
  void applyTheme(stored);
}

export function installAutoThemeListener(): () => void {
  const mql = window.matchMedia("(prefers-color-scheme: dark)");
  const handler = () => {
    const stored = (localStorage.getItem("theme") ??
      "native-auto") as ThemeName;
    if (stored === "auto" || stored === "native-auto") {
      void applyTheme(stored);
    }
  };
  mql.addEventListener("change", handler);
  return () => mql.removeEventListener("change", handler);
}
