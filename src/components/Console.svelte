<script lang="ts">
  import XIcon from "@lucide/svelte/icons/x";
  import { Card } from "$lib/components/ui/card";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Button } from "$lib/components/ui/button";
  import { diagnostics, consoleOpen } from "$lib/stores";

  let viewportEl = $state<HTMLElement | null>(null);

  $effect(() => {
    void $diagnostics;
    if (viewportEl) viewportEl.scrollTop = viewportEl.scrollHeight;
  });

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "`" || e.key === "Dead") {
      consoleOpen.update((v) => !v);
    }
    if (e.key === "Escape" && $consoleOpen) {
      consoleOpen.set(false);
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

{#if $consoleOpen}
  <Card
    class="bg-popover/95 fixed inset-x-0 top-0 z-50 flex h-[45vh] max-h-[400px] flex-col gap-0 rounded-none border-0 border-b py-0 ring-0 backdrop-blur"
  >
    <div
      class="bg-card text-muted-foreground border-border flex items-center justify-between border-b px-2.5 py-1 text-xs"
    >
      <span>Console</span>
      <Button
        variant="ghost"
        size="icon-xs"
        onclick={() => consoleOpen.set(false)}
        aria-label="Close console"
      >
        <XIcon />
      </Button>
    </div>
    <ScrollArea class="flex-1" bind:viewportRef={viewportEl}>
      <div class="px-2.5 py-1 font-mono text-[11px] leading-relaxed">
        {#each $diagnostics as entry}
          <div
            class="whitespace-pre-wrap"
            class:text-foreground={entry.level === "info"}
            class:text-destructive={entry.level === "error"}
          >
            {entry.message}
          </div>
        {/each}
      </div>
    </ScrollArea>
  </Card>
{/if}
