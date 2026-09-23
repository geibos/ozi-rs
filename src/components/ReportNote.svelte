<script lang="ts">
  /**
   * The one line about what happened, asked after the moment was captured.
   *
   * It is the most valuable thing in a report and the hardest to get: by the
   * time somebody writes it down later it has become "the import did something
   * odd", which nobody can act on. Asked here, seconds after, with the screen
   * already safely photographed.
   *
   * Dismissing keeps the report. The folder exists either way.
   */
  import * as Dialog from "$lib/components/ui/dialog";
  import { buttonVariants } from "$lib/components/ui/button";
  import { t } from "$lib/i18n";
  import { describeMoment, reportAwaitingNote } from "$lib/actions/report";

  let note = $state("");

  function close() {
    note = "";
    reportAwaitingNote.set(null);
  }

  async function save() {
    const path = $reportAwaitingNote;
    if (path === null) return;
    const text = note;
    note = "";
    await describeMoment(path, text);
  }
</script>

<Dialog.Root
  open={$reportAwaitingNote !== null}
  onOpenChange={(open) => {
    if (!open) close();
  }}
>
  <Dialog.Content data-testid="report-note">
    <Dialog.Header>
      <Dialog.Title>{$t("report.describe")}</Dialog.Title>
      <Dialog.Description>{$reportAwaitingNote}</Dialog.Description>
    </Dialog.Header>
    <textarea
      bind:value={note}
      rows={3}
      placeholder={$t("report.notePlaceholder")}
      class="border-border w-full rounded-sm border bg-transparent p-2 text-sm"
      data-testid="report-note-input"
      onkeydown={(e: KeyboardEvent) => {
        // Cmd/Ctrl+Enter sends it, because the hands are already on the
        // keyboard and the mouse is on the map.
        if ((e.metaKey || e.ctrlKey) && e.key === "Enter") void save();
      }}
    ></textarea>
    <Dialog.Footer>
      <button
        class={buttonVariants({ variant: "default" })}
        onclick={() => void save()}
        data-testid="report-note-save"
      >
        {$t("report.noteSave")}
      </button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
