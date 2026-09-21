/**
 * Minimal typed i18n for the two supported UI languages (CJ-7 slice).
 *
 * Owner decision (2026-07-15): the UI language is switchable (ru/en), the
 * choice persists across sessions. Hand-rolled instead of an i18n framework:
 * two locales, one dictionary module, a typed key union — a library would
 * add deps without adding safety.
 *
 * Usage in components:
 *   import { t } from "$lib/i18n";
 *   <span>{$t("palette.saveProject")}</span>
 *
 * Strings are migrated screen-by-screen as CJ slices touch them; untouched
 * screens keep their hardcoded English until their slice lands.
 */
import { derived, writable, type Readable } from "svelte/store";

export type Locale = "ru" | "en";

const STORAGE_KEY = "ozi:locale";

const dictionaries = {
  en: {
    "palette.saveProject": "Save project…",
    "palette.openProject": "Open project…",
    "palette.undo": "Undo",
    "palette.redo": "Redo",
    "palette.language": "Language: Русский",
    "shell.save": "Save",
    "shell.saved": "Saved",
    "shell.unsaved": "Unsaved changes",
    "shell.undo": "Undo",
    "shell.redo": "Redo",
    "toast.saveFailed": "Failed to save project",
    "toast.saved": "Project saved",
    "toast.undoFailed": "Undo failed",
    "toast.redoFailed": "Redo failed",
    "closeGuard.title": "Unsaved changes",
    "closeGuard.message":
      "The project has unsaved changes. Quit without saving?",
    "closeGuard.quit": "Quit without saving",
    "closeGuard.cancel": "Cancel",
    "trackInspector.simplify": "Simplify",
    "trackInspector.sortByTime": "Sort points by time",
    "trackInspector.sortDone": "Points sorted by time",
    "trackInspector.sortFailed": "Failed to sort points",
    "trackInspector.cropToView": "Crop to map view",
    "trackInspector.cropToViewConfirm":
      "Remove all track points outside the current map view?",
    "trackInspector.cropTitle": "Crop track",
    "trackInspector.crop": "Crop",
    "trackInspector.cancel": "Cancel",
    "trackInspector.cropByTime": "Crop by time range",
    "trackInspector.cropByTimeFrom": "From",
    "trackInspector.cropByTimeTo": "To",
    "trackInspector.cropByTimeNote": "Untimed points are kept.",
    "trackInspector.cropFailed": "Failed to crop track",
    "trackInspector.pointsRemoved": "{count} points removed",
    "points.splitHere": "Split segment here",
    "points.joinPrevious": "Join with previous",
    "points.splitFailed": "Failed to split segment",
    "points.joinFailed": "Failed to join segments",
    "mapsTab.empty": "No maps in this project",
    "mapsTab.emptyHint": "Open a project to see its maps here.",
    "mapsTab.openProject": "Open project…",
    "tracksTab.import": "Import…",
    "tracksTab.searchPlaceholder": "Find a track…",
    "tracksTab.searchClear": "Clear the search",
    "tracksTab.searchCount": "{shown} of {total}",
    "tracksTab.searchEmpty": "No track matches",
    "tracksTab.empty": "No tracks loaded",
    "tracksTab.importFolder": "Import folder…",
    "tracksTab.importFilterName": "Tracks (GPX, PLT, ZIP)",
    "tracksTab.importFailed": "Failed to import tracks",
    "tracksTab.importDone": "Imported {count} of {total} files",
    "tracksTab.importFailedFiles": "Failed: {files}",
    "tracksTab.importFolderFailed": "Failed to import folder",
    "tracksTab.nameHint": "Format: YYYYMMDD_Callsign",
    "track.showOnMap": "Show on map",
    "track.durationTooltip": "Elapsed time from the first to the last point",
    "track.noPoints": "Track has no points",
    "track.showOnMapFailed": "Failed to show track on map",
    "loader.projects": "Projects",
    "loader.maps": "Maps",
    "loader.filterPlaceholder": "Filter…",
    "loader.refreshing": "Refreshing list…",
    "loader.noMatches": "No matches",
    "loader.selectProject": "Select a project on the left",
    "loader.loadingMaps": "Loading map list…",
    "loader.openBundle": "Open bundle (download)",
    "loader.openLocalBundle": "Open local bundle…",
    "loader.setBundlesRoot": "Set bundles root…",
    "loader.cachedBadge": "cached",
    "loader.openMapFailed": "Failed to open map",
    "download.title": "Downloading bundle",
    "download.files": "Files",
    "download.cancel": "Cancel",
    "download.starting": "Starting download…",
  },
  ru: {
    "palette.saveProject": "Сохранить проект…",
    "palette.openProject": "Открыть проект…",
    "palette.undo": "Отменить",
    "palette.redo": "Повторить",
    "palette.language": "Language: English",
    "shell.save": "Сохранить",
    "shell.saved": "Сохранено",
    "shell.unsaved": "Есть несохранённые изменения",
    "shell.undo": "Отменить",
    "shell.redo": "Повторить",
    "toast.saveFailed": "Не удалось сохранить проект",
    "toast.saved": "Проект сохранён",
    "toast.undoFailed": "Не удалось отменить",
    "toast.redoFailed": "Не удалось повторить",
    "closeGuard.title": "Несохранённые изменения",
    "closeGuard.message":
      "В проекте есть несохранённые изменения. Выйти без сохранения?",
    "closeGuard.quit": "Выйти без сохранения",
    "closeGuard.cancel": "Отмена",
    "trackInspector.simplify": "Упростить",
    "trackInspector.sortByTime": "Сортировать точки по времени",
    "trackInspector.sortDone": "Точки отсортированы по времени",
    "trackInspector.sortFailed": "Не удалось отсортировать точки",
    "trackInspector.cropToView": "Обрезать по видимой области",
    "trackInspector.cropToViewConfirm":
      "Удалить все точки трека за пределами видимой области карты?",
    "trackInspector.cropTitle": "Обрезка трека",
    "trackInspector.crop": "Обрезать",
    "trackInspector.cancel": "Отмена",
    "trackInspector.cropByTime": "Обрезать по времени",
    "trackInspector.cropByTimeFrom": "С",
    "trackInspector.cropByTimeTo": "По",
    "trackInspector.cropByTimeNote": "Точки без времени сохраняются.",
    "trackInspector.cropFailed": "Не удалось обрезать трек",
    "trackInspector.pointsRemoved": "Удалено точек: {count}",
    "points.splitHere": "Разрезать сегмент здесь",
    "points.joinPrevious": "Склеить с предыдущим",
    "points.splitFailed": "Не удалось разрезать сегмент",
    "points.joinFailed": "Не удалось склеить сегменты",
    "mapsTab.empty": "В проекте нет карт",
    "mapsTab.emptyHint": "Откройте проект, чтобы увидеть его карты здесь.",
    "mapsTab.openProject": "Открыть проект…",
    "tracksTab.import": "Импорт…",
    "tracksTab.searchPlaceholder": "Найти трек…",
    "tracksTab.searchClear": "Очистить поиск",
    "tracksTab.searchCount": "{shown} из {total}",
    "tracksTab.searchEmpty": "Ничего не найдено",
    "tracksTab.empty": "Треков пока нет",
    "tracksTab.importFolder": "Импорт папки…",
    "tracksTab.importFilterName": "Треки (GPX, PLT, ZIP)",
    "tracksTab.importFailed": "Не удалось импортировать треки",
    "tracksTab.importDone": "Импортировано файлов: {count} из {total}",
    "tracksTab.importFailedFiles": "Не удалось: {files}",
    "tracksTab.importFolderFailed": "Не удалось импортировать папку",
    "tracksTab.nameHint": "Формат: ГГГГММДД_Позывной",
    "track.showOnMap": "Показать на карте",
    "track.durationTooltip": "Время от первой до последней точки",
    "track.noPoints": "В треке нет точек",
    "track.showOnMapFailed": "Не удалось показать трек на карте",
    "loader.projects": "Проекты",
    "loader.maps": "Карты",
    "loader.filterPlaceholder": "Фильтр…",
    "loader.refreshing": "Обновление списка…",
    "loader.noMatches": "Ничего не найдено",
    "loader.selectProject": "Выберите проект слева",
    "loader.loadingMaps": "Загрузка списка карт…",
    "loader.openBundle": "Открыть бандл (скачать)",
    "loader.openLocalBundle": "Открыть локальный бандл…",
    "loader.setBundlesRoot": "Папка для бандлов…",
    "loader.cachedBadge": "скачано",
    "loader.openMapFailed": "Не удалось открыть карту",
    "download.title": "Загрузка бандла",
    "download.files": "Файлы",
    "download.cancel": "Отмена",
    "download.starting": "Начинаем загрузку…",
  },
} as const satisfies Record<Locale, Record<string, string>>;

export type MessageKey = keyof (typeof dictionaries)["en"];

function initialLocale(): Locale {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored === "ru" || stored === "en") return stored;
  } catch {
    // localStorage unavailable (SSR/tests without DOM) — fall through.
  }
  const language =
    typeof navigator !== "undefined" ? (navigator.language ?? "") : "";
  return language.toLowerCase().startsWith("ru") ? "ru" : "en";
}

export const locale = writable<Locale>(initialLocale());

export function setLocale(next: Locale): void {
  try {
    localStorage.setItem(STORAGE_KEY, next);
  } catch {
    // Persistence is best-effort; the in-memory switch still applies.
  }
  locale.set(next);
}

export function toggleLocale(): void {
  locale.update((current) => {
    const next: Locale = current === "ru" ? "en" : "ru";
    try {
      localStorage.setItem(STORAGE_KEY, next);
    } catch {
      // Best-effort, as above.
    }
    return next;
  });
}

/**
 * Reactive translator: `$t("key")`. Falls back to English when a key is
 * missing from the active dictionary (should not happen — the dictionaries
 * share a compile-checked key set).
 */
export const t: Readable<(key: MessageKey) => string> = derived(
  locale,
  (active) => (key: MessageKey) =>
    dictionaries[active][key] ?? dictionaries.en[key],
);
