<script lang="ts">
  import * as Select from "$lib/components/ui/select";
  import { selectedTheme } from "$lib/stores";
  import { applyTheme, type ThemeName } from "$lib/theme";

  const themes: { value: ThemeName; label: string }[] = [
    { value: "auto", label: "Auto" },
    { value: "latte", label: "Latte" },
    { value: "frappe", label: "Frappé" },
    { value: "macchiato", label: "Macchiato" },
    { value: "mocha", label: "Mocha" },
  ];

  const selectedLabel = $derived(
    themes.find((t) => t.value === $selectedTheme)?.label ?? "Theme",
  );

  $effect(() => {
    applyTheme($selectedTheme as ThemeName);
  });

  $effect(() => {
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = () => {
      if ($selectedTheme === "auto") applyTheme("auto");
    };
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  });
</script>

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
