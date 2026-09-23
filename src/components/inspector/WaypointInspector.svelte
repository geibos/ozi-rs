<script lang="ts">
  /**
   * Waypoint Inspector — inline editor for the selected waypoint.
   *
   * No dialogs at any point (per locked design). Name / symbol / visibility
   * edits dispatch through the existing `ProjectCommand`-shaped endpoints
   * in `src/lib/api.ts`. Delete also dispatches through `ProjectCommand` —
   * there is intentionally no confirmation dialog here (Undo via Cmd+Z
   * remains available through the project command bus).
   */
  import EyeIcon from "@lucide/svelte/icons/eye";
  import { reportExported } from "$lib/actions/export-result";
  import { reportEditFailure } from "$lib/edit-failure";
  import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
  import PaperclipIcon from "@lucide/svelte/icons/paperclip";
  import { revealPath, setWaypointAttachments } from "$lib/api";
  import EyeOffIcon from "@lucide/svelte/icons/eye-off";
  import FileOutputIcon from "@lucide/svelte/icons/file-output";
  import MapPinIcon from "@lucide/svelte/icons/map-pin";
  import MoveIcon from "@lucide/svelte/icons/move";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Textarea } from "$lib/components/ui/textarea";
  import {
    activeWaypointLayerId,
    appState,
    selectedWaypointId,
  } from "$lib/stores";
  import {
    deleteWaypoint,
    exportGpxWaypoints,
    exportWptWaypoints,
    getWaypoints,
    getWaypointsExportDefaultPath,
    renameWaypoint,
    setWaypointSymbol,
    setWaypointColor,
    setWaypointDescription,
    toggleWaypointVisible,
  } from "$lib/api";
  import { open } from "@tauri-apps/plugin-dialog";
  import { t } from "$lib/i18n";
  import { toast } from "svelte-sonner";
  import type { WaypointData } from "$lib/types";
  import SymbolPicker from "../SymbolPicker.svelte";
  import { waypointColorHex } from "$lib/waypoint-symbols";

  // `$state<T>(...)` rather than an annotated `let`: with the annotation,
  // TypeScript's flow analysis narrows the variable to `null` at any point
  // before the first assignment, which is every inline `$derived` in this
  // file — and then the non-null branch is `never`.
  let waypoint = $state<WaypointData | null>(null);
  let nameDraft = $state("");
  let nameDirty = $state(false);
  /**
   * The note beside the mark, as it is being typed.
   *
   * Its own draft, like the name: a field that writes on every keystroke puts
   * one undo step on the stack per letter.
   */
  let descriptionDraft = $state("");
  let descriptionDirty = $state(false);

  $effect(() => {
    const id = $selectedWaypointId;
    const layerId = $activeWaypointLayerId;
    if (id === null || layerId === null || !$appState) {
      waypoint = null;
      return;
    }
    void loadWaypoint(layerId, id);
  });

  async function loadWaypoint(layerId: bigint, id: bigint) {
    try {
      const all = await getWaypoints(layerId);
      const found = all.find((w) => BigInt(w.id) === id) ?? null;
      waypoint = found;
      if (found && !nameDirty) nameDraft = found.name;
      if (found && !descriptionDirty)
        descriptionDraft = found.description ?? "";
    } catch (error) {
      reportEditFailure("inspector.waypointLoadFailed", error);
      waypoint = null;
    }
  }

  async function commitDescription() {
    const wp = waypoint;
    const layerId = $activeWaypointLayerId;
    if (!wp || layerId === null) return;
    const trimmed = descriptionDraft.trim();
    const current = wp.description ?? "";
    if (trimmed === current) {
      descriptionDirty = false;
      return;
    }
    try {
      // An emptied field is a mark with nothing to say, which is not the same
      // as a mark whose note is an empty string.
      await setWaypointDescription(
        layerId,
        BigInt(wp.id),
        trimmed.length === 0 ? null : trimmed,
      );
      descriptionDirty = false;
      await loadWaypoint(layerId, BigInt(wp.id));
    } catch (error) {
      reportEditFailure("inspector.waypointDescriptionFailed", error);
      descriptionDraft = current;
      descriptionDirty = false;
    }
  }

  /**
   * The files that belong to this mark.
   *
   * A find is photographed from three sides, and the photographs are what the
   * next shift actually looks at. They are stored as paths beside the project
   * rather than inside it — see the Rust side — so attaching one is choosing a
   * file that is already on this machine, not copying it anywhere.
   */
  const attachments = $derived(waypoint?.attachments ?? []);

  async function replaceAttachments(next: string[]) {
    const wp = waypoint;
    const layerId = $activeWaypointLayerId;
    if (!wp || layerId === null) return;
    try {
      await setWaypointAttachments(layerId, BigInt(wp.id), next);
      await loadWaypoint(layerId, BigInt(wp.id));
    } catch (error) {
      reportEditFailure("inspector.attachmentsFailed", error);
    }
  }

  async function handleAttach() {
    const picked = await openFileDialog({ multiple: true });
    if (!picked) return;
    const chosen = Array.isArray(picked) ? picked : [picked];
    await replaceAttachments([...attachments, ...(chosen as string[])]);
  }

  async function handleDetach(path: string) {
    await replaceAttachments(attachments.filter((kept) => kept !== path));
  }

  function fileName(path: string): string {
    const parts = path.split(/[\\/]/);
    return parts[parts.length - 1] || path;
  }

  function handleDescriptionInput(event: Event) {
    descriptionDraft = (event.currentTarget as HTMLTextAreaElement).value;
    descriptionDirty = true;
  }

  async function commitName() {
    const wp = waypoint;
    const layerId = $activeWaypointLayerId;
    if (!wp || layerId === null) return;
    const trimmed = nameDraft.trim();
    if (!trimmed || trimmed === wp.name) {
      nameDraft = wp.name;
      nameDirty = false;
      return;
    }
    try {
      await renameWaypoint(layerId, BigInt(wp.id), trimmed);
      nameDirty = false;
    } catch (error) {
      toast.error($t("inspector.waypointRenameFailed"), {
        description: String(error),
      });
      nameDraft = wp.name;
      nameDirty = false;
    }
  }

  function handleNameInput(event: Event) {
    nameDraft = (event.currentTarget as HTMLInputElement).value;
    nameDirty = true;
  }

  async function handleSetSymbol(symbol: string | null) {
    const wp = waypoint;
    const layerId = $activeWaypointLayerId;
    if (!wp || layerId === null) return;
    try {
      await setWaypointSymbol(layerId, BigInt(wp.id), symbol);
    } catch (error) {
      reportEditFailure("inspector.waypointSymbolFailed", error);
    }
  }

  async function handleToggleVisible() {
    const wp = waypoint;
    const layerId = $activeWaypointLayerId;
    if (!wp || layerId === null) return;
    try {
      await toggleWaypointVisible(layerId, BigInt(wp.id));
    } catch (error) {
      toast.error($t("inspector.waypointVisibilityFailed"), {
        description: String(error),
      });
    }
  }

  async function handleDelete() {
    const wp = waypoint;
    const layerId = $activeWaypointLayerId;
    if (!wp || layerId === null) return;
    try {
      await deleteWaypoint(layerId, BigInt(wp.id));
      selectedWaypointId.set(null);
    } catch (error) {
      toast.error($t("inspector.waypointDeleteFailed"), {
        description: String(error),
      });
    }
  }

  /** GPX for phones and other groups' software, WPT for OziExplorer. */
  async function handleExport(format: "gpx" | "wpt") {
    const layerId = $activeWaypointLayerId;
    if (layerId === null) return;
    try {
      const defaultPath = await getWaypointsExportDefaultPath(layerId, format);
      const path = await open({
        save: true,
        defaultPath: defaultPath ?? `waypoints.${format}`,
        filters:
          format === "gpx"
            ? [{ name: "GPX", extensions: ["gpx"] }]
            : [{ name: "OziExplorer WPT", extensions: ["wpt"] }],
      } as Parameters<typeof open>[0]);
      if (!path) return;
      if (format === "gpx") {
        await exportGpxWaypoints(layerId, path as string);
        reportExported(path as string);
      } else {
        await exportWptWaypoints(layerId, path as string);
        reportExported(path as string);
      }
    } catch (error) {
      toast.error($t("inspector.exportFailed"), {
        description: String(error),
      });
    }
  }

  function handleMoveOnMap() {
    toast.message($t("inspector.moveOnMapHint"), {
      description: $t("inspector.moveOnMapHintDetail"),
    });
  }
  const swatchHex = $derived(waypointColorHex(waypoint?.color));

  async function applyColor(color: [number, number, number, number] | null) {
    const wp = waypoint;
    const layerId = $activeWaypointLayerId;
    if (!wp || layerId === null) return;
    try {
      await setWaypointColor(layerId, BigInt(wp.id), color);
    } catch (error) {
      reportEditFailure("inspector.waypointColorFailed", error);
    }
  }

  function handleColorChange(event: Event) {
    const hex = (event.currentTarget as HTMLInputElement).value;
    const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
    void applyColor([r, g, b, 255]);
  }
</script>

<div class="flex h-full flex-col gap-4 overflow-y-auto p-4">
  <header class="flex items-start gap-3">
    <SymbolPicker
      symbol={waypoint?.symbol}
      color={waypoint?.color}
      onSelect={handleSetSymbol}
    />
    <!-- The symbol says what a mark is; the colour says whose it is. Clearing
         returns it to the default rather than to a colour that looks like it. -->
    <div class="flex shrink-0 flex-col items-center gap-1">
      <input
        class="border-border h-8 w-10 rounded-sm border bg-transparent p-0"
        type="color"
        value={swatchHex}
        aria-label={$t("inspector.waypointColor")}
        data-testid="waypoint-color"
        onchange={handleColorChange}
      />
      {#if waypoint?.color}
        <button
          class="text-muted-foreground hover:text-foreground border-0 bg-transparent p-0 text-[10px]"
          onclick={() => void applyColor(null)}
          data-testid="waypoint-color-reset"
        >
          {$t("inspector.waypointColorDefault")}
        </button>
      {/if}
    </div>
    <div class="min-w-0 flex-1">
      <Input
        type="text"
        value={nameDraft}
        oninput={handleNameInput}
        onblur={commitName}
        onkeydown={(e) => {
          if (e.key === "Enter") (e.currentTarget as HTMLInputElement).blur();
          if (e.key === "Escape") {
            nameDraft = waypoint?.name ?? "";
            nameDirty = false;
            (e.currentTarget as HTMLInputElement).blur();
          }
        }}
        disabled={!waypoint}
        class="h-8 text-sm font-medium"
        aria-label={$t("inspector.waypointName")}
      />
    </div>
    <Button
      variant="ghost"
      size="icon-sm"
      onclick={handleToggleVisible}
      disabled={!waypoint}
      aria-label={waypoint?.visible
        ? $t("inspector.hideWaypoint")
        : $t("inspector.showWaypoint")}
    >
      {#if waypoint?.visible}
        <EyeIcon class="size-4" />
      {:else}
        <EyeOffIcon class="size-4" />
      {/if}
    </Button>
  </header>

  <!-- The note beside the mark. «улика» is the place; «красная куртка, 200 м
       от просеки» is what a crew is sent to, and it travels to the штаб next
       door through GPX and WPT. -->
  <div class="border-border border-b px-2 py-1.5">
    <Textarea
      value={descriptionDraft}
      oninput={handleDescriptionInput}
      onblur={commitDescription}
      onkeydown={(e: KeyboardEvent) => {
        if (e.key === "Escape") {
          descriptionDraft = waypoint?.description ?? "";
          descriptionDirty = false;
          (e.currentTarget as HTMLTextAreaElement).blur();
        }
      }}
      disabled={!waypoint}
      rows={2}
      placeholder={$t("inspector.waypointDescriptionPlaceholder")}
      aria-label={$t("inspector.waypointDescription")}
      class="min-h-0 resize-none text-xs"
      data-testid="waypoint-description"
    />
  </div>

  <!-- The files that belong to this mark: the photographs the next shift
       actually looks at. Paths beside the project, not bytes inside it. -->
  <section
    class="bg-card border-border space-y-2 rounded-[var(--radius-card)] border p-4"
    aria-label={$t("inspector.attachments")}
  >
    <h3
      class="text-muted-foreground/80 flex items-center gap-1.5 text-[10px] font-semibold tracking-wider uppercase"
    >
      <PaperclipIcon class="size-3" strokeWidth={1.5} />
      {$t("inspector.attachments")}
    </h3>

    {#if attachments.length > 0}
      <ul class="space-y-1" data-testid="waypoint-attachments">
        {#each attachments as path (path)}
          <li class="flex items-center gap-1.5 text-xs">
            <span class="flex-1 truncate" title={path}>{fileName(path)}</span>
            <button
              type="button"
              class="text-muted-foreground hover:text-foreground text-[11px]"
              onclick={() => void revealPath(path)}
              aria-label={$t("inspector.attachmentShow")}
            >
              {$t("inspector.attachmentShow")}
            </button>
            <button
              type="button"
              class="text-muted-foreground hover:text-destructive text-[11px]"
              onclick={() => void handleDetach(path)}
              aria-label={$t("inspector.attachmentRemove")}
            >
              {$t("inspector.attachmentRemove")}
            </button>
          </li>
        {/each}
      </ul>
    {:else}
      <p class="text-muted-foreground text-[11px]">
        {$t("inspector.attachmentsNone")}
      </p>
    {/if}

    <button
      type="button"
      class="border-border h-7 w-full rounded-sm border text-xs"
      onclick={() => void handleAttach()}
      disabled={!waypoint}
      data-testid="waypoint-attach"
    >
      {$t("inspector.attachmentAdd")}
    </button>
    <p class="text-muted-foreground text-[10px]">
      {$t("inspector.attachmentsHint")}
    </p>
  </section>

  <section
    class="bg-card border-border rounded-[var(--radius-card)] border p-4"
    aria-label={$t("inspector.location")}
  >
    <h3
      class="text-muted-foreground/80 mb-3 flex items-center gap-1.5 text-[10px] font-semibold tracking-wider uppercase"
    >
      <MapPinIcon class="size-3" />
      {$t("inspector.location")}
    </h3>
    {#if waypoint}
      <dl class="grid grid-cols-[3.5rem_1fr] gap-x-3 gap-y-2 text-xs">
        <dt class="text-muted-foreground">{$t("inspector.latitude")}</dt>
        <dd class="font-mono">{waypoint.lat.toFixed(6)}</dd>
        <dt class="text-muted-foreground">{$t("inspector.longitude")}</dt>
        <dd class="font-mono">{waypoint.lon.toFixed(6)}</dd>
      </dl>
      <Button
        variant="outline"
        size="sm"
        class="mt-3 w-full justify-start gap-2"
        onclick={handleMoveOnMap}
      >
        <MoveIcon class="size-3.5" />
        {$t("inspector.moveOnMap")}
      </Button>
    {:else}
      <p class="text-muted-foreground text-xs">{$t("inspector.noWaypoint")}</p>
    {/if}
  </section>

  <section
    class="flex flex-col gap-2"
    aria-label={$t("inspector.waypointActions")}
  >
    <Button
      variant="outline"
      size="sm"
      class="justify-start gap-2"
      onclick={() => handleExport("gpx")}
      disabled={!waypoint}
    >
      <FileOutputIcon class="size-4" />
      {$t("inspector.exportGpx")}
    </Button>
    <Button
      variant="outline"
      size="sm"
      class="justify-start gap-2"
      onclick={() => handleExport("wpt")}
      disabled={!waypoint}
    >
      <FileOutputIcon class="size-4" />
      {$t("inspector.exportWpt")}
    </Button>
    <Button
      variant="outline"
      size="sm"
      class="text-destructive hover:text-destructive justify-start gap-2"
      onclick={handleDelete}
      disabled={!waypoint}
    >
      <Trash2Icon class="size-4" />
      {$t("inspector.deleteWaypoint")}
    </Button>
  </section>
</div>
