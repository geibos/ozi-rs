<script lang="ts">
  /**
   * FTP accounts, on the settings screen.
   *
   * Bundles come from one account; results go to several endpoints, each with
   * an account of its own (owner, 2026-10-01). A password is typed here and
   * goes to the operating system's credential store; it never comes back, so
   * an account says only whether one is stored, and an empty password field
   * on an existing account means "keep it".
   */
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";
  import { confirm as confirmDialog } from "@tauri-apps/plugin-dialog";
  import { Input } from "$lib/components/ui/input";
  import { t, type MessageKey } from "$lib/i18n";
  import {
    checkFtpAccount,
    deleteFtpAccount,
    listFtpAccounts,
    saveFtpAccount,
  } from "$lib/api";
  import type { FtpAccountDto, FtpCheckDto, FtpRole } from "$lib/bindings";
  import { ftpErrorText } from "$lib/ftp-errors";

  interface Draft {
    id: string | null;
    role: FtpRole;
    name: string;
    host: string;
    port: string;
    login: string;
    password: string;
    folder: string;
    hasPassword: boolean;
  }

  let accounts = $state<FtpAccountDto[]>([]);
  let loadError = $state<string | null>(null);
  let editing = $state<Draft | null>(null);
  let saving = $state(false);
  let checks = $state<Record<string, FtpCheckDto | "checking">>({});

  const bundles = $derived(accounts.find((a) => a.role === "bundles") ?? null);
  const results = $derived(accounts.filter((a) => a.role === "results"));

  async function reload() {
    try {
      accounts = await listFtpAccounts();
      loadError = null;
    } catch (error) {
      loadError = ftpErrorText(error, $t);
    }
  }

  onMount(() => {
    void reload();
  });

  function startNew(role: FtpRole) {
    const name =
      role === "bundles"
        ? $t("ftp.defaultBundlesName")
        : $t("ftp.defaultResultsName").replace(
            "{n}",
            String(results.length + 1),
          );
    editing = {
      id: null,
      role,
      name,
      host: "",
      port: "21",
      login: "",
      password: "",
      folder: "/",
      hasPassword: false,
    };
  }

  function startEdit(account: FtpAccountDto) {
    editing = {
      id: account.id,
      role: account.role,
      name: account.name,
      host: account.host,
      port: String(account.port),
      login: account.login,
      password: "",
      folder: account.folder,
      hasPassword: account.has_password,
    };
  }

  async function save() {
    if (!editing || saving) return;
    saving = true;
    const draft = editing;
    try {
      // A port that is not a number goes over as 0, which the backend refuses
      // with the message that names the field.
      const port = Number.parseInt(draft.port, 10);
      await saveFtpAccount(
        {
          id: draft.id,
          role: draft.role,
          name: draft.name,
          host: draft.host,
          port: Number.isInteger(port) && port > 0 && port < 65536 ? port : 0,
          login: draft.login,
          folder: draft.folder,
        },
        draft.password === "" ? null : draft.password,
      );
      editing = null;
      toast.success($t("ftp.saved"));
      await reload();
    } catch (error) {
      toast.error($t("ftp.saveFailed"), {
        description: ftpErrorText(error, $t),
      });
    } finally {
      saving = false;
    }
  }

  async function remove(account: FtpAccountDto) {
    const go = await confirmDialog(
      $t("ftp.deleteConfirm").replace("{name}", account.name),
      {
        kind: "warning",
        okLabel: $t("ftp.delete"),
        cancelLabel: $t("ftp.cancel"),
      },
    );
    if (!go) return;
    try {
      await deleteFtpAccount(account.id);
      toast.success($t("ftp.deleted"));
      await reload();
    } catch (error) {
      toast.error($t("ftp.deleteFailed"), {
        description: ftpErrorText(error, $t),
      });
    }
  }

  async function check(account: FtpAccountDto) {
    checks = { ...checks, [account.id]: "checking" };
    try {
      const result = await checkFtpAccount(account.id);
      checks = { ...checks, [account.id]: result };
    } catch (error) {
      const { [account.id]: _dropped, ...rest } = checks;
      checks = rest;
      toast.error($t("ftp.checkFailed"), {
        description: ftpErrorText(error, $t),
      });
    }
  }

  function endpoint(account: FtpAccountDto): string {
    const port = account.port === 21 ? "" : `:${account.port}`;
    return `${account.login}@${account.host}${port}${account.folder}`;
  }

  function checkText(result: FtpCheckDto): string {
    return $t(`ftp.check.${result.outcome}` as MessageKey).replace(
      "{folder}",
      result.folder ?? "",
    );
  }
</script>

{#snippet card(account: FtpAccountDto)}
  {@const result = checks[account.id]}
  <li
    class="border-border flex flex-col gap-1.5 rounded-md border p-2.5"
    data-testid="ftp-account"
  >
    <div class="flex items-baseline justify-between gap-2">
      <span class="truncate text-xs font-medium">{account.name}</span>
      <span class="text-muted-foreground shrink-0 text-[10px]">
        {account.has_password ? $t("ftp.hasPassword") : $t("ftp.noPassword")}
      </span>
    </div>
    <span class="text-muted-foreground truncate font-mono text-[11px]"
      >{endpoint(account)}</span
    >
    {#if result === "checking"}
      <span class="text-muted-foreground text-[11px]">{$t("ftp.checking")}</span
      >
    {:else if result}
      <span
        class="text-[11px] {result.outcome === 'ok'
          ? 'text-emerald-600 dark:text-emerald-400'
          : 'text-destructive'}"
        data-testid="ftp-check-result"
      >
        {checkText(result)}
      </span>
      {#if result.detail}
        <span class="text-muted-foreground truncate font-mono text-[10px]"
          >{result.detail}</span
        >
      {/if}
    {/if}
    <div class="flex gap-1">
      <button
        type="button"
        class="border-border rounded border px-2 py-0.5 text-[11px]"
        disabled={result === "checking"}
        onclick={() => void check(account)}>{$t("ftp.check")}</button
      >
      <button
        type="button"
        class="border-border rounded border px-2 py-0.5 text-[11px]"
        onclick={() => startEdit(account)}>{$t("ftp.edit")}</button
      >
      <button
        type="button"
        class="border-border text-destructive rounded border px-2 py-0.5 text-[11px]"
        onclick={() => void remove(account)}>{$t("ftp.delete")}</button
      >
    </div>
  </li>
{/snippet}

{#snippet form(draft: Draft)}
  <form
    class="border-primary/60 flex flex-col gap-2 rounded-md border p-2.5 text-xs"
    data-testid="ftp-form"
    onsubmit={(event) => {
      event.preventDefault();
      void save();
    }}
  >
    <label class="flex flex-col gap-1">
      <span class="text-muted-foreground">{$t("ftp.name")}</span>
      <Input class="h-7" bind:value={draft.name} aria-label={$t("ftp.name")} />
    </label>
    <div class="grid grid-cols-[1fr_4.5rem] gap-2">
      <label class="flex min-w-0 flex-col gap-1">
        <span class="text-muted-foreground">{$t("ftp.host")}</span>
        <Input
          class="h-7"
          bind:value={draft.host}
          aria-label={$t("ftp.host")}
          autocapitalize="off"
          spellcheck={false}
        />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-muted-foreground">{$t("ftp.port")}</span>
        <Input
          class="h-7 text-right tabular-nums"
          inputmode="numeric"
          bind:value={draft.port}
          aria-label={$t("ftp.port")}
        />
      </label>
    </div>
    <label class="flex flex-col gap-1">
      <span class="text-muted-foreground">{$t("ftp.login")}</span>
      <Input
        class="h-7"
        bind:value={draft.login}
        aria-label={$t("ftp.login")}
        autocapitalize="off"
        autocomplete="off"
        spellcheck={false}
      />
    </label>
    <label class="flex flex-col gap-1">
      <span class="text-muted-foreground">{$t("ftp.password")}</span>
      <Input
        class="h-7"
        type="password"
        autocomplete="new-password"
        bind:value={draft.password}
        aria-label={$t("ftp.password")}
        placeholder={draft.hasPassword ? $t("ftp.passwordKept") : ""}
      />
    </label>
    <label class="flex flex-col gap-1">
      <span class="text-muted-foreground">{$t("ftp.folder")}</span>
      <Input
        class="h-7 font-mono"
        bind:value={draft.folder}
        aria-label={$t("ftp.folder")}
        autocapitalize="off"
        spellcheck={false}
      />
    </label>
    <div class="flex justify-end gap-1">
      <button
        type="button"
        class="border-border rounded border px-2.5 py-1"
        onclick={() => (editing = null)}>{$t("ftp.cancel")}</button
      >
      <button
        type="submit"
        class="bg-primary text-primary-foreground rounded px-2.5 py-1"
        disabled={saving}>{$t("ftp.save")}</button
      >
    </div>
  </form>
{/snippet}

<section aria-labelledby="settings-ftp" class="flex flex-col gap-3">
  <h3
    id="settings-ftp"
    class="text-muted-foreground/80 text-[10px] font-semibold tracking-wider uppercase"
  >
    {$t("ftp.title")}
  </h3>

  {#if loadError}
    <p class="text-destructive text-xs" data-testid="ftp-load-error">
      {loadError}
    </p>
  {:else}
    <div class="flex flex-col gap-1.5">
      <span class="text-xs font-medium">{$t("ftp.bundles")}</span>
      <p class="text-muted-foreground text-[11px] leading-snug">
        {$t("ftp.bundlesHint")}
      </p>
      {#if editing && editing.role === "bundles"}
        {@render form(editing)}
      {:else if bundles}
        <ul class="flex flex-col gap-1.5">{@render card(bundles)}</ul>
      {:else}
        <button
          type="button"
          class="border-border self-start rounded border border-dashed px-2.5 py-1 text-xs"
          onclick={() => startNew("bundles")}>{$t("ftp.add")}</button
        >
      {/if}
    </div>

    <div class="flex flex-col gap-1.5">
      <span class="text-xs font-medium">{$t("ftp.results")}</span>
      <p class="text-muted-foreground text-[11px] leading-snug">
        {$t("ftp.resultsHint")}
      </p>
      <ul class="flex flex-col gap-1.5">
        {#each results as account (account.id)}
          {#if editing && editing.id === account.id}
            <li>{@render form(editing)}</li>
          {:else}
            {@render card(account)}
          {/if}
        {/each}
      </ul>
      {#if editing && editing.role === "results" && editing.id === null}
        {@render form(editing)}
      {:else}
        <button
          type="button"
          class="border-border self-start rounded border border-dashed px-2.5 py-1 text-xs"
          onclick={() => startNew("results")}>{$t("ftp.addResults")}</button
        >
      {/if}
    </div>
  {/if}
</section>
