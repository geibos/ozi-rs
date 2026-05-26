<script lang="ts">
  import * as Select from "$lib/components/ui/select";
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
    themes.find((t) => t.value === $selectedTheme)?.label ?? "Theme",
  );

  $effect(() => {
    void applyTheme($selectedTheme as ThemeName);
  });

  $effect(() => {
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = () => {
      const v = $selectedTheme;
      if (v === "auto" || v === "native-auto") void applyTheme(v as ThemeName);
    };
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  });
</script>

<div class="theme-picker">
  <Select.Root type="single" bind:value={$selectedTheme}>
    <Select.Trigger aria-label="Color theme" size="sm">
      {selectedLabel}
    </Select.Trigger>
    <Select.Content>
      {#each themes as t (t.value)}
        <Select.Item value={t.value} label={t.label}>{t.label}</Select.Item>
      {/each}
    </Select.Content>
  </Select.Root>

  <label class="pack-toggle" title="Enable the Catppuccin theme pack">
    <input type="checkbox" bind:checked={$catppuccinPackEnabled} />
    <span>Catppuccin pack</span>
  </label>
</div>

<style>
  .theme-picker {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .pack-toggle {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: hsl(var(--muted-foreground));
    cursor: pointer;
    user-select: none;
  }
</style>
