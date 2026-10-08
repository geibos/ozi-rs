<script lang="ts">
  /**
   * «Отправить на сервер»: the processed files of the search's `10-Tracks`
   * to a results account, into the search's folder there (standard п. 34).
   *
   * The owner did this with a separate FTP client, copying the folder's name
   * out of Finder (recording of 2026-10-08). The dialog shows what will go
   * where before anything does, names a waypoint file with the finding in it
   * (п. 36), and asks before making the search's folder on the server, which
   * the standard leaves to the coordinator (п. 33).
   */
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { toast } from "svelte-sonner";
  import {
    getResultsUploadPlan,
    listFtpAccounts,
    uploadResultsFtp,
  } from "$lib/api";
  import type {
    FtpAccountDto,
    FtpUploadDto,
    ResultsUploadPlanDto,
  } from "$lib/bindings";
  import { t, type MessageKey } from "$lib/i18n";
  import { resultsUploadOpen, settingsOpen } from "$lib/stores";

  let plan = $state<ResultsUploadPlanDto | null>(null);
  let accounts = $state<FtpAccountDto[]>([]);
  let chosen = $state<string | null>(null);
  let loaded = $state(false);
  let busy = $state(false);
  /** The server's folder for the search is missing; asking whether to make it. */
  let askFolder = $state<string | null>(null);
  let problem = $state<string | null>(null);

  const account = $derived(accounts.find((a) => a.id === chosen) ?? null);
  const remote = $derived(
    account && plan
      ? `${account.folder.replace(/\/+$/, "")}/${plan.search_folder}`
      : null,
  );

  $effect(() => {
    if (!$resultsUploadOpen) return;
    loaded = false;
    askFolder = null;
    problem = null;
    void Promise.all([getResultsUploadPlan(), listFtpAccounts()])
      .then(([p, list]) => {
        plan = p;
        accounts = list.filter((a) => a.role === "results");
        if (!accounts.some((a) => a.id === chosen)) {
          chosen = accounts[0]?.id ?? null;
        }
      })
      .catch((error) => {
        problem = String(error);
      })
      .finally(() => {
        loaded = true;
      });
  });

  function outcomeText(result: FtpUploadDto): string {
    const key = `resultsUpload.outcome.${result.outcome}` as MessageKey;
    return result.detail ? `${$t(key)}: ${result.detail}` : $t(key);
  }

  async function send(createFolder: boolean) {
    if (!chosen) return;
    busy = true;
    problem = null;
    try {
      const result = await uploadResultsFtp(chosen, createFolder);
      if (result.outcome === "done") {
        toast.success(
          $t("resultsUpload.done").replace(
            "{n}",
            String(result.uploaded.length),
          ),
          { description: result.remote ?? undefined },
        );
        resultsUploadOpen.set(false);
      } else if (result.outcome === "no_search_folder") {
        askFolder = result.remote;
      } else {
        problem = outcomeText(result);
      }
    } catch (error) {
      problem = String(error);
    } finally {
      busy = false;
    }
  }
</script>

<Dialog.Root
  open={$resultsUploadOpen}
  onOpenChange={(open) => resultsUploadOpen.set(open)}
>
  <Dialog.Content data-testid="results-upload" class="sm:max-w-xl">
    <Dialog.Header>
      <Dialog.Title>{$t("resultsUpload.title")}</Dialog.Title>
      <Dialog.Description>{$t("resultsUpload.description")}</Dialog.Description>
    </Dialog.Header>

    {#if !loaded}
      <p class="text-muted-foreground text-sm">{$t("resultsUpload.loading")}</p>
    {:else if !plan}
      <p class="text-sm" data-testid="results-upload-no-search">
        {$t("resultsUpload.noSearch")}
      </p>
    {:else if accounts.length === 0}
      <p class="text-sm" data-testid="results-upload-no-account">
        {$t("resultsUpload.noAccount")}
      </p>
      <Button
        variant="outline"
        size="sm"
        onclick={() => {
          resultsUploadOpen.set(false);
          settingsOpen.set(true);
        }}>{$t("resultsUpload.openSettings")}</Button
      >
    {:else}
      <div class="flex flex-col gap-3 text-sm">
        <label class="flex flex-col gap-1">
          <span class="text-muted-foreground text-xs"
            >{$t("resultsUpload.account")}</span
          >
          <select
            class="border-border bg-background h-8 rounded-md border px-2"
            bind:value={chosen}
            data-testid="results-upload-account"
          >
            {#each accounts as a (a.id)}
              <option value={a.id}>{a.name} — {a.host}</option>
            {/each}
          </select>
        </label>
        <div>
          <div class="text-muted-foreground text-xs">
            {$t("resultsUpload.to")}
          </div>
          <div class="font-mono text-xs" data-testid="results-upload-remote">
            {remote}
          </div>
        </div>
        <div>
          <div class="text-muted-foreground text-xs">
            {$t("resultsUpload.files").replace(
              "{n}",
              String(plan.files.length),
            )}
            <span class="font-mono">{plan.dir}</span>
          </div>
          {#if plan.files.length === 0}
            <p class="text-sm">{$t("resultsUpload.outcome.nothing_to_send")}</p>
          {:else}
            <ul
              class="max-h-40 overflow-y-auto font-mono text-xs"
              data-testid="results-upload-files"
            >
              {#each plan.files as file (file)}
                <li>{file}</li>
              {/each}
            </ul>
          {/if}
        </div>
        {#if plan.with_bvp.length > 0}
          <p class="text-destructive text-xs" data-testid="results-upload-bvp">
            {$t("resultsUpload.bvp").replace(
              "{files}",
              plan.with_bvp.join(", "),
            )}
          </p>
        {/if}
        {#if askFolder}
          <div
            class="border-border rounded-md border p-2 text-xs"
            data-testid="results-upload-ask-folder"
          >
            {$t("resultsUpload.askFolder").replace("{remote}", askFolder)}
          </div>
        {/if}
        {#if problem}
          <p
            class="text-destructive text-xs"
            data-testid="results-upload-problem"
          >
            {problem}
          </p>
        {/if}
      </div>
    {/if}

    <Dialog.Footer class="sm:flex-wrap">
      {#if plan && account && plan.files.length > 0}
        {#if askFolder}
          <Button
            disabled={busy}
            data-testid="results-upload-create"
            onclick={() => void send(true)}
            >{$t("resultsUpload.createAndSend")}</Button
          >
        {:else}
          <Button
            disabled={busy}
            data-testid="results-upload-send"
            onclick={() => void send(false)}>{$t("resultsUpload.send")}</Button
          >
        {/if}
      {/if}
      <Button variant="outline" onclick={() => resultsUploadOpen.set(false)}
        >{$t("resultsUpload.close")}</Button
      >
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
