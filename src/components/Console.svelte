<script lang="ts">
  import { diagnostics, consoleOpen } from "../lib/stores";

  let scrollEl = $state<HTMLDivElement>();

  // Scroll to bottom when new diagnostics arrive
  $effect(() => {
    void $diagnostics; // track reactivity
    if (scrollEl) {
      scrollEl.scrollTop = scrollEl.scrollHeight;
    }
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
  <div class="console">
    <div class="console-header">
      <span>Console</span>
      <button onclick={() => consoleOpen.set(false)}>✕</button>
    </div>
    <div class="console-body" bind:this={scrollEl}>
      {#each $diagnostics as entry}
        <div class="entry {entry.level}">
          <span class="msg">{entry.message}</span>
        </div>
      {/each}
    </div>
  </div>
{/if}

<style>
  .console {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    height: 45vh;
    max-height: 400px;
    background: color-mix(in srgb, var(--ctp-crust) 95%, transparent);
    border-bottom: 1px solid var(--ctp-surface2);
    display: flex;
    flex-direction: column;
    z-index: 1000;
    backdrop-filter: blur(4px);
  }

  .console-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 4px 10px;
    background: var(--ctp-mantle);
    border-bottom: 1px solid var(--ctp-surface0);
    font-size: 11px;
    color: var(--ctp-subtext1);
  }

  .console-header button {
    background: none;
    border: none;
    color: var(--ctp-overlay1);
    padding: 0 4px;
    font-size: 12px;
  }

  .console-body {
    flex: 1;
    overflow-y: auto;
    padding: 4px 10px;
    font-family: "Menlo", "Consolas", monospace;
    font-size: 11px;
  }

  .entry {
    line-height: 1.5;
    white-space: pre-wrap;
  }

  .entry.info .msg {
    color: var(--ctp-text);
  }

  .entry.error .msg {
    color: var(--ctp-red);
  }
</style>
