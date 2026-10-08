## Why

The last step of processing a search is sending the results to the server:
the owner opens a separate FTP client, copies the search folder's name out
of Finder, finds the folder on the server and drags the files in
(recording of 2026-10-08). ozi-rs has held the results accounts since
`ftp-accounts`, and the files are already where the standard puts them —
`10-Tracks` — but nothing sent them.

## What Changes

- «Отправить на сервер» sends the processed files — every `.plt` and `.wpt`
  directly in the search's `10-Tracks` — to a results account, into the
  search's folder under the account's folder. The folder's name is the
  bundle's, `ГГГГ-ММ-ДД_Место`, which the standard prescribes for both
  (п. 34). Raw recordings in `10-Tracks`' subfolders are not sent (п. 5).
- When the search's folder is not on the server, nothing is sent until the
  operator says to make it, with the standard's reminder that the
  coordinator's word comes first (п. 33).
- A waypoint file holding a `BVP` mark is named before sending: it goes up
  only with the coordinator's sanction (п. 36).
- Files of the same name on the server are replaced — sending a corrected
  track again is the ordinary case. The result lists what was sent and
  where; a failure names the file and the server's words.
- Transfers are binary, in passive mode, each network step limited to 30 s,
  on a blocking thread.

## Impact

- Affected specs: `ftp-accounts`
- Affected code: `src-tauri/src/infrastructure/ftp.rs` (`upload_results`,
  `connect_and_login` shared with the check),
  `src-tauri/src/application/mod.rs` (`results_upload_plan`),
  `src-tauri/src/commands/ftp.rs`, `src-tauri/src/commands/mod.rs`,
  `src-tauri/src/lib.rs`, `src/lib/api.ts`,
  `src/components/ResultsUpload.svelte` (new), `src/lib/i18n.ts`
- Evidence: `docs/field-notes/2026-10-08-forest-search-in-ozi.md`
