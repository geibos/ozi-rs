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

  /**
   * A half-finished translation pass leaves English strings in a Russian UI,
   * and the fallback in `translate` hides it: the key resolves, so nothing
   * fails. Comparing the two key sets is the only thing that catches it.
   */
  it("keeps both dictionaries on the same key set", async () => {
    const { dictionaryKeys } = await loadI18n();
    const en = new Set(dictionaryKeys("en"));
    const ru = new Set(dictionaryKeys("ru"));
    expect([...en].filter((k) => !ru.has(k))).toEqual([]);
    expect([...ru].filter((k) => !en.has(k))).toEqual([]);
  });

  it("translates every inspector key into Russian, not through the fallback", async () => {
    const { dictionaryKeys, t, setLocale } = await loadI18n();
    setLocale("ru");
    const translate = get(t);
    const english = await loadI18n();
    english.setLocale("en");
    const inEnglish = get(english.t);
    for (const key of dictionaryKeys("en").filter((k) =>
      k.startsWith("inspector."),
    )) {
      expect(translate(key)).not.toBe(inEnglish(key));
    }
  });

  it("ignores a corrupted stored locale", async () => {
    localStorage.setItem("ozi:locale", "xx");
    vi.stubGlobal("navigator", { language: "ru-RU" });
    const { locale } = await loadI18n();
    expect(get(locale)).toBe("ru");
  });
});
