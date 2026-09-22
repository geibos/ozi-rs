import { describe, expect, it, vi } from "vitest";
import { get } from "svelte/store";

const getAppState = vi.hoisted(() => vi.fn());
vi.mock("$lib/api", () => ({ getAppState }));

import { appState } from "../lib/stores";

/**
 * Перекрывающиеся обновления состояния должны заканчиваться самым свежим.
 *
 * Внешнее ревью 22.09: два `refresh()` безусловно звали `set(state)`, поэтому
 * при обратном порядке ответов старое состояние заменяло новое. Во время
 * скачивания бандла `state-changed` приходит раз на файл, так что перекрытие —
 * не редкость, а обычный режим. Тот же штамп поколения, что в
 * `src/lib/latest-run.ts`, только внутри стора.
 */
function state(project: string): unknown {
  return { project_name: project, tracks: [], diagnostics: [] };
}

describe("appState.refresh", () => {
  it("поздний ответ не возвращает старое состояние", async () => {
    let releaseFirst: (v: unknown) => void = () => {};
    getAppState
      .mockImplementationOnce(
        () => new Promise((resolve) => (releaseFirst = resolve)),
      )
      .mockImplementationOnce(async () => state("второй"));

    const first = appState.refresh();
    const second = appState.refresh();
    await second;
    expect((get(appState) as { project_name: string }).project_name).toBe(
      "второй",
    );

    releaseFirst(state("первый"));
    await first;
    expect(
      (get(appState) as { project_name: string }).project_name,
      "ответ обогнанного обновления не должен попадать в стор",
    ).toBe("второй");
  });
});
