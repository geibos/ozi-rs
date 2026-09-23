/**
 * The stand's replacement for `@tauri-apps/api/window`.
 *
 * The layout asks the window for a close-request hook so it can warn about
 * unsaved changes. In a browser there is no such window, and until 2026-09-23
 * this answered with a handler that was registered and never fired — so the
 * close guard, which is the half of CJ-7 that stands between a night's work
 * and a shut laptop, could not be looked at here at all. It also answered
 * without `destroy()`, which the guard calls when the operator confirms
 * quitting; the first time the handler ran it would have thrown.
 *
 * `standRequestClose()` fires it, the way the window manager would.
 */

type CloseEvent = { preventDefault: () => void };
type CloseHandler = (event: CloseEvent) => void | Promise<void>;

const handlers = new Set<CloseHandler>();

/** What the window did, in order — `close` and `destroy` as the app asked. */
export const standWindowCalls: string[] = [];

/**
 * Ask the window to close, as clicking the red button does.
 *
 * Answers whether the close was allowed through: `false` means a handler
 * called `preventDefault`, which is the guard holding the window open while it
 * asks about unsaved work.
 */
export async function standRequestClose(): Promise<boolean> {
  let prevented = false;
  const event: CloseEvent = {
    preventDefault: () => {
      prevented = true;
    },
  };
  for (const handler of handlers) await handler(event);
  return !prevented;
}

// On `window`, beside `__stand`, so a stand session can fire a close request
// from the console without importing anything — and so an import that lands in
// a second copy of this module shows up as a disagreement instead of silence.
if (typeof window !== "undefined") {
  (window as unknown as Record<string, unknown>).__standWindow = {
    requestClose: standRequestClose,
    calls: standWindowCalls,
    armed: () => handlers.size,
  };
}

/** How many close guards are armed — "did the layout register?" answered. */
export function standCloseHandlerCount(): number {
  return handlers.size;
}

export function getCurrentWindow() {
  return {
    async onCloseRequested(handler: CloseHandler) {
      handlers.add(handler);
      return () => handlers.delete(handler);
    },
    async close() {
      standWindowCalls.push("close");
    },
    async destroy() {
      standWindowCalls.push("destroy");
    },
    async setTitle(_title: string) {},
  };
}
