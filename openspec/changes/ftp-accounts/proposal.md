## Why

LizaAlert keeps two kinds of FTP. Bundles come from one place under one
account; results go back to several — the owner counts up to four endpoints,
"contours", each with an account of its own. `maps.lizaalert.ru` runs an FTP
service that would give a machine-readable bundle listing instead of scraped
HTML, which is what broke in September when the site changed its markup, and
anonymous login is refused there.

None of that can start until the application knows the hosts and holds the
credentials. The owner decided on 2026-09-21 that credentials are entered by
the operator, never shipped in a build, and that a password belongs in the
operating system's credential store rather than a file; on 2026-10-01 he
asked for the accounts to be built, with bundles and results kept apart.

## What Changes

- An FTP section on the settings screen: one account for bundles, any number
  of result endpoints, each with a name, host, port, login and folder.
- Passwords go to the operating system's credential store — Keychain on
  macOS, Credential Manager on Windows — under an id per account. The
  accounts file in the application's data folder holds everything but the
  password.
- An account can be checked: connect, log in, change to its folder. The
  answer says which step failed, in the operator's language, with the
  server's own words beside it.
- No transfers yet. The bundle listing over FTP and uploading results are the
  next changes and stand on this one.

## Impact

- New capability: `ftp-accounts`
- New dependencies: `keyring` 4.2 (credential stores), `suppaftp` 12.1 (FTP
  control connection; plain FTP, no TLS)
- Affected code: `src-tauri/src/infrastructure/ftp.rs` (new),
  `src-tauri/src/commands/ftp.rs` (new), `src-tauri/src/lib.rs`,
  `src-tauri/src/infrastructure/persistence.rs`, `src/lib/api.ts`,
  `src/components/FtpAccounts.svelte` (new), `src/components/Settings.svelte`,
  `src/lib/i18n.ts`
