## 1. The preview

- [x] 1.1 The dialog fetches its own preview, whichever entrance opened it
- [x] 1.2 A tolerance change clears the preview rather than scheduling a second fetch
- [x] 1.3 Walked: opening from the inspector shows «Было: 5 → станет: 3» at once

## 2. The version

- [x] 2.1 Simplify bumps `tracksGeometryVersion`
- [x] 2.2 A test over every component that calls a shape-changing command, verified red
- [x] 2.3 Walked: statistics and segment table now move together, 5 (3+2) → 3 (2+1)

## 3. The stand

- [x] 3.1 Twelve track-editing commands answered, each mutating a session copy of the detail
- [x] 3.2 Row counts derived from the edited detail, so the list agrees with the inspector
- [x] 3.3 A test over the generated bindings for a command with no answer

## 4. Gates

- [x] 4.1 `just ci` green
