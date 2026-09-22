## Why

The app opens in Russian. The one part of it a crew actually watches — the
progress of a bundle download — was English, because those sentences are built
in Rust with `format!` and reach the status bar verbatim. There was no
key-based channel for backend text, so "Downloading 3 files in parallel" and
the phase word next to it sat in an otherwise Russian window for the whole
length of a download.

## What Changes

- Progress messages become a `ProgressText` enum: twelve variants covering
  every message the bundle path emits, each carrying a translation key, its
  arguments in order, and the English wording.
- The `bundle-progress` payload carries `message_key` and `message_args`
  alongside `message`. The English text stays as what diagnostics record and as
  the fallback for a key the interface does not know — a key on screen would be
  worse than English.
- The interface translates the message and the phase word in both status
  surfaces: the cold-start route and the workspace status bar.
- Arguments are positional (`{0}`, `{1}`) and substituted in one pass, so a
  bundle name that itself looks like a placeholder is not substituted into.

## Impact

- Affected specs: `lizaalert-integration`
- Affected code: `src-tauri/src/infrastructure/lizaalert.rs`,
  `src-tauri/src/commands/mod.rs`, `src/lib/types.ts`, `src/lib/i18n.ts`,
  `src/routes/+page.svelte`, `src/components/WorkspaceShell.svelte`,
  `src/test/stand/download-script.ts`
