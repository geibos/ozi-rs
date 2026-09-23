# OziExplorer — референс по оригиналу

> Назначение документа. ozi-rs — это переосмысление OziExplorer под узкую задачу
> (картограф/координатор ПСО «ЛизаАлерт»). Чтобы осознанно решать, **какую основную
> функциональность мы берём и как делаем её удобнее, а что осознанно не берём**, нужно
> понимать суть оригинала. Этот файл — карта оригинального продукта: что он делает,
> как устроена его модель данных и — главное для нас — точные форматы его файлов.
>
> Это описание **прототипа**, а не спецификация ozi-rs. Инженерная спецификация
> нашего проекта — в [`requirements.md`](../requirements.md), [`glossary.md`](../glossary.md),
> [`architecture.md`](../architecture.md) и ADR. Технические термины, имена форматов и
> полей приводятся в оригинале (английский).

## Как получены эти данные

Источник — официальный установщик `oziexp_setup.exe` (Inno Setup 5.5.3, OziExplorer
**3.95**, D & L Software Pty Ltd) и встроенное руководство. Ничего проприетарного в репозиторий
не копируется; ниже — только описание функциональности и публично задокументированных
автором форматов, плюс демонстрационные строки из поставляемых демо-данных как иллюстрация.

Воспроизвести распаковку (macOS):

```sh
brew install innoextract          # распаковщик Inno Setup
innoextract -e -d ./out oziexp_setup.exe        # → out/app/*
7z x ./out/app/OziExplorer.chm -o ./out/chm     # руководство → 181 HTML-страница
```

В дистрибутиве: `OziExp.exe` (главное приложение, ~2.5 МБ, Delphi/Win32, ресурсы упакованы),
`OziExpTrial.exe` (trial), `OziExplorer.chm` (руководство), набор DLL картографических
библиотек (`NCSEcw`/`NCScnet`/`NCSUtil` — ECW; `OziMrSid` — MrSID; `geotiff`/`libtiff` —
GeoTIFF; `proj447` — PROJ 4.4.7 проекции; `mgrs` — военная сетка; `LPng`/`jpeg_osgeo` —
PNG/JPEG; `w8gps` — связь с GPS), демо-данные (`Data/`, `Maps/`), символы (`Symbols/`,
`PSymbols/`), звуки навигации, 12 языковых пакетов `.ozl` (включая `russian.ozl`).

## Что такое OziExplorer

Классическая desktop-программа для работы с **растровыми (отсканированными) картами** и
GPS: калибровка карты (привязка изображения к координатам), нанесение на неё объектов
(точки, треки, маршруты), обмен данными с GPS-навигаторами и режим **Moving Map** —
отображение своего положения в реальном времени поверх карты по данным GPS. Исторически —
shareware одного автора (Des Newman), развивавшийся итеративно с конца 1990-х.

Современное семейство (сайт [oziexplorer4.com](https://www.oziexplorer4.com/)):
**OziExplorer** (Windows desktop), **OziExplorerCE** (Windows CE/Mobile), **OziExplorer for
Android**. Формат карт эволюционировал: `OZF2` → `OZFx3` → `OZF4` (текущий; создаётся
внешним конвертером **Img2ozf**; `OZF2` больше не создаётся, только читается).

### Ментальная модель

- **Карта = пара файлов**: изображение (`.ozf2/.ozfx3/.ozf4`, либо `.ecw/.jp2/.tif/.png/
  .jpg/.bmp`) **+** файл калибровки `.map` (ASCII), который привязывает пиксели к
  координатам, задаёт datum и проекцию.
- **Поверх карты — «Map Objects»**: Waypoints (точки), Tracks (треки), Routes (маршруты),
  Events (события), Points (наборы точек), Map Features и Map Comments (аннотации на карте).
  Каждый тип объектов хранится в отдельном текстовом файле.
- **Координаты** всегда в **decimal degrees** (отрицательные = южная широта / западная
  долгота). У каждого файла объектов в шапке указан свой **datum**.
- **Два режима работы**: статичная работа с картой (калибровка, редактирование объектов,
  измерения, печать) и **Moving Map** (реальное время: GPS-позиция, тревоги, AIS, автопилот).

## Карта функциональности

Полное руководство — 168 разделов. Ниже — сгруппированный обзор с пометкой релевантности
для ozi-rs: **[ЯДРО]** берём, **[позже]** периферия/улучшение на будущее, **[не берём]**
вне scope нашего продукта (см. [ADR-0020](../adr/adr-0020-mvp-scope.md), раздел
«К чему стремимся» ниже).

- **Карты**
  - Calibrating Maps (калибровка по 1–30 точкам) — **[ЯДРО]** ✅ с 23.09.2026: импорт готового `.map` и привязка картинки, у которой его нет (два угла в форме, бэкенд принимает любое число точек)
  - Image Formats Supported (BMP/TIF/PNG/JPG/ECW/SID + OZF2/x3/4) — **[ЯДРО]** ✅ OZF2, JPEG, PNG, TIFF, BMP, GIF, WebP. ECW/SID и `.ozfx3` — нет: первые два проприетарные, третий зашифрован
  - Map Projections (14 проекций/грид-систем), France Grids — **[позже]** (репроецирование — non-goal)
  - Import: DRG, GeoTIFF, BSB, NOS/GEO, NV.Digital, Maptech PCX/RML, QuoVadis, ECW, SID, Kompass — **[не берём]**
  - Map Searching: Index Map, Name Search, Find Map, Blank Map — **[позже]**
  - Seamless Maps (много карт как одна) — **[позже]**
  - Save Map to Image, Magnetic Variation — **[не берём]**
- **Datums**: выбор datum, User Datums, Display Datum — **[не берём]** (только предупреждение)
- **Moving Map (real-time)**: Proximity Waypoints, Anchor Alarm, Alarm Zones, Range Rings,
  Regional Map Window, User Pointers — **[не берём]** (нет live-GPS)
- **Navigation**: AIS (Automatic Ship Identification), Auto Pilot — **[не берём]**
- **GPS Receivers**: Garmin, Magellan, Lowrance/Eagle, MLR, Brunton/Silva, Tripmate,
  Earthmate, Bluetooth/USB/NMEA/Windows GPS; upload/download waypoints/tracks/routes — **[не берём]**
- **Объекты карты**
  - Waypoints (свойства, список, проекция точки) — **[ЯДРО]** ✅; вложения (фото к точке) — **нет**, см. `docs/backlog.md`
  - Tracks (Track Control, List, Profile, Filter, проекция точки) — **[ЯДРО]** ✅ (Filter = упрощение Дугласа–Пекера с превью); **Replay** и **Move** — нет, см. `docs/backlog.md`
  - Routes (Route Editor, свойства) — **[не берём]**
  - Events, Map Features, Map Comments, Points (Point Sets) — **[не берём]** (Points ≈ waypoints концептуально)
- **Import / Export**: text-файлы, MapInfo MIF, ESRI Shape, ArcInfo E00 — **[не берём]**
  (у нас вместо этого GPX/PLT — см. ниже)
- **Измерения**: Distance & Area ✅ (отдельными инструментами), Distance/Bearing ✅ (проекция точки), круг по радиусу ✅; Distance Between Waypoints ✅ (линейка притягивается к метке и называет обе)
- **Grids**: отображение сетки Lat/Lon и других (UTM, OSGB, Irish) — **[позже]**
- **Printing**: печать карт и списков — **[не берём]** ([ADR-0023](../adr/adr-0023-no-map-printing.md))
- **OziExplorer3D / Elevation**: 3D-рельеф из DEM (USA DEM, DTED, GTOPO30, GLOBE) — **[не берём]**

## Форматы файлов (главное для нас)

Все файлы объектов — **текстовые ASCII**, с общими конвенциями:

- **Строка 1** — сигнатура и версия (`OziExplorer <Type> File Version X.Y`).
- **Строка 2** — geodetic datum координат в этом файле (напр. `WGS 84`).
- Далее — зарезервированные строки (должны присутствовать) и данные, **по одной записи
  на строку**, поля разделены запятыми.
- Пустые необязательные поля оставляются пустыми, но **запятые-разделители сохраняются**
  (`,,`); хвостовые поля можно опускать целиком.
- Запятая внутри текстового поля запрещена — вместо неё символ **`chr(209)`** (при чтении
  заменяется на запятую).
- **Цвета** записываются как десятичный Windows **COLORREF** = `0x00BBGGRR` (порядок байт
  B-G-R, а не RGB — руководство называет их «RGB value», но фактически это BGR).
  Примеры: `65535` = `0x0000FFFF` → жёлтый; `16711680` = `0x00FF0000` → синий;
  `16711935` = `0x00FF00FF` → пурпурный. В ozi-rs это учтено: `colorref_to_rgba`
  (`infrastructure/import/plt.rs`).
- **Даты** — Delphi **TDateTime**: целая часть = число дней с `1899-12-30`, дробная = время
  суток. Пример: `36169.6307194` ≈ 09-Jan-1999 15:08. В ozi-rs: `ole_date_to_chrono`.
- **Высоты** — всегда в **футах**; sentinel «нет данных» = **`-777`**.

### `.map` — калибровка карты

Привязывает изображение к координатам. Строки в фиксированном порядке:

```
OziExplorer Map Data File Version 2.2          ← сигнатура (видели 2.1 / 2.2)
World Map                                       ← 2: заголовок карты (любой текст)
world.ozf2                                      ← 3: путь/имя файла изображения
1 ,Map Code,                                    ← 4: legacy (или "1  TIFF scale factor") — не изменяется, но обязательна
WGS 84,,   0.0000,   0.0000,WGS 84              ← 5: datum (первое поле — используется; далее сдвиги datum)
Reserved 1                                      ← 6
Reserved 2                                      ← 7
Magnetic Variation,,,E                          ← 8: degrees,minutes,,dir(E/W)
Map Projection,Mercator,PolyCal,No,AutoCalOnly,No,BSBUseWPX,No   ← 9: проекция + флаги
Point01,xy,   59,  151,in, deg,  80,  0.0,N, 170,  0.0,W, grid, , , ,N   ← 10: до 30 калибровочных точек
...
Point30,xy,     ,     ,in, deg,    ,    ,N,    ,    ,W, grid, , , ,N
Projection Setup, 0.0, 152.0, 1.0, 2500000.0, 100000.0, -24.667, -27.333,,,   ← параметры проекции
Map Feature = MF ; Map Comment = MC     These follow if they exist   ← маркер
Track File = TF      These follow if they exist                      ← маркер
Moving Map Parameters = MM?    These follow if they exist            ← маркер
MM0,Yes                     ← использовать карту в Moving Map
MMPNUM,4                    ← число угловых калибровочных точек
MMPXY,1,3,2                 ← угол №1 в пикселях (x,y)
MMPLL,1,-179.0, 83.575619   ← угол №1 в lon,lat
MM1B,18975.755534           ← метров на пиксель
MOP,Map Open Position,0,0
IWH,Map Image Width/Height,2108,1912   ← размер растра в пикселях
```

Формат **калибровочной точки** `Point0N`:
`Point0N , xy , <pixel_x> , <pixel_y> , in , deg , <lat_deg> , <lat_min> , <N|S> ,
<lon_deg> , <lon_min> , <E|W> , grid , <zone> , <easting> , <northing> , <hemisphere>`.
То есть широта/долгота задаются в **градусах + десятичных минутах** с указанием полушария;
альтернативно — через grid-зону (UTM). Пустые точки (координаты не заданы) присутствуют как
плейсхолдеры. В UI OziExplorer правятся только 9 точек, но файл может хранить все 30.

Параметры `Projection Setup` по порядку: Latitude Origin, Longitude Origin (central
meridian), K Factor, False Easting, False Northing, Latitude 1, Latitude 2, Height, Sat, Path.

В ozi-rs: `infrastructure/import/ozi_map.rs` (метаданные, тип растра `Ozf2`/`Ozfx3`) и
`infrastructure/import/ozi_georeference.rs` (калибровочные точки → **аффинное преобразование**
pixel↔lat/lon). **Datum не трансформируется**, координаты берутся как есть; проекция читается
как строка, репроецирование не выполняется — для наших карт (локальный район, привязка по
углам) этого достаточно.

### `.plt` — трек (Track Point File)

```
OziExplorer Track Point File Version 2.0
WGS 84                              ← datum
Altitude is in Feet                 ← напоминание
Reserved 3
0,2,16711680,Demo Track 1        ,0 ← шапка трека (8 полей, см. ниже)
1979                                ← число точек (при чтении игнорируется)
 -26.856090, 152.960782,0,-777      ← точки: lat, lon, code, altitude[, date, dateStr, timeStr]
 ...
```

Шапка трека (строка 5): `0` (всегда), width (ширина линии на экране, 1–2), **color**
(COLORREF), description (без запятых), skip value (прореживание при отрисовке), **track type**
(`0`=normal, `10`=closed polygon, `20`=Alarm Zone), fill style (`bs*` 0–7), fill color.

Точка трека: **latitude**, **longitude**, **code** (`0`=обычная, `1`=разрыв линии / начало
нового сегмента), **altitude** (футы, `-777`=нет), опц. **date** (TDateTime — источник
истины), date-строка и time-строка (игнорируются при чтении).

В ozi-rs: импорт `infrastructure/import/plt.rs`, экспорт `infrastructure/export/plt.rs`
(PLT 2.1, cp1251, COLORREF BGR, OLE-даты). Декодирование байт: BOM → UTF-8 → chardetng →
fallback **Windows-1251** (важно для кириллических имён из старых файлов).

### `.wpt` — точки (Waypoint File), 24 поля

Строки 1–4: сигнатура, datum, Reserved, «GPS Symbol set» (не используется). Далее одна точка
на строку, поля:

1. **Number** — слот в GPS (для Lowrance/Eagle/Silva уникальный; для прочих `-1`)
2. **Name**
3. **Latitude** (decimal degrees)
4. **Longitude**
5. **Date** (TDateTime)
6. **Symbol** (индекс символа)
7. **Status** (всегда `1`)
8. Map Display Format
9. **Foreground Color** (COLORREF)
10. **Background Color**
11. **Description** (макс. 40, без запятых)
12. Pointer Direction
13. Garmin Display Format
14. Proximity Distance (`0`=выкл)
15. **Altitude** (футы, `-777`)
16. Font Size · 17. Font Style (`0`/`1`) · 18. Symbol Size (`17`=норм)
19. Proximity Symbol Position · 20. Proximity Time · 21. Proximity/Route/Both
22. File Attachment Name · 23. Proximity File Attachment Name · 24. Proximity Symbol Name

В ozi-rs: **только экспорт** — `infrastructure/export/wpt.rs` (версия 1.1, cp1251,
[ADR-0022](../adr/adr-0022-wpt-waypoint-export.md)). Импорта `.wpt` пока нет (точки к нам
приходят через GPX).

### `.rte` — маршруты (Route File)

Две записи: заголовок маршрута и его точки.

- **Route record**: `R, <number>, <name>, <description>, <route color COLORREF>`
- **Route waypoint**: `W, <route_number>, <seq (игнорируется)>, <wp_number>, <name>,
  <lat>, <lon>, <date>, <symbol>, <status=1>, <mapDisplay>, <fgColor>, <bgColor>,
  <description>, <pointerDir>, <garminDisplay>` (те же поля, что у waypoint).

Файл содержит фиксированный массив маршрутов (пустые `R,N,RN,` — плейсхолдеры).
В ozi-rs **не поддерживается** (маршруты — non-goal).

### `.evt` — события (Event File)

`<number>, <lat>, <lon>, <symbol>, <mapDisplay=0>, <fgColor>, <bgColor>, <symbolSize=17>`.
Максимум 1000. В ozi-rs **не поддерживается** (non-goal).

### `.pnt` — наборы точек (Point Sets)

Строка 5 — свойства набора: fore color, back color, size, font size, format, style,
description. Точки: `<lat>, <lon>, <rotation>, <name>, <desc1>, <desc2>, <desc3>`.
Концептуально близко к waypoints, но отдельный тип. В ozi-rs **не поддерживается**.

### `.ozp` — проект OziExplorer

Табличный (TAB-разделённый) индекс связанных файлов:

```
PH1	OziExplorer Project File
PH2	Version 1.0
PM1	Maps\Demo1.map        ← карта (слот 1)
PWF	Data\Demo1.wpt        ← waypoint-файл
PTF	Data\Demo1.plt        ← track-файл
```

Коды: `PHn` header, `PMn` map slot, `PWF` waypoint file, `PTF` track file (и др.).
**Важно:** ozi-rs использует то же расширение `.ozp`, но это **свой JSON-формат**
(`serde_json`, [ADR-0004](../adr/adr-0004-json-project-persistence.md)) — нативный
проект OziExplorer мы **не читаем**, только заимствовали расширение.

### Форматы изображений карт

| Формат | Что это | ozi-rs |
| --- | --- | --- |
| `OZF2` | Проприетарный сжатый пирамидальный растр (legacy, только чтение) | **Декодируется** (крейт `ozf2-rs`, [ADR-0006](../adr/adr-0006-ozf2-sibling-crate.md)) |
| `OZFx3` | Следующее поколение OZF | Классифицируется; полноценный рендер не подтверждён |
| `OZF4` | Текущий формат (Img2ozf) | **Не поддерживается** |
| `ECW`, `JP2` | Wavelet-сжатие (large maps) | Не поддерживается |
| `TIF/GeoTIFF`, `PNG`, `JPG`, `BMP` | Обычные растры (BMP — единственный в shareware) | Не как карты |
| `BSB/KAP`, `SID` (MrSID) | Морские карты / MrSID | Не поддерживается |

## Проекции и грид-системы

OziExplorer поддерживает следующие проекции (полный набор задаётся в калибровке `.map`):
**Latitude/Longitude**, **Mercator**, **Transverse Mercator** (= Gauss Conformal; основа для
UTM, Gauss-Krüger, German/Dutch/Israeli Grid через «User Grid»), **(UTM) Universal Transverse
Mercator**, **(BNG) British National Grid** (OSGB), **(IG) Irish National Grid**,
**(NZG) New Zealand Grid**, **(NZTM2) New Zealand TM 2000**, **(SG) Swedish Grid**,
**(SUI) Swiss Grid**, **Lambert Conformal Conic**, **Sinusoidal**, **Polyconic (American)**,
**Albers Equal Area**; плюс отдельные **France Grids**.

Для ozi-rs это в основном **[позже]/[не берём]**: наши карты привязываются аффинно по
калибровочным точкам, без строгого репроецирования (репроецирование и управление проекциями —
явный non-goal MVP).

## Датумы

Datum — референсный эллипсоид + сдвиг/поворот, определяющий, как координаты ложатся на карту.
OziExplorer различает три роли:

- **Map Datum** (в калибровке) — должен совпадать с datum, в котором нарисована карта.
- **Data File Datum** — datum координат в файле объектов.
- **GPS Upload/Download Datum** — datum, который ожидает GPS.

Приложение знает **много датумов** (сотни, зашиты в бинарь) и умеет пересчитывать координаты
между ними, плюс поддерживает **User Datums**. Типичные: `WGS 84`, `AGD66/84` (Австралия),
`NAD27/NAD83` (США), `European 1950`, `OSGB` (Airy). Полный список из распакованных данных
извлечь не удалось (ресурсы exe упакованы); наличие `Pulkovo 1942 / СК-42` (важно для РФ)
**не подтверждено** — при необходимости проверить в UI/на сайте.

В ozi-rs полноценной трансформации датумов нет: если datum карты/данных **не из семейства
WGS-84**, показывается предупреждение о возможном смещении ~100–150 м
(`application/mod.rs`: `warn_on_non_wgs84_datum`, `datum_shift_warning`). Само управление
датумами — non-goal.

**Решение владельца (2026-07-16): рабочий датум фиксируется — только `WGS 84`.**
Импортируемые данные (GPX от современных навигаторов, наши бандлы, maps.lizaalert.ru) и так
в WGS 84, поэтому список датумов оригинала для нас имеет лишь исторический интерес, а
трансформация датумов не нужна.

## Лимиты оригинала

| Объект | Лимит |
| --- | --- |
| Waypoints (загружено) | 100 000 |
| Events | 1 000 |
| Routes / waypoints на маршрут | 100 / 300 |
| Tracks / точек на трек | 1 000 / без лимита |
| Point Sets / точек на набор | 1 000 / без лимита |
| Map Features / Map Comments на карту | 500 / 500 |
| Вложений на карту (track / point-set / wpt / evt / rte) | 50 / 50 / 1 / 1 / 1 |

Фактические лимиты объектов, выгружаемых в GPS, дополнительно ограничены моделью GPS
(вкладка GPS в конфигурации).

## К чему стремимся (сопоставление с ozi-rs)

ozi-rs берёт **ядро OziExplorer** — растровые калиброванные карты + треки + точки — и
переносит его в современный, узко заточенный под SAR инструмент: кроссплатформенность
(macOS + Windows), offline-first, командная модель редактирования с delta undo/redo,
Command Palette, темизация, локализация (ru/en). «Сделать удобнее» = убрать перегруженность
оригинала и оставить рабочий поток координатора штаба.

| Область оригинала | Берём? | Статус в ozi-rs / как делаем удобнее |
| --- | --- | --- |
| Растровые карты, калибровка `.map` | **Ядро** | Есть: OZF2, обычные картинки (JPEG/PNG/TIFF/BMP/GIF/WebP) и SQLite/MBTiles тайлы; аффинная привязка; привязка картинки без `.map` пишет настоящий `.map` рядом. Плюс бандлы «ЛизаАлерт» |
| Треки: `.plt` импорт/экспорт, GPX | **Ядро** | Есть. Редактирование move/delete/insert/draw, упрощение (Douglas-Peucker) с превью |
| Треки: sort по времени, crop, split/join, обрезка по точке | **Ядро** | Есть, с UI в таблице сегментов и инспекторе; каждый шаг откатывается |
| Точки (waypoints) | **Ядро** | Есть: add/move/rename/symbol/цвет/заметка/visibility; импорт и экспорт `.wpt` и GPX. Вложений (фото) нет |
| On-map инструменты: линейка, площадь, круг (центр+радиус), проекция точки (азимут+дистанция) | **Берём** | Есть все четыре. Длина и площадь — отдельные инструменты (решение владельца 23.09); круг называет и радиус, и площадь |
| Проекции / репроецирование | Позже/нет | Проекция читается как строка; репроецирования нет (non-goal) |
| Управление датумами | **Не берём** | Только предупреждение о non-WGS84 |
| Moving Map, live-GPS, тревоги, AIS, автопилот | **Не берём** | Нет обмена с GPS-устройствами (non-goal) |
| Routes, Events, Point Sets `.pnt` | **Не берём** | Non-goal |
| GIS-импорт (MapInfo/Shape/E00), морские/DRG/ECW/SID импорты | **Не берём** | Non-goal; вместо этого GPX/PLT |
| Печать карт и списков | **Не берём** | [ADR-0023](../adr/adr-0023-no-map-printing.md) |
| 3D / рельеф из DEM | **Не берём** | Non-goal |
| Seamless maps, Name Search, Index Map | Позже | Возможные улучшения поиска/склейки карт |
| Track Replay (проигрывание трека во времени) | Позже | Нет. «Где была группа в 14:30» отвечается таблицей точек с временами |
| Track Move (сдвиг трека целиком) | Позже | Нет. В поиске почти не нужен: трек — это запись, а не чертёж |
| Вложения к точке (фото находки) | Позже | Нет. Требует хранения файлов в `.ozp`, то есть смены формата |
| Distance Between Waypoints | **Берём** | Есть с 23.09, но иначе: не диалог со списком пар, а линейка, которая притягивается к метке |
| Сетка координат на карте (Lat/Lon) | **Берём** | Есть с 23.09: шаг по зуму, всегда на круглом числе, подписи по краям. UTM и национальные сетки — нет |

> **GPS-устройства.** Целевые навигаторы пользователей (напр. Garmin **GPSMAP 78**) отдают
> треки и точки файлами (GPX и родственные форматы), которые ozi-rs импортирует напрямую.
> Поэтому живой обмен с железом (протоколы Garmin/NMEA, Moving Map) не нужен — данные
> попадают в проект на уровне файлов, а не через кабель/порт.

Границы scope зафиксированы в [`requirements.md`](../requirements.md) (MVP Scope / Non-Goals)
и [ADR-0020](../adr/adr-0020-mvp-scope.md). Терминология проекта — в [`glossary.md`](../glossary.md).

> Сверено с кодом 2026-09-23.

## Источники

- Официальный сайт и текущее семейство: <https://www.oziexplorer4.com/>
- Форматы карт для мобильных (OZF2/OZFx3/OZF4/ECW/JP2): <https://www.oziexplorer4.com/android/using_maps.html>
- Конвертер Img2ozf: <https://www.oziexplorer4.com/img2ozf/img2ozf.html>
- Форматы файлов и функциональность — встроенное руководство `OziExplorer.chm` (v3.95), разделы
  «OziExplorer File Formats», «Map File Format», «Map Projections», «Datums», «Limitations».
