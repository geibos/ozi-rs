/**
 * The stand's replacement for `@tauri-apps/api/window`.
 *
 * The layout asks the window for a close-request hook so it can warn about
 * unsaved changes. In a browser there is no such window; the stand answers
 * with an object that registers the handler and never fires it.
 */
export function getCurrentWindow() {
  return {
    onCloseRequested: async () => () => {},
    async close() {},
    async setTitle(_title: string) {},
  };
}
