## 1. Frontend

- [x] 1.1 The list: bounded, newest first, upsert by path, empty on rubbish, file name on either platform's separator
- [x] 1.2 Recorded on open, on Save and on Save As; nothing on a failed save
- [x] 1.3 Offered in the palette, re-read each time it opens
- [x] 1.4 A path that fails to open is forgotten, with a message

## 2. Evidence

- [x] 2.1 Seven tests on the list, three on the save wiring
- [x] 2.2 Walked on the stand: the palette shows "Недавние проекты" with the file name above its path

## 3. Gates

- [x] 3.1 `just ci` green
- [ ] 3.2 `just smoke` green (blocked: the Mac2 driver host crashes at session creation — see `docs/STATE.md`)
