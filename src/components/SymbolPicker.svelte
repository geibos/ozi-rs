<script lang="ts">
  import { buttonVariants } from "$lib/components/ui/button";
  import * as Popover from "$lib/components/ui/popover";
  import * as Tooltip from "$lib/components/ui/tooltip";

  let {
    symbol = null,
    onSelect,
  }: {
    symbol?: string | null;
    onSelect: (symbol: string | null) => void;
  } = $props();

  let open = $state(false);

  const SYMBOLS = [
    { value: "flag", emoji: "🏁", label: "Flag" },
    { value: "camp", emoji: "🏕️", label: "Camp" },
    { value: "danger", emoji: "⚠️", label: "Danger" },
    { value: "water", emoji: "💧", label: "Water" },
    { value: "shelter", emoji: "🏠", label: "Shelter" },
    { value: "meeting-point", emoji: "👥", label: "Meeting Point" },
    { value: "start", emoji: "🟢", label: "Start" },
    { value: "finish", emoji: "🔴", label: "Finish" },
    { value: "viewpoint", emoji: "👁️", label: "Viewpoint" },
    { value: "parking", emoji: "🅿️", label: "Parking" },
  ];

  function getEmoji(val: string | null | undefined): string {
    if (!val) return "📍";
    const found = SYMBOLS.find((s) => s.value === val);
    return found ? found.emoji : "📍";
  }

  function handleSelect(val: string | null) {
    onSelect(val);
    open = false;
  }
</script>

<Popover.Root bind:open>
  <Popover.Trigger
    class={buttonVariants({ variant: "ghost", size: "icon-sm" })}
    aria-label={symbol ? `Symbol: ${symbol}` : "Default symbol"}
  >
    <span class="text-sm leading-none">{getEmoji(symbol)}</span>
  </Popover.Trigger>
  <Popover.Content class="w-auto p-2">
    <Tooltip.Provider delayDuration={300}>
      <div class="grid grid-cols-5 gap-2">
        <Tooltip.Root>
          <Tooltip.Trigger
            class={buttonVariants({
              variant: !symbol ? "secondary" : "ghost",
              size: "icon",
            })}
            onclick={() => handleSelect(null)}
            aria-label="None (default)"
          >
            <span class="text-base leading-none">📍</span>
          </Tooltip.Trigger>
          <Tooltip.Content>None</Tooltip.Content>
        </Tooltip.Root>

        {#each SYMBOLS as s (s.value)}
          <Tooltip.Root>
            <Tooltip.Trigger
              class={buttonVariants({
                variant: symbol === s.value ? "secondary" : "ghost",
                size: "icon",
              })}
              onclick={() => handleSelect(s.value)}
              aria-label={s.label}
            >
              <span class="text-base leading-none">{s.emoji}</span>
            </Tooltip.Trigger>
            <Tooltip.Content>{s.label}</Tooltip.Content>
          </Tooltip.Root>
        {/each}
      </div>
    </Tooltip.Provider>
  </Popover.Content>
</Popover.Root>
