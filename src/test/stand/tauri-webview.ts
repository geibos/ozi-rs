/**
 * The stand's replacement for `@tauri-apps/api/webview`.
 *
 * The layout asks the webview for drag-and-drop events so a day's files can be
 * dropped on the window. A browser has no such webview; the stand registers
 * the handler and hands back a way to fire it, so the drop path can be walked
 * here rather than only in the packaged application.
 *
 * `standDropFiles(["/searches/day3.gpx"])` from a console session plays a
 * drop, which is the only way to look at what it does to the screen.
 */
type DragDropPayload =
  | { type: "over"; position: { x: number; y: number } }
  | { type: "drop"; paths: string[]; position: { x: number; y: number } }
  | { type: "leave" };

type Handler = (event: { payload: DragDropPayload }) => void;

const handlers: Handler[] = [];

export function getCurrentWebview() {
  return {
    async onDragDropEvent(handler: Handler) {
      handlers.push(handler);
      return () => {
        const index = handlers.indexOf(handler);
        if (index >= 0) handlers.splice(index, 1);
      };
    },
  };
}

/** Play a drop of these paths on the window. */
export function standDropFiles(paths: string[]): void {
  for (const handler of handlers) {
    handler({ payload: { type: "drop", paths, position: { x: 640, y: 400 } } });
  }
}
