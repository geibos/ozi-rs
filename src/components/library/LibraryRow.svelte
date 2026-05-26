<script lang="ts">
  /**
   * Shared row primitive for the Library Tracks and Waypoints tabs.
   *
   * Anatomy (from left to right):
   *   [visibility-toggle] [leadingControl] [name + optional subline] [actions]
   *
   * The component owns inline rename mode on the name cell (`editing` local
   * state) and exposes:
   *   - `onToggleVisibility()` — caller routes the API call.
   *   - `onSelect()` — fires on a row-body click (not on inline controls).
   *   - `onRename(newName)` — fires when Enter or blur commits a non-empty
   *     non-identical name. Pressing Esc discards.
   *   - `leadingControl` snippet — receiver of the swatch (Tracks) or symbol
   *     (Waypoints) cell.
   *   - `actions` snippet — receiver of the `DropdownMenu.Item`s.
   *   - `subline` snippet — optional second line (Tracks distance/duration/pts).
   *   - `nameSuffix` snippet — optional inline-after-name slot (e.g. the
   *     OK-name warning).
   *
   * The visibility toggle, leading control, and actions slot stop event
   * propagation so the body-click `onSelect` does not fire when interacting
   * with them.
   */
  import { type Snippet } from "svelte";
  import EyeIcon from "@lucide/svelte/icons/eye";
  import EyeOffIcon from "@lucide/svelte/icons/eye-off";
  import MoreHorizontalIcon from "@lucide/svelte/icons/more-horizontal";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import * as Tooltip from "$lib/components/ui/tooltip";

  let {
    name,
    visible,
    visibilityLabel,
    selected = false,
    onToggleVisibility,
    onSelect,
    onRename,
    leadingControl,
    actions,
    subline,
    nameSuffix,
  }: {
    name: string;
    visible: boolean;
    visibilityLabel?: string;
    selected?: boolean;
    onToggleVisibility?: () => void;
    onSelect?: () => void;
    onRename?: (newName: string) => void | Promise<void>;
    leadingControl?: Snippet;
    actions?: Snippet;
    subline?: Snippet;
    nameSuffix?: Snippet;
  } = $props();

  let editing = $state(false);
  let editName = $state("");

  function focusOnMount(node: HTMLInputElement) {
    node.focus();
    node.select();
  }

  function startRename() {
    if (!onRename) return;
    editing = true;
    editName = name;
  }

  async function commitRename() {
    if (!onRename) {
      editing = false;
      return;
    }
    const trimmed = editName.trim();
    if (trimmed && trimmed !== name) {
      await onRename(trimmed);
    }
    editing = false;
  }

  function cancelRename() {
    editing = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      void commitRename();
    } else if (e.key === "Escape") {
      e.preventDefault();
      cancelRename();
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="hover:bg-muted flex cursor-pointer items-center gap-1.5 px-2 py-1 transition-colors"
  class:bg-accent={selected}
  class:text-accent-foreground={selected}
  class:opacity-60={!visible}
  onclick={() => onSelect?.()}
>
  <!-- visibility toggle -->
  {#if onToggleVisibility}
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
    <span onclick={(e) => e.stopPropagation()}>
      <Tooltip.Root>
        <Tooltip.Trigger
          class="text-muted-foreground hover:text-foreground inline-flex size-6 items-center justify-center rounded-sm"
          onclick={onToggleVisibility}
          aria-label={visibilityLabel ?? (visible ? `Hide ${name}` : `Show ${name}`)}
        >
          {#if visible}
            <EyeIcon class="size-3.5" />
          {:else}
            <EyeOffIcon class="size-3.5" />
          {/if}
        </Tooltip.Trigger>
        <Tooltip.Content>
          {visible ? "Hide" : "Show"}
        </Tooltip.Content>
      </Tooltip.Root>
    </span>
  {/if}

  <!-- leading control: color swatch (Tracks) / symbol (Waypoints) / cached badge (Maps) -->
  {#if leadingControl}
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
    <span
      class="flex shrink-0 items-center"
      onclick={(e) => e.stopPropagation()}
    >
      {@render leadingControl()}
    </span>
  {/if}

  <!-- name + optional subline -->
  <div class="flex min-w-0 flex-1 flex-col gap-px">
    {#if editing}
      <input
        class="bg-background text-foreground border-input w-full rounded-sm border px-1 py-px text-xs"
        bind:value={editName}
        onblur={commitRename}
        onkeydown={handleKeydown}
        onclick={(e) => e.stopPropagation()}
        use:focusOnMount
      />
      {#if nameSuffix}
        {@render nameSuffix()}
      {/if}
    {:else}
      <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
      <span
        class="cursor-text truncate text-xs"
        title={name}
        ondblclick={startRename}
      >
        {name}
      </span>
      {#if nameSuffix}
        {@render nameSuffix()}
      {/if}
      {#if subline}
        {@render subline()}
      {/if}
    {/if}
  </div>

  <!-- actions menu -->
  {#if actions}
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
    <span
      class="flex shrink-0 items-center"
      onclick={(e) => e.stopPropagation()}
    >
      <DropdownMenu.Root>
        <DropdownMenu.Trigger
          class="text-muted-foreground hover:text-foreground inline-flex size-6 items-center justify-center rounded-sm"
          aria-label="Actions"
        >
          <MoreHorizontalIcon class="size-3.5" />
        </DropdownMenu.Trigger>
        <DropdownMenu.Content>
          {@render actions()}
        </DropdownMenu.Content>
      </DropdownMenu.Root>
    </span>
  {/if}
</div>
