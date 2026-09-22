import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  CATALOG_CACHE_KEY,
  loadCatalogWrittenAt,
  markCatalogueRefreshed,
  saveCatalogCache,
} from "../lib/stores";

/**
 * Дата списка должна означать, когда список пришёл с сервера.
 *
 * Внешнее ревью 22.09 (`docs/reviews/2026-09-22/02-architecture-and-patterns.md`)
 * нашло, что подписка на `projectsStore` сохраняет кэш через 800 мс после
 * ЛЮБОГО непустого изменения — включая первичную гидратацию из самого кэша, —
 * и ставит при этом текущее время. То есть запуск без сети «омолаживал»
 * вчерашний список, и строка «от 21.09, 09:12» переставала что-либо значить
 * ровно в том состоянии, ради которого её добавляли.
 */
const ROWS = [{ slug: "2026-09-21_lavrovo", name: "Lavrovo", cached: true }];

describe("дата каталога", () => {
  beforeEach(() => {
    localStorage.removeItem(CATALOG_CACHE_KEY);
  });

  it("сохранение без свежего обхода не двигает дату", () => {
    saveCatalogCache(ROWS, "2026-09-21T06:12:00.000Z");
    saveCatalogCache(ROWS); // перезапись тем же содержимым, обхода не было
    expect(loadCatalogWrittenAt()).toBe("2026-09-21T06:12:00.000Z");
  });

  it("успешный обход двигает дату", () => {
    saveCatalogCache(ROWS, "2026-09-21T06:12:00.000Z");
    markCatalogueRefreshed("2026-09-22T11:00:00.000Z");
    saveCatalogCache(ROWS);
    expect(loadCatalogWrittenAt()).toBe("2026-09-22T11:00:00.000Z");
  });

  it("первая запись без известной даты ставит текущую", async () => {
    // Свежий модуль: отметка об обходе живёт время процесса, и в приложении
    // это верно — один запуск, одна сессия. В тестах её надо обнулить, иначе
    // предыдущий случай отдаёт свою дату этому.
    vi.resetModules();
    localStorage.removeItem(CATALOG_CACHE_KEY);
    const fresh = await import("../lib/stores");
    fresh.saveCatalogCache(ROWS);
    const written = fresh.loadCatalogWrittenAt();
    expect(written).not.toBeNull();
    expect(Date.now() - new Date(written as string).getTime()).toBeLessThan(
      5000,
    );
  });
});
