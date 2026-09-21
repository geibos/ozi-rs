<script lang="ts">
  import { buttonVariants } from "$lib/components/ui/button";
  import { t } from "$lib/i18n";
  import {
    WAYPOINT_SYMBOLS,
    DEFAULT_WAYPOINT_GLYPH,
    waypointGlyph,
    waypointColorCss,
  } from "$lib/waypoint-symbols";
  import * as Popover from "$lib/components/ui/popover";
  import * as Tooltip from "$lib/components/ui/tooltip";

  let {
    symbol = null,
    color = null,
    onSelect,
  }: {
    symbol?: string | null;
    /** Drawn behind the glyph, so the row reads like the marker on the map. */
    color?: [number, number, number, number] | null;
    onSelect: (symbol: string | null) => void;
  } = $props();

  let open = $state(false);

  function handleSelect(val: string | null) {
    onSelect(val);
    open = false;
  }
</script>

<Popover.Root bind:open>
  <Popover.Trigger
    class={`${buttonVariants({ variant: "ghost", size: "icon-sm" })} waypoint-swatch`}
    style={`--waypoint-color: ${waypointColorCss(color)}`}
    aria-label={symbol
      ? $t("symbol.named").replace("{symbol}", symbol)
      : $t("symbol.default")}
  >
    <span class="text-sm leading-none">{waypointGlyph(symbol)}</span>
  </Popover.Trigger>
  <Popover.Content class="w-auto p-2">
    <div class="grid grid-cols-5 gap-2">
      <Tooltip.Root>
        <Tooltip.Trigger
          class={buttonVariants({
            variant: !symbol ? "secondary" : "ghost",
            size: "icon",
          })}
          onclick={() => handleSelect(null)}
          aria-label={$t("symbol.none")}
        >
          <span class="text-base leading-none">{DEFAULT_WAYPOINT_GLYPH}</span>
        </Tooltip.Trigger>
        <Tooltip.Content>{$t("symbol.noneShort")}</Tooltip.Content>
      </Tooltip.Root>

      {#each WAYPOINT_SYMBOLS as s (s.value)}
        <Tooltip.Root>
          <Tooltip.Trigger
            class={buttonVariants({
              variant: symbol === s.value ? "secondary" : "ghost",
              size: "icon",
            })}
            onclick={() => handleSelect(s.value)}
            aria-label={$t(s.labelKey as never)}
          >
            <span class="text-base leading-none">{s.emoji}</span>
          </Tooltip.Trigger>
          <Tooltip.Content>{$t(s.labelKey as never)}</Tooltip.Content>
        </Tooltip.Root>
      {/each}
    </div>
  </Popover.Content>
</Popover.Root>

<style>
  /* The same disc the map draws, so a row and its marker read as one thing. */
  :global(.waypoint-swatch) {
    background: var(--waypoint-color);
    border: 1px solid var(--ctp-crust, #232634);
    border-radius: 50%;
  }
</style>
