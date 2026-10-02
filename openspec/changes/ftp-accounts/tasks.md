## 1. Accounts and passwords

- [x] 1.1 `infrastructure/ftp.rs`: the account model, the accounts file (atomic write), validation, one bundle account
- [x] 1.2 A credential-store seam: the system store in the application, memory in tests
- [x] 1.3 Tests: save, list, update without a password, delete, restart, four endpoints, second bundle account refused, validation, no password on disk, a damaged file refused

## 2. Checking an account

- [x] 2.1 Connect, log in, change folder, with read and write time limits; the failing step named
- [x] 2.2 Tests against a scripted server on localhost: success, 530, 550, nothing listening, a silent server

## 3. Commands and the screen

- [x] 3.1 `list_ftp_accounts`, `save_ftp_account`, `delete_ftp_account`, `check_ftp_account`; bindings regenerated
- [x] 3.2 `FtpAccounts.svelte` on the settings screen: bundles and results, add, edit, delete, check
- [x] 3.3 Both dictionaries; the stand answers the four commands
- [x] 3.4 Component test: the form saves what is typed, an empty password field sends none

## 4. Gates

- [x] 4.1 `just ci` green (2026-10-02: 423 Rust, 718 vitest; clippy --all-targets clean; cargo audit: no new advisories)
- [x] 4.2 Walked on the stand 2026-10-02: a bundle account and a result endpoint added; `ftp://contour1.example.org/` saved as the server name with port 2121 and folder `/incoming`; the check reported «Вход выполнен, папка /» for the one with a password and «Для этой учётной записи не сохранён пароль» for the one without. `docs/progress/2026-10-02-ftp-accounts/`
- [ ] 4.3 The real keychain: an account saved in the packaged application, its password visible in Keychain Access and not in the file
- [ ] 4.4 `just smoke` green on a bundle built from this tree
