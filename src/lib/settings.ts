/**
 * Settings that belong to the machine rather than to the project.
 *
 * The theme and the language already live in `localStorage` (`theme.ts`,
 * `i18n.ts`); the thresholds the track statistics use join them. They describe
 * how a crew's GPS behaves, not the search, and a project handed to another HQ
 * must not change the statistics they see.
 */
import { writable, type Readable } from "svelte/store";
import { DEFAULT_MOTION_SETTINGS, type MotionSettings } from "./track-motion";

const STORAGE_KEY = "ozi:motion-settings";

function isPositive(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value) && value > 0;
}

function load(): MotionSettings {
  try {
    if (typeof localStorage === "undefined") return DEFAULT_MOTION_SETTINGS;
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return DEFAULT_MOTION_SETTINGS;
    const stored = JSON.parse(raw) as Partial<Record<keyof MotionSettings, unknown>>;
    // Value by value: one field written by an older build, or edited by hand,
    // must not cost the operator the other two.
    const out = { ...DEFAULT_MOTION_SETTINGS };
    for (const key of Object.keys(out) as Array<keyof MotionSettings>) {
      if (isPositive(stored?.[key])) out[key] = stored[key] as number;
    }
    return out;
  } catch {
    return DEFAULT_MOTION_SETTINGS;
  }
}

const store = writable<MotionSettings>(load());

export const motionSettings: Readable<MotionSettings> = {
  subscribe: store.subscribe,
};

/**
 * Set one threshold. A value that is not a positive number is ignored, so a
 * field cleared on the way to typing a new number leaves the old one in force.
 */
export function setMotionSetting(
  key: keyof MotionSettings,
  value: number,
): void {
  if (!isPositive(value)) return;
  store.update((current) => {
    const next = { ...current, [key]: value };
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
    } catch {
      // Best-effort, as the theme and the language are: the value is in force
      // for this session either way.
    }
    return next;
  });
}
