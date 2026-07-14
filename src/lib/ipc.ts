/**
 * Dev-toast IPC error reporting shared by the typed command surface and the
 * raw-byte fallback path. Backend rejections surface in dev builds as a
 * sticky Sonner toast carrying `data-testid="ipc-error"` so the next
 * regression of the "frontend sent the wrong payload shape" kind is visible
 * from a screenshot, not buried as `[object Object]`.
 *
 * Two consumers:
 *
 *   - `src/lib/api.ts` — every typed wrapper delegates to the generated
 *     tauri-specta bindings (`src/lib/bindings.ts`) and calls
 *     `reportIpcError` before rethrowing when the `Result` unwrap fails.
 *   - `invokeIpc` below — kept ONLY for the three raw-byte tile commands
 *     (`get_sqlite_tile`, `get_ozi_tile`, `get_ozi_tile_projected`) that are
 *     intentionally not in the generated bindings.
 *
 * Design notes (from `fix-redesign-functional-bugs`):
 *
 *   - The wrapper is additive: existing user-facing `toast.error(...)` calls
 *     at call sites stay; the dev toast stacks alongside them with the raw
 *     payload.
 *   - In production builds the wrapper passes the rejection through
 *     untouched; no dev toast renders.
 *   - The toast root carries a `ipc-error` class. A one-time
 *     `MutationObserver` installed by `installIpcErrorToastObserver()` in
 *     the layout reflects that class onto a `data-testid="ipc-error"`
 *     attribute on the toast `<li>`, because `svelte-sonner` does not
 *     forward arbitrary `data-*` attributes to its rendered list item.
 *   - `JSON.stringify` is wrapped in a try/catch to handle circular
 *     references / BigInt and falls back to `String(error)`. A second
 *     uncaught exception MUST NOT escape.
 */
import {
  invoke,
  type InvokeArgs,
  type InvokeOptions,
} from "@tauri-apps/api/core";
import { toast } from "svelte-sonner";

/**
 * CSS class applied to the dev IPC-error toast. The observer in
 * `installIpcErrorToastObserver` mirrors this onto `data-testid="ipc-error"`
 * on the toast root.
 */
const IPC_ERROR_CLASS = "ipc-error";

function stringifyError(error: unknown): string {
  try {
    if (error === undefined) return "undefined";
    if (error === null) return "null";
    if (typeof error === "string") return error;
    return JSON.stringify(
      error,
      (_key, value) =>
        typeof value === "bigint" ? `${value.toString()}n` : value,
      2,
    );
  } catch {
    try {
      return String(error);
    } catch {
      return "<unstringifiable error>";
    }
  }
}

/**
 * Report an IPC command failure via the dev-only sticky toast. No-op in
 * production builds. Used by `invokeIpc` below and by the `Result`-unwrap
 * path in `src/lib/api.ts` so both transports surface failures identically.
 */
export function reportIpcError(command: string, error: unknown): void {
  if (!import.meta.env.DEV) return;
  try {
    toast.error(command, {
      description: stringifyError(error),
      duration: Number.POSITIVE_INFINITY,
      class: IPC_ERROR_CLASS,
    });
  } catch {
    // A toast that itself throws MUST NOT mask the original rejection.
  }
}

const MAX_SAFE_BIGINT = BigInt(Number.MAX_SAFE_INTEGER);
const MIN_SAFE_BIGINT = BigInt(Number.MIN_SAFE_INTEGER);

function isPlainObject(value: object): boolean {
  const proto = Object.getPrototypeOf(value);
  return proto === Object.prototype || proto === null;
}

/**
 * Safety net for the untyped `invokeIpc` path: Tauri 2 serializes `invoke`
 * args with `JSON.stringify`, which throws
 * `TypeError: Do not know how to serialize a BigInt` — a bigint arg would
 * reject the command before it reached Rust. The typed command surface now
 * goes through the generated tauri-specta bindings (`src/lib/bindings.ts`)
 * and converts bigint IDs explicitly in `api.ts`; `invokeIpc` remains only
 * for the raw-byte tile commands, whose args carry no bigint today. Keep the
 * recursive bigint → number conversion anyway so a future arg change cannot
 * silently reintroduce the rejection. IDs here are small u64 counters; a
 * bigint outside the safe-integer range must never be silently truncated, so
 * it throws a descriptive error instead.
 */
function toIpcSafeValue(value: unknown): unknown {
  if (typeof value === "bigint") {
    if (value > MAX_SAFE_BIGINT || value < MIN_SAFE_BIGINT) {
      throw new Error(
        `IPC argument bigint ${value.toString()} exceeds Number.MAX_SAFE_INTEGER ` +
          "and cannot be losslessly converted to a JSON number",
      );
    }
    return Number(value);
  }
  if (Array.isArray(value)) {
    return value.map(toIpcSafeValue);
  }
  if (typeof value === "object" && value !== null && isPlainObject(value)) {
    return Object.fromEntries(
      Object.entries(value).map(([key, entry]) => [key, toIpcSafeValue(entry)]),
    );
  }
  return value;
}

/**
 * Drop-in wrapper around Tauri's `invoke` that surfaces rejections via the
 * dev IPC-error toast. Call sites do not need to change their `try/catch`
 * blocks — the wrapper rethrows after surfacing.
 *
 * Only the three raw-byte tile commands (`get_sqlite_tile`, `get_ozi_tile`,
 * `get_ozi_tile_projected`) still route through here — they return raw
 * bytes and are intentionally excluded from the generated bindings. All
 * other commands go through `commands.*` in `src/lib/bindings.ts`.
 */
export async function invokeIpc<T>(
  command: string,
  args?: InvokeArgs,
  options?: InvokeOptions,
): Promise<T> {
  try {
    const safeArgs =
      args === undefined ? undefined : (toIpcSafeValue(args) as InvokeArgs);
    // Forward the call shape Tauri's `invoke` was already receiving from
    // `api.ts` — pass `options` only when caller-provided so the existing
    // 2-arg call site signature is preserved for mock-based tests that
    // assert `toHaveBeenCalledWith(cmd, args)` strictly.
    return options === undefined
      ? await invoke<T>(command, safeArgs)
      : await invoke<T>(command, safeArgs, options);
  } catch (error) {
    reportIpcError(command, error);
    throw error;
  }
}

/**
 * Install a one-time `MutationObserver` that mirrors the `ipc-error` class
 * on Sonner toast roots onto a `data-testid="ipc-error"` attribute. Mounted
 * once from the root layout's `onMount`. Safe to call multiple times — only
 * the first invocation installs an observer.
 */
let observerInstalled = false;

export function installIpcErrorToastObserver(): () => void {
  if (typeof document === "undefined") return () => {};
  if (observerInstalled) return () => {};
  observerInstalled = true;

  const markIfTarget = (node: Element) => {
    if (!(node instanceof HTMLElement)) return;
    if (!node.hasAttribute("data-sonner-toast")) return;
    if (!node.classList.contains(IPC_ERROR_CLASS)) return;
    if (node.getAttribute("data-testid") === "ipc-error") return;
    node.setAttribute("data-testid", "ipc-error");
  };

  const scan = (root: ParentNode) => {
    root
      .querySelectorAll<HTMLElement>(`[data-sonner-toast].${IPC_ERROR_CLASS}`)
      .forEach((el) => markIfTarget(el));
  };

  const observer = new MutationObserver((mutations) => {
    for (const m of mutations) {
      m.addedNodes.forEach((node) => {
        if (!(node instanceof Element)) return;
        markIfTarget(node);
        scan(node);
      });
      if (m.type === "attributes" && m.target instanceof Element) {
        markIfTarget(m.target);
      }
    }
  });

  observer.observe(document.body, {
    subtree: true,
    childList: true,
    attributes: true,
    attributeFilter: ["class"],
  });
  // Catch any toast already on screen at install time.
  scan(document.body);

  return () => {
    observer.disconnect();
    observerInstalled = false;
  };
}
