/**
 * The stand's replacement for `@tauri-apps/api/event`.
 *
 * Nothing emits on the stand: the fixtures are a still frame. `listen`
 * registers the handler and hands back an unsubscribe, so the layout's
 * single-owner event wiring runs exactly as it does in the app.
 */
type Handler<T> = (event: { payload: T }) => void;

const handlers = new Map<string, Set<Handler<unknown>>>();

export async function listen<T>(
  event: string,
  handler: Handler<T>,
): Promise<() => void> {
  const set = handlers.get(event) ?? new Set();
  set.add(handler as Handler<unknown>);
  handlers.set(event, set);
  return () => set.delete(handler as Handler<unknown>);
}

/** Deliver an event by hand, for driving a state from the console. */
export function standEmit<T>(event: string, payload: T): void {
  for (const handler of handlers.get(event) ?? []) {
    handler({ payload });
  }
}
