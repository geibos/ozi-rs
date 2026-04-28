<script lang="ts">
  let {
    symbol = null,
    onSelect,
  }: {
    symbol?: string | null;
    onSelect: (symbol: string | null) => void;
  } = $props();

  let open = $state(false);

  const SYMBOLS = [
    { value: 'flag', emoji: '🏁', label: 'Flag' },
    { value: 'camp', emoji: '🏕️', label: 'Camp' },
    { value: 'danger', emoji: '⚠️', label: 'Danger' },
    { value: 'water', emoji: '💧', label: 'Water' },
    { value: 'shelter', emoji: '🏠', label: 'Shelter' },
    { value: 'meeting-point', emoji: '👥', label: 'Meeting Point' },
    { value: 'start', emoji: '🟢', label: 'Start' },
    { value: 'finish', emoji: '🔴', label: 'Finish' },
    { value: 'viewpoint', emoji: '👁️', label: 'Viewpoint' },
    { value: 'parking', emoji: '🅿️', label: 'Parking' },
  ];

  function getEmoji(val: string | null | undefined): string {
    if (!val) return '📍';
    const found = SYMBOLS.find(s => s.value === val);
    return found ? found.emoji : '📍';
  }

  function handleSelect(val: string | null) {
    onSelect(val);
    open = false;
  }

  // Click outside and Escape handling
  $effect(() => {
    if (!open) return;

    const handleDocumentClick = (e: MouseEvent) => {
      const target = e.target as HTMLElement;
      if (!target.closest('.symbol-picker-container')) {
        open = false;
      }
    };

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        open = false;
      }
    };

    // Use capture to handle it before other elements stop propagation
    document.addEventListener('click', handleDocumentClick, true);
    document.addEventListener('keydown', handleKeyDown, true);

    return () => {
      document.removeEventListener('click', handleDocumentClick, true);
      document.removeEventListener('keydown', handleKeyDown, true);
    };
  });
</script>

<div class="symbol-picker-container">
  <button 
    class="trigger-btn" 
    title={symbol ? `Symbol: ${symbol}` : "Default symbol"}
    onclick={(e) => {
      e.stopPropagation();
      open = !open;
    }}
  >
    {getEmoji(symbol)}
  </button>

  {#if open}
    <div class="popover">
      <div class="grid">
        <button 
          class="grid-item" 
          class:selected={!symbol}
          title="None (Default)"
          onclick={() => handleSelect(null)}
        >
          <span class="emoji">📍</span>
          <span class="label">None</span>
        </button>

        {#each SYMBOLS as s}
          <button 
            class="grid-item" 
            class:selected={symbol === s.value}
            title={s.label}
            onclick={() => handleSelect(s.value)}
          >
            <span class="emoji">{s.emoji}</span>
            <span class="label">{s.label}</span>
          </button>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .symbol-picker-container {
    position: relative;
    display: inline-flex;
    align-items: center;
  }

  .trigger-btn {
    background: transparent;
    border: 1px solid transparent;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    transition: background-color 0.1s, border-color 0.1s;
  }

  .trigger-btn:hover {
    background: var(--ctp-surface0);
    border-color: var(--ctp-surface1);
  }

  .popover {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: 4px;
    background: var(--ctp-mantle);
    border: 1px solid var(--ctp-surface1);
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    padding: 8px;
    z-index: 1000;
    width: 240px;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 4px;
  }

  .grid-item {
    display: flex;
    align-items: center;
    gap: 6px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 4px;
    padding: 4px 8px;
    cursor: pointer;
    text-align: left;
    color: var(--ctp-text);
    transition: background-color 0.1s;
  }

  .grid-item:hover {
    background: var(--ctp-surface0);
  }

  .grid-item.selected {
    background: var(--ctp-surface1);
    border-color: var(--ctp-surface2);
  }

  .emoji {
    font-size: 14px;
    flex-shrink: 0;
  }

  .label {
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
