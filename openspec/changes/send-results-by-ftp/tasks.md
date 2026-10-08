## 1. The transfer

- [x] 1.1 Tests against a scripted server with a data channel: files land in
      the search's folder; a missing folder stops the upload unless asked;
      a wrong password sends nothing
- [x] 1.2 `upload_results` over `connect_and_login`, shared with the check
- [x] 1.3 `results_upload_plan`: the search folder, the processed files, the
      waypoint files with a BVP; test

## 2. The interface

- [x] 2.1 `get_results_upload_plan`, `upload_results_ftp`, bindings,
      commands reference, stand answers
- [x] 2.2 «Отправить на сервер»: the account, the folder, the files, the BVP
      warning, the folder question, the result

## 3. Gates

- [ ] 3.1 `just ci` green
- [x] 3.2 Walked on the stand 2026-10-08 (`?ftp=results`): the dialog showed
      `/results/2026-07-08_Lavrovo` and two files; «Отправить» came back with
      the folder question citing п. 33; «Создать папку и отправить» closed the
      dialog with «Отправлено файлов: 2». Component test for the same and for a
      refused login. `docs/progress/2026-10-08-send-results/`
- [ ] 3.3 Against a real results server with the owner's account — the owner
      has to be there for it (credentials and the coordinator's sanction)
