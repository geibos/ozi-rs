## Why

Real-world PLT files reach our users in **mixed encodings**: legacy OziExplorer exports on Russian Windows are Windows-1251 (cp1251), modern editors and recent OziExplorer builds emit UTF-8 (sometimes with a BOM), and occasional files arrive as UTF-16 LE/BE (Notepad "save as Unicode") or other locale code pages. The current implementation in `src-tauri/src/infrastructure/import/plt.rs` reads the file with `String::from_utf8_lossy()`, which assumes UTF-8 and replaces every non-UTF-8 byte sequence with U+FFFD. That silently corrupts:

- cp1251 Cyrillic track names (the dominant case for LizaAlert SAR volunteers — broken today),
- UTF-16 names (replaced wholesale),
- any other single-byte locale page.

The existing `track-import` spec mentions Windows-1251 only, which is itself too narrow. This change broadens the requirement to auto-detect the encoding (BOM → UTF-8 strict → cp1251 heuristic) and converts to UTF-8 before structural parsing.

## What Changes

- Replace the unconditional `String::from_utf8_lossy()` with an encoding-detection step:
  1. **BOM check** — UTF-8 BOM (`EF BB BF`), UTF-16 LE BOM (`FF FE`), UTF-16 BE BOM (`FE FF`) → decode accordingly.
  2. **Strict UTF-8** — if the byte stream is valid UTF-8 (no decoding errors), use UTF-8.
  3. **Auto-detect** — use `chardetng::EncodingDetector` (Mozilla's encoding detector; same crate Firefox uses) over the body bytes to pick the most likely legacy encoding. If detection lands on a Cyrillic/Latin single-byte page, decode with that.
  4. **Fallback** — Windows-1251 (the historically correct choice for OziExplorer PLT from Russian Windows).
- The detection step SHALL NOT introduce `U+FFFD` for files that have a valid known encoding. Decoder errors for genuinely-corrupt files SHALL be surfaced as a structured import error (not silent replacement).
- Keep the existing PLT structural parser unchanged — only the byte-to-string step changes.
- Add regression tests covering: cp1251 Cyrillic, UTF-8 (no BOM), UTF-8 with BOM, UTF-16 LE with BOM, pure ASCII.

## Impact

- Affected capability: `track-import` (existing "PLT import accepts Windows-1251 encoded text" requirement is broadened to "PLT import accepts mixed encodings via auto-detection").
- Affected code: `src-tauri/src/infrastructure/import/plt.rs` (only the byte-decode step in `import_plt_file` and the test module).
- New runtime dependencies:
  - `encoding_rs` (decoder for all named encodings; MIT/Apache-2.0; already pulled in transitively by `tauri`),
  - `chardetng` (statistical encoding detector; same license; small).
- No frontend changes, no `ProjectCommand` changes, no migration of existing `.ozp` projects.
- Out of scope: PLT export encoding (separate capability `track-export`); GPX/ZIP encoding handling (GPX carries its own XML encoding declaration).
