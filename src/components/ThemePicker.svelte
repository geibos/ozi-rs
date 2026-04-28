<script lang="ts">
  import { selectedTheme } from "../lib/stores";
  import { applyTheme, type ThemeName } from "../lib/theme";

  const themes: { value: ThemeName; label: string }[] = [
    { value: "auto", label: "Auto" },
    { value: "latte", label: "Latte" },
    { value: "frappe", label: "Frappé" },
    { value: "macchiato", label: "Macchiato" },
    { value: "mocha", label: "Mocha" },
  ];

  // Apply on mount and on change
  $effect(() => {
    applyTheme($selectedTheme as ThemeName);
  });

  // Re-apply when system preference changes (for Auto mode)
  $effect(() => {
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = () => {
      if ($selectedTheme === "auto") applyTheme("auto");
    };
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  });
</script>

<select bind:value={$selectedTheme} title="Color theme">
  {#each themes as t}
    <option value={t.value}>{t.label}</option>
  {/each}
</select>
