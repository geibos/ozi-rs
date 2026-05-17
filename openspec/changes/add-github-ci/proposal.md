## Why

В репозитории нет автоматических проверок: clippy, type-check, тесты, валидация OpenSpec и сборка Tauri-приложения запускаются только локально через `just`. Это значит, что регрессии в Rust, фронтенде или OpenSpec-документации легко попадают в `main`, а релизные артефакты под macOS / Windows собираются вручную и не воспроизводимы. CI на GitHub Actions нужен, чтобы каждый PR и push в `main` проверялись одинаково на трёх платформах, а релизные бинарники собирались из тегов.

## What Changes

- Добавить GitHub Actions workflow `ci.yml`, который на каждом PR в `main`, push в `main`, ручном `workflow_dispatch` и tag push выполняет полный набор гейтов:
  - Rust: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo check`, `cargo test` (через `just clippy` / `just check` / `just test-rust`).
  - Frontend: `npm ci`, ESLint (`just lint`), `svelte-check` (`just check-ui`), Vitest (`just test-ui`).
  - OpenSpec: `openspec validate --strict` для всех активных изменений + проверка структуры specs.
  - Доп. аудит: `cargo audit` (RustSec), `npm audit --omit=dev --audit-level=high` для прод-зависимостей фронтенда.
- Добавить workflow `release.yml`, который по push тега `v*` собирает релизный Tauri-бандл под macOS (universal) и Windows (x86_64) через `tauri-apps/tauri-action`, прикрепляет артефакты к GitHub Release.
- Добавить файлы конфигурации, нужные CI: pin версии toolchain (`rust-toolchain.toml`), cache key strategy, опциональный `cargo-audit` install step.
- Документировать процесс в `docs/ci.md` и сослаться на него из `AGENTS.md` / `CLAUDE.md` ("Workflow" блок).

## Capabilities

### New Capabilities

- `ci-pipeline`: автоматическая верификация (Rust, фронтенд, OpenSpec, аудит) и релизная сборка через GitHub Actions. Описывает требования к гейтам PR, матрице платформ, кешированию, релизной публикации.

### Modified Capabilities

_(нет — CI не меняет user-visible behavior существующих capability)_

## Impact

- Новые файлы: `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `rust-toolchain.toml`, `docs/ci.md`.
- Затрагиваемые файлы: `AGENTS.md`, `CLAUDE.md`, `justfile` (возможно добавление `just ci` алиаса), `package.json` (без изменений зависимостей, только при необходимости — `lint:audit`).
- Внешние зависимости: GitHub-runners (`ubuntu-latest`, `macos-latest`, `windows-latest`), action-плагины `actions/checkout@v4`, `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2`, `actions/setup-node@v4`, `tauri-apps/tauri-action@v0`, `taiki-e/install-action@v2` (для `cargo-audit`).
- Секреты: ничего нового для основного CI. Для подписи macOS-релиза в будущем могут потребоваться `APPLE_*` секреты — вне scope этого change (только подготовить точку расширения).
- Стоимость CI-минут: основной job на `ubuntu-latest`, тяжёлые сборки (`macos-latest`, `windows-latest`) — только при release/tag push, плюс smoke-build на PR (без bundle, чтобы экономить минуты).
