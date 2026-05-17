// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  SEMANTIC_MAP_DARK,
  SEMANTIC_MAP_LIGHT,
  applySemanticTokens,
  applyTheme,
  formatHslTriplet,
  installAutoThemeListener,
  type ResolvedTheme,
  type SemanticToken,
} from "../lib/theme";

const HSL_TRIPLET_RE = /^\d+(\.\d+)?\s+\d+(\.\d+)?%\s+\d+(\.\d+)?%$/;

const ALL_TOKENS: readonly SemanticToken[] = [
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

function clearRoot() {
  document.documentElement.removeAttribute("style");
  document.documentElement.classList.remove("dark");
}

beforeEach(clearRoot);
afterEach(clearRoot);

describe("formatHslTriplet", () => {
  it("renders the HSL triplet shape used by Tailwind", () => {
    const text = formatHslTriplet({ h: 220, s: 0.23, l: 0.95 });
    expect(text).toMatch(HSL_TRIPLET_RE);
    expect(text).toBe("220 23% 95%");
  });
});

describe("applySemanticTokens", () => {
  const flavours: ResolvedTheme[] = ["latte", "frappe", "macchiato", "mocha"];

  for (const flavour of flavours) {
    it(`emits HSL-triplet values for every semantic token (${flavour})`, () => {
      applySemanticTokens(flavour);

      const root = document.documentElement;
      const expectedMap = flavour === "latte" ? SEMANTIC_MAP_LIGHT : SEMANTIC_MAP_DARK;

      expect(Object.keys(expectedMap).sort()).toEqual([...ALL_TOKENS].sort());

      for (const token of ALL_TOKENS) {
        const value = root.style.getPropertyValue(`--${token}`).trim();
        expect(value, `--${token} on ${flavour}`).toMatch(HSL_TRIPLET_RE);
      }
    });

    it(`toggles the dark class correctly for ${flavour}`, () => {
      applySemanticTokens(flavour);
      expect(document.documentElement.classList.contains("dark")).toBe(flavour !== "latte");
    });
  }
});

describe("applyTheme auto + matchMedia", () => {
  it("re-resolves dark/light when matchMedia flips under auto", () => {
    const listeners = new Set<(e: MediaQueryListEvent) => void>();
    let prefersDark = false;
    const mql: MediaQueryList = {
      get matches() {
        return prefersDark;
      },
      media: "(prefers-color-scheme: dark)",
      onchange: null,
      addEventListener: (_: string, l: EventListener) => {
        listeners.add(l as (e: MediaQueryListEvent) => void);
      },
      removeEventListener: (_: string, l: EventListener) => {
        listeners.delete(l as (e: MediaQueryListEvent) => void);
      },
      dispatchEvent: () => true,
      addListener: () => {},
      removeListener: () => {},
    };

    vi.stubGlobal(
      "matchMedia",
      vi.fn(() => mql),
    );

    vi.stubGlobal("localStorage", {
      getItem: vi.fn(() => "auto"),
      setItem: vi.fn(),
      removeItem: vi.fn(),
      clear: vi.fn(),
      key: vi.fn(),
      length: 0,
    });

    applyTheme("auto");
    expect(document.documentElement.classList.contains("dark")).toBe(false);

    const unlisten = installAutoThemeListener();

    prefersDark = true;
    for (const l of listeners) {
      l({ matches: true, media: mql.media } as MediaQueryListEvent);
    }
    expect(document.documentElement.classList.contains("dark")).toBe(true);

    prefersDark = false;
    for (const l of listeners) {
      l({ matches: false, media: mql.media } as MediaQueryListEvent);
    }
    expect(document.documentElement.classList.contains("dark")).toBe(false);

    unlisten();
    expect(listeners.size).toBe(0);
    vi.unstubAllGlobals();
  });
});

describe("semantic token allow-list does not overlap with domain colour fields", () => {
  it("has no overlap with TrackStyle / waypoint colour names", () => {
    const trackOrWaypointColourFields = new Set<string>([
      "color",
      "track-color",
      "waypoint-color",
      "stroke",
      "fill",
      "rgba",
    ]);
    for (const token of ALL_TOKENS) {
      expect(trackOrWaypointColourFields.has(token)).toBe(false);
    }
  });
});
