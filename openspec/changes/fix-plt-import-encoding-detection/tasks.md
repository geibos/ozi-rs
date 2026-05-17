## 1. Validate spec delta

- [ ] 1.1 Confirm the MODIFIED requirement in `specs/track-import/spec.md` carries the broadened SHALL text and all scenarios (cp1251, UTF-8, UTF-8+BOM, UTF-16 LE, ASCII).
- [ ] 1.2 Run `openspec validate fix-plt-import-encoding-detection --strict` and confirm it passes.

## 2. Add dependencies

- [ ] 2.1 Add `encoding_rs` and `chardetng` to `src-tauri/Cargo.toml` under `[dependencies]`.
- [ ] 2.2 Run `cargo check --manifest-path src-tauri/Cargo.toml` to confirm both crates resolve.

## 3. Implement detection

- [ ] 3.1 In `src-tauri/src/infrastructure/import/plt.rs`, factor a `decode_plt_bytes(&[u8]) -> Result<String, PltImportError>` helper.
- [ ] 3.2 Step 1: BOM check (UTF-8 `EF BB BF`, UTF-16 LE `FF FE`, UTF-16 BE `FE FF`) → strip BOM, decode with the corresponding `encoding_rs::Encoding`.
- [ ] 3.3 Step 2: if no BOM, try strict UTF-8 (`std::str::from_utf8`); on success use the result.
- [ ] 3.4 Step 3: feed the bytes to `chardetng::EncodingDetector` with `tld = None`, `allow_utf8 = false`; decode with the returned encoding.
- [ ] 3.5 Step 4: if the detector returns a non-decodable result, fall back to `encoding_rs::WINDOWS_1251`.
- [ ] 3.6 Wire `import_plt_file` to call `decode_plt_bytes`. Keep `import_plt_text(&str)` unchanged so existing structural tests stay valid.
- [ ] 3.7 Confirm pure-ASCII PLT files are unchanged (UTF-8 strict path hits on step 2).

## 4. Add tests

- [ ] 4.1 Add unit test `decode_plt_bytes_cp1251_cyrillic` — bytes for "Поход 2025" in Windows-1251 decode to the exact Russian string with zero U+FFFD.
- [ ] 4.2 Add unit test `decode_plt_bytes_utf8_no_bom` — same string encoded as UTF-8 without BOM decodes correctly via step 2.
- [ ] 4.3 Add unit test `decode_plt_bytes_utf8_with_bom` — UTF-8 BOM is stripped and string decodes correctly.
- [ ] 4.4 Add unit test `decode_plt_bytes_utf16_le_with_bom` — UTF-16 LE BOM file decodes correctly.
- [ ] 4.5 Add unit test `decode_plt_bytes_ascii_unchanged` — ASCII bytes round-trip unchanged.
- [ ] 4.6 Add unit test `import_plt_file_cyrillic_track_name_round_trip` — full path: write a cp1251 PLT to a tempfile, import, assert `Track::name()` equals the original Russian string with no `U+FFFD`.
- [ ] 4.7 Keep all existing ASCII tests in `mod tests` passing unchanged.

## 5. Verify

- [ ] 5.1 Run `just clippy` and resolve any new warnings.
- [ ] 5.2 Run `just test` and confirm all tests pass, including the five encoding-detection tests.
- [ ] 5.3 If implementation reveals scope drift, update `proposal.md` / `tasks.md` and re-run `openspec validate fix-plt-import-encoding-detection --strict`.
