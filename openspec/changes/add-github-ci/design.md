## Context

Сейчас все проверки проекта (Rust clippy/check/test, Svelte type-check + ESLint, Vitest, валидация OpenSpec) живут в `justfile` и запускаются вручную. Релизные бандлы Tauri собираются локально. Это работает, пока проект ведёт один разработчик на macOS, но:

- Регрессии под Windows / Linux не ловятся, а целевые пользователи LizaAlert используют все три платформы.
- `cargo clippy -- -D warnings` строго запрещает варнинги (см. ADR в `docs/adr/`) — нужна автоматическая enforcement-точка.
- Каждое user-visible изменение проходит через OpenSpec; нужно гарантировать, что предложения проходят `openspec validate --strict`, иначе они не архивируются.
- Релизы под macOS / Windows требуют воспроизводимой сборки из тегов, иначе теряется auditability.

Доступные ресурсы: бесплатные GitHub-runners (`ubuntu-latest`, `macos-latest`, `windows-latest`). Подписи / нотарификации Apple вне scope.

## Goals / Non-Goals

**Goals:**

- Запускать полный набор гейтов (Rust + frontend + OpenSpec + аудит) на каждом PR в `main` и push в `main`.
- Обеспечить кросс-платформенную проверку сборки Rust + frontend на Linux/macOS/Windows.
- Производить релизные Tauri-бандлы под macOS (universal) и Windows (x86_64-msvc) при push тега `v*` и публиковать в GitHub Release.
- Минимизировать время CI: кеширование cargo / npm, fail-fast по очевидным гейтам (fmt → clippy → check → test).
- Сохранить совместимость с `just`: jobs зовут существующие `just` recipes, чтобы локальная и CI-команды совпадали.

**Non-Goals:**

- Apple-нотарификация / подпись DMG / Windows code signing — отдельный change, требует секретов и согласований.
- Linux-релиз (AppImage / deb / rpm) — пользователи LizaAlert в основном Windows + macOS; добавим позже при запросе.
- Аппиум / native-QA через MCP в CI — недетерминированно, остаётся локальной верификацией (см. `docs/agent-verification.md`).
- Деплой / публикация на сторонние реестры (Homebrew, winget) — out of scope.
- Замена / отказ от `just`: CI зовёт `just` recipes как тонкую обёртку.

## Decisions

### D1. Два workflow вместо одного

**Решение:** `ci.yml` (PR / push / manual) + `release.yml` (tag push).

**Альтернативы:** один workflow с условной матрицей — отвергнут, потому что:
- Релизный pipeline тяжёлый (build + bundle + upload), запускать на каждом PR — расточительно.
- Логика триггеров и permissions разная (release нуждается в `contents: write`, CI — нет).

### D2. Матрица OS только для тяжёлых job-ов

**Решение:** Базовые гейты (fmt, clippy, check, lint, type-check, unit-тесты, openspec validate, audit) — на `ubuntu-latest`. Полная сборка Tauri (без bundle, `--no-bundle`) — матрица `{ubuntu, macos, windows}` на PR. Релизный bundle — `macos-latest` + `windows-latest`.

**Почему:** clippy/тесты Rust одинаковы на всех ОС с точки зрения логики; разные платформенные ошибки ловятся при компиляции Tauri (linkage с webview2 / WKWebView / webkit2gtk). Прогонять unit-тесты на трёх ОС — экономически невыгодно.

**Альтернатива:** полная матрица для всех job — отклонена (≈3× CI-минуты без существенного выигрыша).

### D3. Кеширование

**Решение:**
- Rust: `Swatinem/rust-cache@v2` с key, привязанным к `src-tauri/Cargo.lock` + платформа.
- Node: `actions/setup-node@v4` с `cache: 'npm'` и `cache-dependency-path: package-lock.json`.
- `cargo-audit`: установка через `taiki-e/install-action@v2` (бинарь, без компиляции).

**Почему:** первая холодная сборка Tauri ~7–10 мин на ubuntu, ~12–15 мин на windows. Кеш режет до 2–3 мин.

### D4. Версия toolchain

**Решение:** Добавить `rust-toolchain.toml` с pinned stable-версией (например, `1.84.0`) + компоненты `clippy`, `rustfmt`. Node version из `.nvmrc` (создать; pin `lts/iron` = 20.x).

**Почему:** воспроизводимость; разработчики локально и CI получают одинаковую toolchain. Альтернатива `dtolnay/rust-toolchain@stable` без пина — отклонена, т.к. может неожиданно сломаться при апдейте stable.

### D5. OpenSpec в CI

**Решение:** Установить `@openspec/cli` (или эквивалент) через `npm` и запустить `openspec validate --strict` для всех активных изменений. Если CLI не установлен глобально — использовать `npx`. Дополнительно: проверить, что все папки `openspec/changes/*` содержат обязательные артефакты (`proposal.md`, `tasks.md`).

**Почему:** OpenSpec — обязательный gate (см. `AGENTS.md`). Без CI он держится только на дисциплине.

### D6. Аудит зависимостей

**Решение:**
- Rust: `cargo audit --file src-tauri/Cargo.lock` через `taiki-e/install-action@v2`. **Без** `--deny warnings`: vulnerabilities (severity advisories) валят CI, а informational warnings (unmaintained / unsound) — нет. Причина: транзитивные зависимости Tauri / gtk-rs выдают 20+ unmaintained-warnings, которые ozi-rs не может починить напрямую (только через upstream Tauri bump). Делать их блокирующими — превратить CI в шум.
- Игнорируемые advisories — через `.cargo/audit.toml`, каждая запись требует комментария с причиной и recheck-условием (например, «ждать Tauri bump»). На момент создания change в ignore три уязвимости `rustls-webpki 0.103.10` (RUSTSEC-2026-0098/0099/0104), приходящие цепочкой `tauri → reqwest → quinn → rustls-webpki`.
- Frontend: `npm audit --omit=dev --audit-level=high` — только prod-зависимости, только high+critical. Dev-зависимости (eslint, vitest) шумные и не попадают в бандл.

**Альтернатива:** Dependabot/Renovate — отдельная задача, не блокирует этот change.

### D7. Релизный workflow через `tauri-action`

**Решение:** Использовать `tauri-apps/tauri-action@v0` с `tagName: ${{ github.ref_name }}`, `releaseName: ${{ github.ref_name }}`, `releaseDraft: true`. Сборка matrix `[macos-latest, windows-latest]`. Артефакты:
- macOS: `.dmg` + `.app.tar.gz` (universal через `--target universal-apple-darwin`).
- Windows: `.msi` + `.exe`.

**Permissions:** `contents: write` для создания release. Использует встроенный `GITHUB_TOKEN`, без секретов.

**Почему:** `tauri-action` — поддерживаемый maintainer'ами Tauri путь, абстрагирует target/bundle/upload.

### D8. Кросс-платформенные системные зависимости

**Решение:** На `ubuntu-latest` доустанавливать `libwebkit2gtk-4.1-dev`, `libsoup-3.0-dev`, `libjavascriptcoregtk-4.1-dev`, `librsvg2-dev`, `build-essential`, `curl`, `wget`, `file`, `libssl-dev`, `libayatana-appindicator3-dev`. macOS и Windows: ничего сверх дефолта.

**Почему:** Tauri 2 на Linux требует webkit2gtk-4.1 (см. tauri docs).

### D9. Concurrency control

**Решение:** На `ci.yml` использовать `concurrency: { group: ci-${{ github.ref }}, cancel-in-progress: true }`, чтобы при пуше нового коммита в ту же ветку старый прогон отменялся. На `release.yml` — без отмены (релизы не должны теряться).

## Risks / Trade-offs

- **Холодный кеш для PR от форков:** `actions/cache` для PR из форков ограничен (read-only). Митигация: документировать ожидаемое увеличенное время первой сборки; не блокировать.
- **macOS-runner-минуты дороги (10× от Linux):** только smoke-build на PR (`--no-bundle`), полный bundle — только при tag push. Митигация: monitorить usage, при превышении лимита перенести macOS-сборку в release-only.
- **`cargo audit` ложные срабатывания:** иногда RustSec помечает транзитивные зависимости без фикса. Митигация: allow-list через `.cargo/audit.toml` с явной фиксацией ignored advisories + justification в комментарии.
- **`npm audit` шум:** даже с `--omit=dev` бывают transitive issues без фикса. Митигация: `audit-level=high`, плюс возможность переопределить через `.npmrc` overrides; при невозможности фикса — `continue-on-error: true` для job audit + сообщать в summary, не блокировать merge.
- **rust-toolchain pin:** при апдейте Rust надо обновить файл и проверить clippy lints. Митигация: документировать в `docs/ci.md` процедуру апдейта.
- **Windows path issues в Cargo:** длинные пути / pwsh quoting. Митигация: использовать `bash` shell в job-ах через `defaults.run.shell: bash` где возможно; для Windows-only шагов — pwsh.
- **Время реакции CI:** даже с кешем PR-цикл ~8–12 мин. Trade-off приемлем — нет user-facing impact.

## Migration Plan

1. Добавить файлы `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `rust-toolchain.toml`, `.nvmrc`, `.cargo/audit.toml` (пустой allow-list).
2. Прогнать workflow на feature-ветке через `workflow_dispatch`, убедиться что все jobs зелёные.
3. Зафиксировать ожидания в `docs/ci.md` (список гейтов, ссылка на бейджи, процедура обновления toolchain).
4. Включить branch protection rules в GitHub UI (вне scope этого change как кодовое изменение — описать в `docs/ci.md`).
5. Откат: удалить файлы workflow / отключить в GitHub UI. Никаких side-effects в коде.

## Open Questions

- Нужно ли в `release.yml` сразу universal-binary под macOS или сначала только `aarch64-apple-darwin`? **Предложение:** universal (Tauri поддерживает `--target universal-apple-darwin`), решает оба архитектурных рынка одним артефактом.
- Включать ли в CI smoke-запуск приложения (headless через `xvfb` на Linux)? **Предложение:** не сейчас — desktop-QA остаётся локальной через `ozi-rs-mcp` (ADR-0024 запрещает Playwright как evidence).
- Pin конкретной stable-Rust версии: `1.84` или последняя на момент мержа? **Предложение:** на момент мержа change, документировать политику апдейта в `docs/ci.md`.
