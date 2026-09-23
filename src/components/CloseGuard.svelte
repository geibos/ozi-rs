<script lang="ts">
  /**
   * What the application asks before a window with unsaved work closes.
   *
   * It used to ask through the operating system's confirm box, which offers
   * two buttons. So the question was «Выйти без сохранения?» and the answers
   * were "quit" and "cancel" — and the thing the operator almost always wants,
   * to save and then quit, was not among them. They had to cancel, find
   * Cmd+S, and close again, at the one moment in the session when they are
   * already leaving and least likely to be careful.
   *
   * Three answers need a dialog of our own. This is it: the safe answer first,
   * the destructive one plainly named, and Escape or the overlay meaning
   * "stay", which is what a dismissed dialog should always mean when the
   * alternative is losing a night's work.
   */
  import * as Dialog from "$lib/components/ui/dialog";
  import { buttonVariants } from "$lib/components/ui/button";
  import { t } from "$lib/i18n";
  import { closeGuardOpen, resolveCloseGuard } from "$lib/stores";

  let busy = $state(false);

  async function answer(choice: "save" | "discard" | "stay") {
    if (busy) return;
    busy = choice === "save";
    resolveCloseGuard(choice);
    busy = false;
  }
</script>

<Dialog.Root
  open={$closeGuardOpen}
  onOpenChange={(open) => {
    // Dismissing — Escape, the overlay, the corner cross — means stay. The
    // other reading would quit without saving on a stray keystroke.
    if (!open && $closeGuardOpen) void answer("stay");
  }}
>
  <Dialog.Content data-testid="close-guard">
    <Dialog.Header>
      <Dialog.Title>{$t("closeGuard.title")}</Dialog.Title>
      <Dialog.Description>{$t("closeGuard.message")}</Dialog.Description>
    </Dialog.Header>
    <Dialog.Footer>
      <button
        class={buttonVariants({ variant: "default" })}
        data-testid="close-guard-save"
        disabled={busy}
        onclick={() => void answer("save")}
      >
        {$t("closeGuard.saveAndQuit")}
      </button>
      <button
        class={buttonVariants({ variant: "destructive" })}
        data-testid="close-guard-discard"
        onclick={() => void answer("discard")}
      >
        {$t("closeGuard.quit")}
      </button>
      <button
        class={buttonVariants({ variant: "outline" })}
        data-testid="close-guard-stay"
        onclick={() => void answer("stay")}
      >
        {$t("closeGuard.cancel")}
      </button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
