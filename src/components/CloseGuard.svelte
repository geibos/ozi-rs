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
  import {
    closeGuardOpen,
    closeGuardPurpose,
    resolveCloseGuard,
  } from "$lib/stores";

  // The same three answers for leaving the window and for leaving a search
  // for the next one; the words have to say which, or "Выйти без сохранения"
  // would stand on a dialog that does not quit anything.
  const leavingSearch = $derived($closeGuardPurpose === "newSearch");

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
  <!-- Wider than the stock `max-w-sm`, and the buttons allowed to wrap. The
       root font is 13px, so `sm` is 312px here; three Russian answers need
       about 444, and with `nowrap` buttons the grid grew past its own
       background, text and all. Seen on the stand on 2026-10-02 with the
       next-search wording, whose labels are the longest. -->
  <Dialog.Content data-testid="close-guard" class="sm:max-w-xl">
    <Dialog.Header>
      <Dialog.Title>{$t("closeGuard.title")}</Dialog.Title>
      <Dialog.Description>
        {$t(leavingSearch ? "closeGuard.newSearchMessage" : "closeGuard.message")}
      </Dialog.Description>
    </Dialog.Header>
    <Dialog.Footer class="sm:flex-wrap">
      <button
        class={buttonVariants({ variant: "default" })}
        data-testid="close-guard-save"
        disabled={busy}
        onclick={() => void answer("save")}
      >
        {$t(leavingSearch ? "closeGuard.saveAndContinue" : "closeGuard.saveAndQuit")}
      </button>
      <button
        class={buttonVariants({ variant: "destructive" })}
        data-testid="close-guard-discard"
        onclick={() => void answer("discard")}
      >
        {$t(leavingSearch ? "closeGuard.continueWithoutSaving" : "closeGuard.quit")}
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
