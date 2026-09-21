import { describe, expect, it, beforeEach, vi } from "vitest";
import { get } from "svelte/store";

// Fresh module state per test: i18n keeps a module-level store.
async function loadI18n() {
  vi.resetModules();
  return await import("../lib/i18n");
}

describe("i18n locale store", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  // The users are a Russian-speaking search-and-rescue crew. The OS language
  // says nothing about that — the owner runs an English system — so the app
  // opens in Russian and switching is an explicit choice, not a guess.
  it("defaults to Russian regardless of the system language", async () => {
    vi.stubGlobal("navigator", { language: "en-US" });
    expect(get((await loadI18n()).locale)).toBe("ru");

    vi.stubGlobal("navigator", { language: "de-DE" });
    expect(get((await loadI18n()).locale)).toBe("ru");

    vi.stubGlobal("navigator", { language: "ru-RU" });
    expect(get((await loadI18n()).locale)).toBe("ru");
  });

  it("defaults to Russian when the environment reports no language at all", async () => {
    vi.stubGlobal("navigator", undefined);
    expect(get((await loadI18n()).locale)).toBe("ru");
  });

  it("persists an explicit choice and restores it over the default", async () => {
    vi.stubGlobal("navigator", { language: "ru-RU" });
    const first = await loadI18n();
    first.setLocale("en");
    expect(localStorage.getItem("ozi:locale")).toBe("en");

    const second = await loadI18n();
    expect(get(second.locale)).toBe("en");
  });

  it("translates keys reactively when the locale changes", async () => {
    vi.stubGlobal("navigator", { language: "ru-RU" });
    const { t, setLocale } = await loadI18n();
    expect(get(t)("palette.saveProject")).toBe("Сохранить проект…");
    setLocale("en");
    expect(get(t)("palette.saveProject")).toBe("Save project…");
  });

  it("ignores a corrupted stored locale", async () => {
    localStorage.setItem("ozi:locale", "xx");
    vi.stubGlobal("navigator", { language: "ru-RU" });
    const { locale } = await loadI18n();
    expect(get(locale)).toBe("ru");
  });
});
