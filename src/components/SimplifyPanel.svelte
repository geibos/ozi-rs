<script lang="ts">
  import { simplifyState } from "../lib/stores";
  import { getSimplifiedPreview, simplifyTrack } from "../lib/api";

  let debounceTimer: ReturnType<typeof setTimeout>;

  $effect(() => {
    if (!$simplifyState.active) {
      return;
    }

    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(async () => {
      try {
        const preview = await getSimplifiedPreview(
          $simplifyState.layerId,
          $simplifyState.trackId,
          $simplifyState.tolerance
        );
        simplifyState.update((s) => ({ ...s, preview }));
      } catch (err) {
        console.error("Failed to get simplified preview", err);
      }
    }, 300);
  });

  async function handleConfirm() {
    try {
      await simplifyTrack(
        $simplifyState.layerId,
        $simplifyState.trackId,
        $simplifyState.tolerance
      );
      closePanel();
    } catch (err) {
      console.error("Failed to simplify track", err);
    }
  }

  function closePanel() {
    simplifyState.update((s) => ({ ...s, active: false, preview: null }));
  }
</script>

{#if $simplifyState.active}
  <div class="simplify-panel">
    <div class="header">
      <h3>Simplify Track</h3>
      <button class="close-btn" onclick={closePanel}>×</button>
    </div>

    <div class="content">
      <div class="control-group">
        <label for="tolerance">
          Tolerance: {$simplifyState.tolerance}m
        </label>
        <input
          id="tolerance"
          type="range"
          min="1"
          max="1000"
          step="1"
          bind:value={$simplifyState.tolerance}
        />
      </div>

      {#if $simplifyState.preview}
        <div class="stats">
          <p>
            Original: <strong>{$simplifyState.preview.original_count}</strong> points → Simplified: <strong>{$simplifyState.preview.simplified_count}</strong> points
          </p>
        </div>
      {/if}
    </div>

    <div class="footer">
      <button class="btn btn-secondary" onclick={closePanel}>Cancel</button>
      <button
        class="btn btn-primary"
        disabled={!$simplifyState.preview || $simplifyState.preview.simplified_count === 0}
        onclick={handleConfirm}
      >Confirm</button>
    </div>
  </div>
{/if}

<style>
  .simplify-panel {
    background-color: var(--ctp-surface0);
    border: 1px solid var(--ctp-surface1);
    border-radius: 8px;
    padding: 12px;
    margin-top: 12px;
    color: var(--ctp-text);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }

  .header h3 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--ctp-text);
    font-size: 18px;
    cursor: pointer;
    opacity: 0.7;
  }

  .close-btn:hover {
    opacity: 1;
  }

  .control-group {
    margin-bottom: 12px;
  }

  .control-group label {
    display: block;
    margin-bottom: 4px;
    font-size: 12px;
  }

  .control-group input[type="range"] {
    width: 100%;
  }

  .stats {
    background-color: var(--ctp-mantle);
    padding: 8px;
    border-radius: 4px;
    font-size: 12px;
    margin-bottom: 12px;
  }

  .stats p {
    margin: 0;
    line-height: 1.4;
  }

  .footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .btn {
    padding: 6px 12px;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
    border: none;
  }

  .btn-secondary {
    background-color: var(--ctp-surface2);
    color: var(--ctp-text);
  }

  .btn-secondary:hover {
    background-color: var(--ctp-surface1);
  }

  .btn-primary {
    background-color: var(--ctp-mauve);
    color: var(--ctp-base);
    font-weight: 600;
  }

  .btn-primary:hover {
    opacity: 0.9;
  }
</style>