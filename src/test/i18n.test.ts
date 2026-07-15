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

  it("defaults to Russian when navigator reports a ru locale", async () => {
    vi.stubGlobal("navigator", { language: "ru-RU" });
    const { locale } = await loadI18n();
    expect(get(locale)).toBe("ru");
  });

  it("defaults to English for any non-ru locale", async () => {
    vi.stubGlobal("navigator", { language: "de-DE" });
    const { locale } = await loadI18n();
    expect(get(locale)).toBe("en");
  });

  it("persists an explicit choice and restores it over the navigator default", async () => {
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
