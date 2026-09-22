## 1. Frontend

- [x] 1.1 The map's point context menu — delete point, insert point after
- [x] 1.2 Inspector rail heading and empty state, console title, symbol picker's "none", theme pack label and hint, collapsed rail placeholders, live-preview label
- [x] 1.3 The two inspector sections' accessible names

## 2. The guard

- [x] 2.1 A test over every component for literal `aria-label` / `title` / `placeholder`
- [x] 2.2 It found the last two offenders on its first run; allowlist carries one entry with its reason
- [x] 2.3 A second rule for a label built in an expression: strip the `$t(…)` / `$i18n(…)` calls, fail on English left in what remains
- [x] 2.4 Verified red by putting `` `Symbol: ${symbol}` `` back

## 3. Gates

- [x] 3.1 `just ci` green
- [x] 3.2 `just smoke` green (2026-09-22, against a bundle built the same hour; the owner granted the Accessibility permission the Mac2 driver needs)
