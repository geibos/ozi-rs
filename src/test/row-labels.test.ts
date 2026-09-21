import { describe, expect, it } from "vitest";
import { get } from "svelte/store";
import { locale, t } from "../lib/i18n";

/**
 * The library rows carried English in a Russian window: the eye button's
 * tooltip said "Hide", its label said "Hide <name>", the row menu was
 * "Actions" and the swatch "Track color". The visible text around them was
 * already Russian, and these are also what a screen reader speaks and what the
 * customer-journey smoke matches on.
 */
describe("library row labels", () => {
  it("speaks the interface's language", () => {
    locale.set("ru");
    const ru = get(t);
    expect(ru("row.hideShort")).toBe("Скрыть");
    expect(ru("row.showShort")).toBe("Показать");
    expect(ru("row.actions")).toBe("Действия");
    expect(ru("row.trackColor")).toBe("Цвет трека");

    locale.set("en");
    expect(get(t)("row.actions")).toBe("Actions");
  });

  /**
   * Russian declines the object of "скрыть", and a track name substituted raw
   * stays nominative: "Скрыть точка отсечки" is what a crew would read. The
   * colon sidesteps the case for any name, including the ones that are dates
   * and call signs rather than words.
   */
  it("does not decline a name it cannot decline", () => {
    locale.set("ru");
    const label = get(t)("row.hide").replace("{name}", "точка отсечки");
    expect(label).toBe("Скрыть: точка отсечки");
    expect(label).not.toBe("Скрыть точка отсечки");
  });
});
