<script lang="ts">
  import XIcon from "@lucide/svelte/icons/x";
  import { toast } from "svelte-sonner";
  import { Button } from "$lib/components/ui/button";
  import { Label } from "$lib/components/ui/label";
  import { Slider } from "$lib/components/ui/slider";
  import { Switch } from "$lib/components/ui/switch";
  import { simplifyState } from "$lib/stores";
  import { getSimplifiedPreview, simplifyTrack } from "$lib/api";

  let debounceTimer: ReturnType<typeof setTimeout>;
  let livePreview = $state(true);

  $effect(() => {
    if (!$simplifyState.active) {
      return;
    }
    if (!livePreview) {
      simplifyState.update((s) => ({ ...s, preview: null }));
      return;
    }

    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(async () => {
      try {
        const preview = await getSimplifiedPreview(
          $simplifyState.layerId,
          $simplifyState.trackId,
          $simplifyState.tolerance,
        );
        simplifyState.update((s) => ({ ...s, preview }));
      } catch (err) {
        console.error("Failed to get simplified preview", err);
        toast.error("Failed to get simplified preview", {
          description: String(err),
        });
      }
    }, 300);
  });

  async function handleConfirm() {
    try {
      await simplifyTrack(
        $simplifyState.layerId,
        $simplifyState.trackId,
        $simplifyState.tolerance,
      );
      closePanel();
    } catch (err) {
      console.error("Failed to simplify track", err);
      toast.error("Failed to simplify track", { description: String(err) });
    }
  }

  function closePanel() {
    simplifyState.update((s) => ({ ...s, active: false, preview: null }));
  }
</script>

{#if $simplifyState.active}
  <div
    class="bg-card text-card-foreground border-border mt-3 flex flex-col gap-3 rounded-lg border p-3"
  >
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-semibold">Simplify Track</h3>
      <Button
        variant="ghost"
        size="icon-xs"
        onclick={closePanel}
        aria-label="Close simplify panel"
      >
        <XIcon />
      </Button>
    </div>

    <div class="flex flex-col gap-2">
      <Label for="simplify-tolerance" class="text-xs">
        Tolerance: {$simplifyState.tolerance}m
      </Label>
      <Slider
        id="simplify-tolerance"
        type="single"
        min={1}
        max={1000}
        step={1}
        bind:value={$simplifyState.tolerance}
      />
    </div>

    <div class="flex items-center gap-2">
      <Switch id="simplify-live-preview" bind:checked={livePreview} />
      <Label for="simplify-live-preview" class="text-xs">Live preview</Label>
    </div>

    {#if $simplifyState.preview}
      <div class="bg-muted text-muted-foreground rounded-md p-2 text-xs leading-snug">
        Original:
        <strong class="text-foreground">{$simplifyState.preview.original_count}</strong>
        points → Simplified:
        <strong class="text-foreground">{$simplifyState.preview.simplified_count}</strong>
        points
      </div>
    {/if}

    <div class="flex justify-end gap-2">
      <Button variant="outline" size="sm" onclick={closePanel}>Cancel</Button>
      <Button
        size="sm"
        disabled={!$simplifyState.preview ||
          $simplifyState.preview.simplified_count === 0}
        onclick={handleConfirm}
      >
        Confirm
      </Button>
    </div>
  </div>
{/if}
