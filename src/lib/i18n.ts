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
 * Strings are migrated screen-by-screen as slices touch them. Russian is the
 * default locale (owner decision, 2026-07-15), so a screen that still carries
 * hardcoded English is a screen that has not been migrated yet, and reads as a
 * defect rather than a choice.
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
    "rail.maps": "Maps",
    "rail.tracks": "Tracks",
    "rail.waypoints": "Waypoints",
    "shell.modeView": "View",
    "shell.modeDraw": "Draw",
    "shell.modeEdit": "Edit",
    "shell.modeMeasure": "Measure",
    "shell.palette": "Command palette",
    "mapsTab.cached": "cached",
    "shell.language": "Switch to Russian",
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
    "mapsTab.switchFailed": "Failed to switch map",
    "mapsTab.revealFailed": "Failed to reveal the bundle",
    "mapsTab.downloading": "Downloading",
    "mapsTab.cachedTooltip": "Tiles cached locally",
    "mapsTab.switchTo": "Switch to this map",
    "mapsTab.reveal": "Reveal in Finder",
    "mapsTab.empty": "No maps in this project",
    "mapsTab.emptyHint": "Open a project to see its maps here.",
    "mapsTab.openProject": "Open project…",
    "tracksTab.import": "Import…",
    "tracksTab.layer": "Track layer",
    "tracksTab.pickLayer": "Pick a layer",
    "tracksTab.createTrack": "Draw a track",
    "tracksTab.finishTrack": "Finish the track ({count} points)",
    "tracksTab.exportGpx": "Export GPX",
    "tracksTab.exportPlt": "Export PLT",
    "tracksTab.delete": "Delete",
    "tracksTab.loadFailed": "Failed to load tracks",
    "tracksTab.showAll": "Show all tracks",
    "tracksTab.hideAll": "Hide all tracks",
    "tracksTab.onlyThis": "Show only this one",
    "tracksTab.visibilityFailed": "Failed to change visibility",
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
    "waypointsTab.empty": "No waypoints yet",
    "waypointsTab.layer": "Waypoint layer",
    "waypointsTab.pickLayer": "Pick a layer",
    "waypointsTab.add": "Add waypoint",
    "waypointsTab.addCancel": "Stop adding waypoints",
    "waypointsTab.exportWpt": "Export WPT",
    "waypointsTab.delete": "Delete",
    "waypointsTab.hide": "Hide {name}",
    "waypointsTab.show": "Show {name}",
    "inspector.track": "Track",
    "inspector.statistics": "Statistics",
    "inspector.distance": "Distance",
    "inspector.duration": "Duration",
    "inspector.points": "Points",
    "inspector.startTime": "Start time",
    "inspector.noTrack": "No track selected.",
    "inspector.hideTrack": "Hide track",
    "inspector.showTrack": "Show track",
    "inspector.elevation": "Elevation",
    "inspector.elevationSoon": "Elevation chart — coming in a follow-up change",
    "inspector.width": "Width",
    "inspector.lineWidth": "Track line width",
    "inspector.exportGpx": "Export GPX",
    "inspector.exportPlt": "Export PLT",
    "inspector.deleteTrack": "Delete track",
    "inspector.segmentsPoints": "Segments / Points",
    "inspector.waypointName": "Waypoint name",
    "inspector.hideWaypoint": "Hide waypoint",
    "inspector.showWaypoint": "Show waypoint",
    "inspector.location": "Location",
    "inspector.latitude": "Latitude",
    "inspector.longitude": "Longitude",
    "inspector.moveOnMap": "Move on map",
    "inspector.noWaypoint": "No waypoint selected.",
    "inspector.exportWpt": "Export WPT",
    "inspector.deleteWaypoint": "Delete waypoint",
    "inspector.waypointActions": "Waypoint actions",
    "inspector.calibration": "Calibration",
    "inspector.loadingMetadata": "Loading metadata…",
    "inspector.datum": "Datum",
    "inspector.bounds": "Bounds",
    "inspector.resolution": "Resolution",
    "inspector.nativeZoom": "Native zoom",
    "inspector.noCalibration":
      "No OZF2 calibration metadata available (this map may be a SQLite bundle).",
    "inspector.revealInFinder": "Reveal in Finder",
    "waypointsTab.searchPlaceholder": "Find a waypoint…",
    "waypointsTab.searchClear": "Clear the search",
    "waypointsTab.searchCount": "{shown} of {total}",
    "waypointsTab.searchEmpty": "No waypoint matches",
    "waypointsTab.showAll": "Show all waypoints",
    "waypointsTab.hideAll": "Hide all waypoints",
    "waypointsTab.onlyThis": "Show only this one",
    "waypointsTab.visibilityFailed": "Failed to change waypoint visibility",
    "waypointsTab.loadFailed": "Failed to load waypoints",
    "waypointsTab.deleteFailed": "Failed to delete waypoint",
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
    "loader.openBundleBusy": "Wait — the project list is still loading",
    "loader.openBundleFailed": "Could not start the download",
    "loader.previewFailed": "Could not read the project's map list",
    "loader.openLocalBundleFailed": "Could not open the local bundle",
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
    "rail.maps": "Карты",
    "rail.tracks": "Треки",
    "rail.waypoints": "Точки",
    "shell.modeView": "Просмотр",
    "shell.modeDraw": "Рисование",
    "shell.modeEdit": "Правка",
    "shell.modeMeasure": "Измерение",
    "shell.palette": "Команды",
    "mapsTab.cached": "В кэше",
    "shell.language": "Switch to English",
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
    "mapsTab.switchFailed": "Не удалось переключить карту",
    "mapsTab.revealFailed": "Не удалось показать бандл",
    "mapsTab.downloading": "Скачивается",
    "mapsTab.cachedTooltip": "Тайлы есть локально",
    "mapsTab.switchTo": "Переключиться на эту карту",
    "mapsTab.reveal": "Показать в Finder",
    "mapsTab.empty": "В проекте нет карт",
    "mapsTab.emptyHint": "Откройте проект, чтобы увидеть его карты здесь.",
    "mapsTab.openProject": "Открыть проект…",
    "tracksTab.import": "Импорт…",
    "tracksTab.layer": "Слой треков",
    "tracksTab.pickLayer": "Выберите слой",
    "tracksTab.createTrack": "Нарисовать трек",
    "tracksTab.finishTrack": "Завершить трек ({count} точек)",
    "tracksTab.exportGpx": "Экспорт GPX",
    "tracksTab.exportPlt": "Экспорт PLT",
    "tracksTab.delete": "Удалить",
    "tracksTab.loadFailed": "Не удалось загрузить треки",
    "tracksTab.showAll": "Показать все треки",
    "tracksTab.hideAll": "Скрыть все треки",
    "tracksTab.onlyThis": "Показать только этот",
    "tracksTab.visibilityFailed": "Не удалось изменить видимость",
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
    "waypointsTab.empty": "Точек пока нет",
    "waypointsTab.layer": "Слой точек",
    "waypointsTab.pickLayer": "Выберите слой",
    "waypointsTab.add": "Добавить точку",
    "waypointsTab.addCancel": "Закончить добавление",
    "waypointsTab.exportWpt": "Экспорт WPT",
    "waypointsTab.delete": "Удалить",
    "waypointsTab.hide": "Скрыть {name}",
    "waypointsTab.show": "Показать {name}",
    "inspector.track": "Трек",
    "inspector.statistics": "Статистика",
    "inspector.distance": "Длина",
    "inspector.duration": "Длительность",
    "inspector.points": "Точек",
    "inspector.startTime": "Начало",
    "inspector.noTrack": "Трек не выбран.",
    "inspector.hideTrack": "Скрыть трек",
    "inspector.showTrack": "Показать трек",
    "inspector.elevation": "Высота",
    "inspector.elevationSoon": "График высоты — будет в следующем изменении",
    "inspector.width": "Толщина",
    "inspector.lineWidth": "Толщина линии трека",
    "inspector.exportGpx": "Экспорт GPX",
    "inspector.exportPlt": "Экспорт PLT",
    "inspector.deleteTrack": "Удалить трек",
    "inspector.segmentsPoints": "Сегменты и точки",
    "inspector.waypointName": "Название точки",
    "inspector.hideWaypoint": "Скрыть точку",
    "inspector.showWaypoint": "Показать точку",
    "inspector.location": "Координаты",
    "inspector.latitude": "Широта",
    "inspector.longitude": "Долгота",
    "inspector.moveOnMap": "Переместить на карте",
    "inspector.noWaypoint": "Точка не выбрана.",
    "inspector.exportWpt": "Экспорт WPT",
    "inspector.deleteWaypoint": "Удалить точку",
    "inspector.waypointActions": "Действия с точкой",
    "inspector.calibration": "Привязка",
    "inspector.loadingMetadata": "Загрузка метаданных…",
    "inspector.datum": "Датум",
    "inspector.bounds": "Границы",
    "inspector.resolution": "Разрешение",
    "inspector.nativeZoom": "Родной зум",
    "inspector.noCalibration":
      "Нет метаданных привязки OZF2 (возможно, это SQLite-комплект).",
    "inspector.revealInFinder": "Показать в Finder",
    "waypointsTab.searchPlaceholder": "Найти точку…",
    "waypointsTab.searchClear": "Очистить поиск",
    "waypointsTab.searchCount": "{shown} из {total}",
    "waypointsTab.searchEmpty": "Ничего не найдено",
    "waypointsTab.showAll": "Показать все точки",
    "waypointsTab.hideAll": "Скрыть все точки",
    "waypointsTab.onlyThis": "Показать только эту",
    "waypointsTab.visibilityFailed": "Не удалось изменить видимость точек",
    "waypointsTab.loadFailed": "Не удалось загрузить точки",
    "waypointsTab.deleteFailed": "Не удалось удалить точку",
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
    "loader.openBundleBusy": "Подождите — список проектов ещё грузится",
    "loader.openBundleFailed": "Не удалось начать загрузку",
    "loader.previewFailed": "Не удалось прочитать список карт проекта",
    "loader.openLocalBundleFailed": "Не удалось открыть локальный бандл",
    "loader.openMapFailed": "Не удалось открыть карту",
    "download.title": "Загрузка бандла",
    "download.files": "Файлы",
    "download.cancel": "Отмена",
    "download.starting": "Начинаем загрузку…",
  },
} as const satisfies Record<Locale, Record<string, string>>;

export type MessageKey = keyof (typeof dictionaries)["en"];

/// Russian is the default, whatever the operating system reports.
///
/// The users are a Russian-speaking search-and-rescue crew, and the machine's
/// language says nothing about that — the owner runs an English system. Only
/// an explicit choice, remembered from last time, overrides it.
function initialLocale(): Locale {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored === "ru" || stored === "en") return stored;
  } catch {
    // localStorage unavailable (SSR/tests without DOM) — fall through.
  }
  return "ru";
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

/**
 * The keys one dictionary defines. Exported for the test that holds the two
 * dictionaries to the same key set — a missing key falls back to English at
 * runtime, so nothing else would notice a half-finished translation pass.
 */
export function dictionaryKeys(locale: Locale): MessageKey[] {
  return Object.keys(dictionaries[locale]) as MessageKey[];
}
