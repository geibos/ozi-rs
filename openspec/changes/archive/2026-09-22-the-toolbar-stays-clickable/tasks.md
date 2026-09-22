## 1. Frontend

- [x] 1.1 `.canvas-column` declares an explicit shrinkable column
- [x] 1.2 The actions never shrink; the inert chips yield first, then the labels
- [x] 1.3 A guard test pins the declarations, since jsdom computes no layout

## 2. Verification

- [x] 2.1 Measured on the stand with a track selected, at 880, 1024, 1280, 1300 and 1440 px: the bar never exceeds the column (overflow 0 px at every width) and `elementFromPoint` returns each of the four actions
- [x] 2.2 Before the change, at 1024 px: Save at 780..882 against a column ending at 664, and `elementFromPoint` returned `HEADER.rail-header`
- [x] 2.3 `just ci` green
- [x] 2.4 `just smoke` green (2026-09-22, against a bundle built the same hour; the owner granted the Accessibility permission the Mac2 driver needs)
