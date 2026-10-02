// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { get } from "svelte/store";
import { tick } from "svelte";

import Settings from "../components/Settings.svelte";
import { motionSettings, setMotionSetting } from "../lib/settings";
import { settingsOpen } from "../lib/stores";
import { setLocale } from "../lib/i18n";

// jsdom has no `matchMedia`, and the theme picker applies the theme it shows.
window.matchMedia ??= ((query: string) => ({
  matches: false,
  media: query,
  onchange: null,
  addEventListener: () => {},
  removeEventListener: () => {},
  addListener: () => {},
  removeListener: () => {},
  dispatchEvent: () => false,
})) as typeof window.matchMedia;

/**
 * The settings screen writes what the operator types into the thresholds the
 * inspector reads, and a field cleared on the way to a new number does not
 * zero anything.
 */
describe("the settings screen", () => {
  beforeEach(async () => {
    localStorage.clear();
    setLocale("ru");
    setMotionSetting("stopDistanceM", 25);
    setMotionSetting("stopWindowS", 120);
    setMotionSetting("climbThresholdM", 5);
    settingsOpen.set(true);
    render(Settings);
    await tick();
  });

  afterEach(() => {
    settingsOpen.set(false);
    cleanup();
  });

  it("shows the thresholds in force, the window in minutes", () => {
    expect(
      (screen.getByLabelText("Расстояние стоянки, м") as HTMLInputElement).value,
    ).toBe("25");
    expect(
      (screen.getByLabelText("Окно стоянки, мин") as HTMLInputElement).value,
    ).toBe("2");
    expect(
      (screen.getByLabelText("Порог перепада, м") as HTMLInputElement).value,
    ).toBe("5");
  });

  it("puts a typed value in force", async () => {
    await fireEvent.input(screen.getByLabelText("Расстояние стоянки, м"), {
      target: { value: "40" },
    });
    await fireEvent.input(screen.getByLabelText("Окно стоянки, мин"), {
      target: { value: "3" },
    });
    expect(get(motionSettings).stopDistanceM).toBe(40);
    expect(get(motionSettings).stopWindowS).toBe(180);
  });

  it("keeps the old value while the field is empty, and shows it again on leaving", async () => {
    const field = screen.getByLabelText("Порог перепада, м") as HTMLInputElement;
    await fireEvent.input(field, { target: { value: "" } });
    expect(get(motionSettings).climbThresholdM).toBe(5);
    await fireEvent.blur(field);
    expect(field.value).toBe("5");
  });

  it("offers the theme and the language", () => {
    expect(screen.getByTestId("settings-theme")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Русский" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "English" })).toBeTruthy();
  });
});
