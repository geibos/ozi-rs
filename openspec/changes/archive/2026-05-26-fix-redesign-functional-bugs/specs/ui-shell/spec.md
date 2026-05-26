## ADDED Requirements

### Requirement: Dev builds expose IPC failures via a structured error toast

In development builds (`import.meta.env.DEV === true`), every IPC call SHALL surface its rejection through a Sonner toast whose root element carries `data-testid="ipc-error"`. The toast SHALL display the IPC command name as its title and the full, JSON-stringified error payload as its description. The toast SHALL be sticky (no auto-dismiss) so the user can read the payload before it disappears.

In production builds the dev toast SHALL NOT render; existing user-facing `toast.error(...)` call sites at IPC call sites continue to render their friendly messages unchanged in both dev and production.

The dev toast SHALL be additive — it stacks alongside any user-facing toast a call site already emits.

#### Scenario: An IPC call rejects in a dev build

- **WHEN** the frontend invokes any Tauri command AND the backend rejects (e.g. payload-shape mismatch, missing field) AND `import.meta.env.DEV` is true
- **THEN** a Sonner toast appears whose root carries `data-testid="ipc-error"` AND whose title equals the IPC command name AND whose description equals the JSON-stringified error payload

#### Scenario: A production build hides the dev toast

- **WHEN** the same IPC rejection occurs in a production build (`import.meta.env.DEV` is false)
- **THEN** no `data-testid="ipc-error"` toast is rendered AND the existing user-facing `toast.error(...)` (if any at the call site) renders unchanged

#### Scenario: The error payload itself is unstringifiable

- **WHEN** an IPC rejection's error object throws when passed to `JSON.stringify` (circular reference, `BigInt`, etc.)
- **THEN** the dev toast description falls back to `String(error)` AND the toast still carries `data-testid="ipc-error"` AND no second uncaught exception is thrown

### Requirement: Workspace status bar reflects in-flight bundle download progress

The workspace status bar element (`data-testid="workspace-status-bar"` in `src/components/WorkspaceShell.svelte`) SHALL render the active bundle's progress text whenever `bundleProgress` in `src/lib/stores.ts` is non-null. The text SHALL include the phase (`scanning` / `downloading` / `extracting` / `indexing`) and a numeric progress hint (completed / total OR downloaded_bytes / total_bytes, whichever is available on the payload), mirroring the text rendered by `BundleLoader.svelte` for the same payload.

When `bundleProgress` is null the status bar SHALL render its idle content (today: empty).

#### Scenario: Download in progress

- **WHEN** a bundle download is in flight AND the backend emits `bundle-progress` with phase `downloading` and `completed: 3, total: 12`
- **THEN** the workspace status bar shows a line containing "downloading" and the "3 / 12" (or equivalent) progress hint within one animation frame of the event arriving

#### Scenario: Status bar clears when the download completes

- **WHEN** the download completes AND the backend emits a final `bundle-progress` or `state-changed` event that resets `bundleProgress` to null
- **THEN** the status bar returns to its idle content with no leftover progress text

#### Scenario: Status bar is shared with the bundle loader, not duplicated

- **WHEN** the bundle loader Sheet is open AND a download is in progress
- **THEN** both the status bar and the loader's own progress region show the same `bundleProgress` payload sourced from the same store; no second listener is registered

### Requirement: "Maps…" trigger ignores re-entry while a refresh is in flight

The "Maps…" button in the Library Rail Maps tab (and any other affordance that triggers `load_projects` from the workspace) SHALL be inert while `busy` in `src/lib/stores.ts` is true. The button SHALL be visibly disabled in that state so the user observes the click had no effect. A second click during an in-flight `load_projects` SHALL NOT send a second IPC.

#### Scenario: Double-click during refresh is a no-op

- **WHEN** the user clicks "Maps…" AND a `load_projects` IPC is already in flight (`$busy === true`) AND the user clicks "Maps…" a second time within 5 seconds
- **THEN** exactly one `load_projects` IPC is sent AND the UI does not freeze AND the button is visibly disabled during the freeze window

#### Scenario: Button re-enables after refresh completes

- **WHEN** the in-flight `load_projects` resolves and `busy` transitions back to false
- **THEN** the "Maps…" button becomes interactive again on the next render

### Requirement: Cmd-K trigger is visually a button, not a text input

The Cmd-K trigger in `src/components/WorkspaceShell.svelte` SHALL be implemented as a `<button type="button">` with the following discipline:

- It SHALL NOT contain or render as an `<input>` element.
- It SHALL NOT use a blinking text caret (`cursor: text` is forbidden on the trigger and its children).
- It SHALL NOT use a focus-ring style that visually replicates input focus (no inset border, no input-like outline).
- It SHALL NOT use placeholder-style copy as its label (e.g. "Search…" with the trailing ellipsis suggesting an input awaiting text). Acceptable labels include "Open command palette", "Command palette", an icon-only affordance, or the `⌘K` chord alone.
- The `⌘K` chord SHALL be rendered as a `<kbd>` element styled as a key-cap badge, visually heavier than text placeholder copy.
- On hover / focus the trigger SHALL use button-style affordances (background tint, ring) matching other buttons in the shell.

Activating the trigger (click, Enter, Space) SHALL open the command palette dialog, whose internal `cmdk` `Command.Input` takes focus automatically.

#### Scenario: Trigger does not look like an input

- **WHEN** the user inspects the workspace context bar in any theme
- **THEN** the Cmd-K trigger does not render a blinking caret AND its computed `cursor` is not `text` AND its label is not "Search…" or any other placeholder-style copy AND its focus ring matches the shell's button-focus style, not an input-focus style

#### Scenario: Trigger opens the palette

- **WHEN** the user clicks the Cmd-K trigger (or presses Enter / Space while it is focused)
- **THEN** the command palette dialog opens AND focus moves into the dialog's internal `Command.Input` AND typing characters there filters the palette results
