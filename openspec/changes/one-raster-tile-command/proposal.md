## Why

`get_ozi_tile` returns an OZF2 tile in the file's own grid, unprojected. It
is registered, has a wrapper in `api.ts` and an answer on the stand, and
nothing calls it: the map draws OZF2 through the `ozi://` protocol, which
asks `get_ozi_tile_projected` for Web Mercator tiles. A command nobody calls
is a command nobody tests, still reachable from the webview, and the tile
requirement in `tile-rendering` names it as if it were part of the contract.

## What Changes

- `get_ozi_tile` goes: the Rust command, its registration, the `api.ts`
  wrapper and the stand's answer.
- The requirement that tiles travel as raw IPC bytes names the two tile
  commands that exist.

## Impact

- Affected specs: `tile-rendering`
- Affected code: `src-tauri/src/commands/tiles.rs`, `src-tauri/src/lib.rs`,
  `src/lib/api.ts`, `src/lib/ipc.ts`, `src/test/stand/tauri-core.ts`
