<script lang="ts">
  /**
   * Settings, in one place.
   *
   * Before this the theme was seven entries in the command palette, the
   * language a two-letter button in the status bar, and `ThemePicker` — which
   * `ui-shell` requires to be reachable from settings — was mounted nowhere.
   * The thresholds the track statistics need (owner, 2026-10-01) had nowhere
   * to go at all. Opened from the gear in the top bar, from the palette and
   * with ⌘,; mounted once in the root layout so both routes have it.
   */
  import * as Sheet from "$lib/components/ui/sheet";
  import { Input } from "$lib/components/ui/input";
  import ThemePicker from "./ThemePicker.svelte";
  import { locale, setLocale, t, type Locale } from "$lib/i18n";
  import { motionSettings, setMotionSetting } from "$lib/settings";
  import { settingsOpen } from "$lib/stores";
  import type { MotionSettings } from "$lib/track-motion";

  const LANGUAGES: { value: Locale; label: string }[] = [
    { value: "ru", label: "Русский" },
    { value: "en", label: "English" },
  ];

  /**
   * A threshold field. What is typed is applied as soon as it is a positive
   * number; anything else leaves the stored value in force, and leaving the
   * field puts the stored value back in it — a cleared field is a step on the
   * way to a new number, not a request for zero.
   */
  function apply(key: keyof MotionSettings, raw: string, scale = 1) {
    const value = Number(raw.replace(",", "."));
    setMotionSetting(key, value * scale);
  }

  function shown(key: keyof MotionSettings, scale = 1): string {
    return String(Number(($motionSettings[key] / scale).toFixed(2)));
  }
</script>

<Sheet.Root bind:open={$settingsOpen}>
  <Sheet.Content side="right" class="w-[400px] sm:max-w-[400px]">
    <Sheet.Header>
      <Sheet.Title>{$t("settings.title")}</Sheet.Title>
    </Sheet.Header>

    <div class="flex flex-col gap-6 overflow-y-auto px-4 pb-6 text-sm">
      <section aria-labelledby="settings-appearance" class="flex flex-col gap-3">
        <h3
          id="settings-appearance"
          class="text-muted-foreground/80 text-[10px] font-semibold tracking-wider uppercase"
        >
          {$t("settings.appearance")}
        </h3>
        <div class="flex flex-col gap-1.5">
          <span class="text-muted-foreground text-xs">{$t("settings.theme")}</span>
          <ThemePicker />
        </div>
        <div class="flex flex-col gap-1.5">
          <span class="text-muted-foreground text-xs"
            >{$t("settings.language")}</span
          >
          <div class="flex gap-1" role="group" aria-label={$t("settings.language")}>
            {#each LANGUAGES as language (language.value)}
              <button
                type="button"
                class="border-border rounded-md border px-3 py-1 text-xs aria-pressed:bg-primary aria-pressed:text-primary-foreground aria-pressed:border-primary"
                aria-pressed={$locale === language.value}
                onclick={() => setLocale(language.value)}
              >
                {language.label}
              </button>
            {/each}
          </div>
        </div>
      </section>

      <section aria-labelledby="settings-track-stats" class="flex flex-col gap-3">
        <h3
          id="settings-track-stats"
          class="text-muted-foreground/80 text-[10px] font-semibold tracking-wider uppercase"
        >
          {$t("settings.trackStats")}
        </h3>

        <div class="flex flex-col gap-1.5">
          <span class="text-xs font-medium">{$t("settings.stop")}</span>
          <div class="flex items-center gap-1.5 text-xs">
            <span class="text-muted-foreground">{$t("settings.lessThan")}</span>
            <Input
              type="number"
              min="1"
              step="1"
              inputmode="decimal"
              class="h-7 w-16 text-right tabular-nums"
              aria-label={$t("settings.stopDistance")}
              data-testid="settings-stop-distance"
              value={shown("stopDistanceM")}
              oninput={(e) => apply("stopDistanceM", e.currentTarget.value)}
              onblur={(e) => (e.currentTarget.value = shown("stopDistanceM"))}
            />
            <span class="text-muted-foreground">{$t("settings.metresIn")}</span>
            <Input
              type="number"
              min="0.5"
              step="0.5"
              inputmode="decimal"
              class="h-7 w-16 text-right tabular-nums"
              aria-label={$t("settings.stopWindow")}
              data-testid="settings-stop-window"
              value={shown("stopWindowS", 60)}
              oninput={(e) => apply("stopWindowS", e.currentTarget.value, 60)}
              onblur={(e) => (e.currentTarget.value = shown("stopWindowS", 60))}
            />
            <span class="text-muted-foreground">{$t("settings.minutes")}</span>
          </div>
          <p class="text-muted-foreground text-[11px] leading-snug">
            {$t("settings.stopHint")}
          </p>
        </div>

        <div class="flex flex-col gap-1.5">
          <span class="text-xs font-medium">{$t("settings.climb")}</span>
          <div class="flex items-center gap-1.5 text-xs">
            <span class="text-muted-foreground">{$t("settings.from")}</span>
            <Input
              type="number"
              min="1"
              step="1"
              inputmode="decimal"
              class="h-7 w-16 text-right tabular-nums"
              aria-label={$t("settings.climbThreshold")}
              data-testid="settings-climb-threshold"
              value={shown("climbThresholdM")}
              oninput={(e) => apply("climbThresholdM", e.currentTarget.value)}
              onblur={(e) => (e.currentTarget.value = shown("climbThresholdM"))}
            />
            <span class="text-muted-foreground">{$t("settings.metres")}</span>
          </div>
          <p class="text-muted-foreground text-[11px] leading-snug">
            {$t("settings.climbHint")}
          </p>
        </div>
      </section>
    </div>
  </Sheet.Content>
</Sheet.Root>
