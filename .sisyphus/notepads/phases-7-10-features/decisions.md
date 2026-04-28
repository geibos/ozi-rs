# Decisions

## Architecture
- Undo/redo: delta-based (CommandStack stores forward+reverse command pairs), NOT snapshot-based
- Drag coalescing: `apply_or_merge()` on CommandStack — replaces last entry if same entity targeted
- Style mutations (color, visibility, line-width): non-undoable direct mutations, NOT through CommandStack
- Waypoint symbols: `symbol: Option<String>` field, fixed predefined set (flag, camp, danger, etc.)
- Track editing UI: map drag + read-only point list panel (no inline coord editing in list)
- Point list: flat list, no virtualization, first 1000 points per segment with "Show more"
- Print/PDF: deferred, not in this plan

## Format
- PLT export: Windows line endings (\r\n), OLE date (days since 1899-12-30), COLORREF as BGR
- GPX: add `<sym>` element for waypoint symbols

## Testing
- Tests written with implementation, not separate
- Inline `#[cfg(test)]` blocks only
- PLT requires import→export→import round-trip test

## Commit Strategy
- One commit per logical unit per wave
- Pre-commit gate: `cargo test --all && cargo clippy --all-targets --all-features -- -D warnings`

## [2026-04-04] Task: T5
- Implemented `CommandStack` as delta history (`forward` + `reverse` command pairs) with `MAX_STACK_DEPTH = 100`; removed project snapshot-based history.
- Added `ProjectCommand::reverse(&self, project: &Project) -> ProjectCommand` and used it to derive inverse commands from current project state before applying forward commands.
- Added `apply_or_merge(command, project)` to coalesce same-entity command streams by replacing only the latest `forward` command while preserving the original `reverse`.
- Kept redo semantics linear: any newly applied command clears redo history (no branching).

## [2026-04-04] History rewrite cleanup
- Used `git filter-repo` with a message callback instead of interactive rebase so commit order/content stayed intact while only commit messages were rewritten.
- Used `--force-with-lease` (after fetching remote ref) for final push to preserve safer force-push semantics.

## [2026-04-04] Clippy question_mark cleanup
- Removed all remaining `#[allow(clippy::question_mark)]` from `src-tauri/src/commands/mod.rs` and standardized the mutation handlers on `lock_app_state(state.inner())?`.
