<script lang="ts">
  /**
   * The theme selector, as the settings screen shows it.
   *
   * Mounted nowhere from the workspace redesign until 2026-10-02, while
   * `ui-shell` required it to be reachable "from settings" and there were no
   * settings. The operating system's light/dark switch is followed by the
   * listener the root layout installs (`installAutoThemeListener`), not here:
   * this component exists only while the settings screen is open.
   */
  import * as Select from "$lib/components/ui/select";
  import { t } from "$lib/i18n";
  import { catppuccinPackEnabled, selectedTheme } from "$lib/stores";
  import { applyTheme, type ThemeName } from "$lib/theme";

  type ThemeEntry = { value: ThemeName; label: string };

  const NATIVE_THEMES: ThemeEntry[] = [
    { value: "native-auto", label: "Native — Auto" },
    { value: "native-light", label: "Native — Light" },
    { value: "native-dark", label: "Native — Dark" },
  ];

  const CATPPUCCIN_THEMES: ThemeEntry[] = [
    { value: "auto", label: "Catppuccin — Auto" },
    { value: "latte", label: "Catppuccin — Latte" },
    { value: "frappe", label: "Catppuccin — Frappé" },
    { value: "macchiato", label: "Catppuccin — Macchiato" },
    { value: "mocha", label: "Catppuccin — Mocha" },
  ];

  const themes = $derived(
    $catppuccinPackEnabled
      ? [...NATIVE_THEMES, ...CATPPUCCIN_THEMES]
      : NATIVE_THEMES,
  );

  const selectedLabel = $derived(
    themes.find((entry) => entry.value === $selectedTheme)?.label ??
      $selectedTheme,
  );

  $effect(() => {
    void applyTheme($selectedTheme as ThemeName);
  });
</script>

<div class="flex flex-col gap-2">
  <Select.Root type="single" bind:value={$selectedTheme}>
    <Select.Trigger
      aria-label={$t("shell.colourTheme")}
      size="sm"
      class="w-full"
      data-testid="settings-theme"
    >
      {selectedLabel}
    </Select.Trigger>
    <Select.Content>
      {#each themes as entry (entry.value)}
        <Select.Item value={entry.value} label={entry.label}
          >{entry.label}</Select.Item
        >
      {/each}
    </Select.Content>
  </Select.Root>

  <label
    class="text-muted-foreground flex cursor-pointer items-center gap-2 text-xs select-none"
    title={$t("theme.catppuccinPackHint")}
  >
    <input type="checkbox" bind:checked={$catppuccinPackEnabled} />
    <span>{$t("theme.catppuccinPack")}</span>
  </label>
</div>
