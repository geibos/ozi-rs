## 1. Frontend

- [x] 1.1 The list is a `listbox` with `tabindex`, a label and `aria-activedescendant`; rows are `option`s carrying ids and `aria-selected`
- [x] 1.2 Arrows, Page Up/Down, Home and End move an index into the filtered array, clamped at both ends
- [x] 1.3 The list scrolls to keep the position in view
- [x] 1.4 Enter opens the row; Down from the search box walks in, Enter there opens the first match
- [x] 1.5 Narrowing the list clears the position
- [x] 1.6 The position has an outline of its own, distinct from the selected row

## 2. Tests and evidence

- [x] 2.1 Behavioural tests: pointing, both ends, Enter, Enter before any row, walking in from the search box
- [x] 2.2 Walked on the stand: End reaches the last search, its outline computes to 3px, Enter fires `preview_project` for that slug

## 3. Gates

- [x] 3.1 `just ci` green
- [ ] 3.2 `just smoke` green (blocked: the Mac2 driver host crashes at session creation — see `docs/STATE.md`)
